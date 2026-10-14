//! Typing on Windows (ADR 0006): a low-level keyboard hook feeds the platform-independent `evkey_core::Session`, and
//! each answer is typed back with `SendInput` (backspaces, then the new characters). No Windows keyboard entry and
//! no DLL are needed. This file only maps Win32 state (modifiers, caps lock, foreground window, password fields)
//! into session inputs and injects what the session asks for.
//!
//! Everything runs on one dedicated thread, because hook callbacks are delivered on the thread that installed them.
//! Events this module injects carry `MARKER` in `dwExtraInfo` and are ignored when they come back through the hook.
use std::cell::RefCell;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, LazyLock};

use evkey_core::session::EMPTY_CHAR;
use evkey_core::{is_modifier_vk, KeyInput, Mods, Resend, Session};
use evkey_engine::settings;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_CAPITAL, VK_LEFT,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, PostThreadMessageW, SetTimer,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_APP,
};

use crate::{foreground, tray};

/// Tags our own injected events ("EVKY").
const MARKER: usize = 0x4556_4B59;
/// The Vietnamese/English flag. The session on the hook thread shares it, so other threads can read and set it.
static MODE: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));

pub fn english() -> bool {
    MODE.load(Ordering::Relaxed)
}

pub fn set_english(on: bool) {
    MODE.store(on, Ordering::Relaxed);
    tray::mode_changed();
    post_to_hook_thread(WM_APP_MODE);
}

/// Wakes the hook thread to re-read the frontmost app. Called from the foreground WinEvent hook.
pub fn foreground_changed() {
    post_to_hook_thread(WM_APP_FOREGROUND);
}

fn post_to_hook_thread(msg: u32) {
    let tid = HOOK_THREAD.load(Ordering::Relaxed);
    if tid != 0 {
        let _ = unsafe { PostThreadMessageW(tid, msg, WPARAM(0), LPARAM(0)) };
    }
}

/// Physical state of the modifier keys, read from the OS on each key event.
fn async_mods() -> Mods {
    let down = |vks: &[u16]| vks.iter().any(|&v| unsafe { GetAsyncKeyState(v as i32) } as u16 & 0x8000 != 0);
    Mods { ctrl: down(&[0xA2, 0xA3]), alt: down(&[0xA4, 0xA5]), shift: down(&[0xA0, 0xA1]), win: down(&[0x5B, 0x5C]) }
}

struct State {
    session: Session,
    /// Foreground window last seen; its app is looked up again only when it changes.
    fg: HWND,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

impl State {
    fn new() -> Self {
        State { session: Session::with_mode(settings::default_dir(), MODE.clone()), fg: HWND::default() }
    }

    /// Tells the session which app is frontmost (and so which remembered V/E mode applies). Runs on each key press;
    /// the app is looked up again only when the foreground window changes, so this is cheap.
    fn refresh_foreground(&mut self) {
        let fg = unsafe { GetForegroundWindow() };
        if fg != self.fg {
            self.fg = fg;
            self.set_app(foreground::exe_name(fg).as_deref());
        }
    }

    /// Sets the frontmost app on the session; the tray follows when that restores a different mode.
    fn set_app(&mut self, app: Option<&str>) {
        let before = self.session.is_english();
        self.session.set_foreground(app);
        if self.session.is_english() != before {
            tray::mode_changed();
        }
    }

    /// The foreground window changed (posted by the WinEvent hook), so a switch restores that app's remembered mode at
    /// once and the tray follows. EVKey's own popup or settings window is skipped: the user is still typing in the app
    /// behind it, so a mode toggled from the popup must be remembered for that app.
    fn on_foreground_event(&mut self) {
        let fg = unsafe { GetForegroundWindow() };
        if fg == self.fg {
            return;
        }
        let app = foreground::exe_name(fg);
        if !app.as_deref().is_some_and(|a| a.eq_ignore_ascii_case("evkey.exe")) {
            self.fg = fg;
            self.set_app(app.as_deref());
        }
    }

    /// Password-field state, needed only for keys that may be converted. The UIA poll result is cached; the classic
    /// ES_PASSWORD check is a zero-latency fast path.
    fn refresh_secure(&mut self) {
        self.session.set_secure(crate::secure_input_win::active() || crate::secure_input_win::classic_password_focused());
    }

    /// Handles one key event. Returns whether to swallow it, plus input to inject.
    fn on_key(&mut self, vk: u16, down: bool) -> (bool, Vec<INPUT>) {
        // Always, even in English mode: leaving an English app for a Vietnamese one must restore its remembered mode.
        if down {
            if vk == 0x1B {
                crate::actions::dismiss_menu(None);
            }
            self.refresh_foreground();
            if !is_modifier_vk(vk) && !self.session.is_english() {
                self.refresh_secure();
            }
        }
        let caps_lock = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
        let before = self.session.is_english();
        let reaction = self.session.on_key(KeyInput { vk, down, mods: async_mods(), caps_lock });
        if self.session.is_english() != before {
            tray::mode_changed();
        }
        if reaction.hotkey_toggled && self.session.settings().toggle_sound {
            crate::keyboard::play_toggle_sound(self.session.is_english());
        }
        if reaction.translate_requested {
            crate::actions::trigger(true);
        }
        if reaction.actions_requested {
            crate::actions::trigger(false);
        }
        let mut input = Vec::new();
        if let Some(edit) = reaction.edit {
            if edit.prefix_empty_char {
                input.extend(unicode(EMPTY_CHAR));
            }
            for _ in 0..edit.backspaces {
                input.extend([key(VK_BACK.0, false), key(VK_BACK.0, true)]);
            }
            let text = expand_vars(&edit.text);
            let (text, back) = split_cursor(&text);
            for unit in text.encode_utf16() {
                input.extend(unicode(unit));
            }
            match edit.resend {
                Some(Resend::Char(ch)) => input.extend(unicode(ch)),
                Some(Resend::Vk(vk)) => input.extend([key(vk, false), key(vk, true)]),
                None => {}
            }
            // `{cursor}`: step back over the text after it and the re-sent trigger key.
            if let Some(back) = back {
                for _ in 0..back + edit.resend.is_some() as usize {
                    input.extend([key(VK_LEFT.0, false), key(VK_LEFT.0, true)]);
                }
            }
        }
        if reaction.dummy_key {
            input.extend([key(0xFF, false), key(0xFF, true)]);
        }
        (reaction.swallow, input)
    }
}

/// Presses `modifier` + `vk` and releases both (e.g. Ctrl+C). Marked as ours, so the hook ignores it.
pub fn send_combo(modifier: u16, vk: u16) {
    let input = [key(modifier, false), key(vk, false), key(vk, true), key(modifier, true)];
    unsafe { SendInput(&input, size_of::<INPUT>() as i32) };
}

fn key(vk: u16, up: bool) -> INPUT {
    input(vk, 0, if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) })
}

/// Types one UTF-16 unit as a down/up pair.
fn unicode(unit: u16) -> [INPUT; 2] {
    [input(0, unit, KEYEVENTF_UNICODE), input(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP)]
}

/// Fills macro variables: `{date}` (dd/MM/yyyy), `{time}` (HH:mm), `{clipboard}` (current clipboard text).
/// ponytail: applied to every edit, not only macro output; typed words never contain `{`, so plain typing is untouched.
fn expand_vars(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains('{') {
        return text.into();
    }
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    fill_vars(text, (t.wDay, t.wMonth, t.wYear, t.wHour, t.wMinute), crate::clipboard::get_text).into()
}

fn fill_vars(text: &str, (d, mo, y, h, mi): (u16, u16, u16, u16, u16), clipboard: impl FnOnce() -> Option<String>) -> String {
    let mut out = text.replace("{date}", &format!("{d:02}/{mo:02}/{y}")).replace("{time}", &format!("{h:02}:{mi:02}"));
    if out.contains("{clipboard}") {
        out = out.replace("{clipboard}", &clipboard().unwrap_or_default());
    }
    out
}

/// Removes every `{cursor}` and returns the text plus how many characters follow the first one (`None` if absent).
fn split_cursor(text: &str) -> (std::borrow::Cow<'_, str>, Option<usize>) {
    const MARK: &str = "{cursor}";
    let Some(at) = text.find(MARK) else { return (text.into(), None) };
    let out = text.replace(MARK, "");
    let back = text[at + MARK.len()..].replace(MARK, "").chars().count();
    (out.into(), Some(back))
}

fn input(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: VIRTUAL_KEY(vk), wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: MARKER },
        },
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    const WM_KEYDOWN: usize = 0x100;
    const WM_KEYUP: usize = 0x101;
    const WM_SYSKEYDOWN: usize = 0x104;
    const WM_SYSKEYUP: usize = 0x105;
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }
    let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let down = match wparam.0 {
        WM_KEYDOWN | WM_SYSKEYDOWN => true,
        WM_KEYUP | WM_SYSKEYUP => false,
        _ => return unsafe { CallNextHookEx(None, code, wparam, lparam) },
    };
    if info.dwExtraInfo == MARKER {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }
    let (eat, input) = STATE.with(|s| match s.borrow_mut().as_mut() {
        Some(state) => state.on_key(info.vkCode as u16, down),
        None => (false, Vec::new()),
    });
    if !input.is_empty() {
        unsafe { SendInput(&input, size_of::<INPUT>() as i32) };
    }
    if eat {
        return LRESULT(1);
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// A click can move the caret, so the word being typed is forgotten.
unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // left, right, middle, X button down
    if code >= 0 && matches!(wparam.0, 0x201 | 0x204 | 0x207 | 0x20B) {
        let pt = unsafe { (*(lparam.0 as *const MSLLHOOKSTRUCT)).pt };
        crate::actions::dismiss_menu(Some((pt.x, pt.y)));
        STATE.with(|s| {
            if let Some(state) = s.borrow_mut().as_mut() {
                state.session.on_mouse_click();
            }
        });
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Installs both hooks. Windows silently removes a low-level hook whose callback ever times out (a busy machine is
/// enough), so `start` re-installs them on a timer instead of trusting them to live forever.
fn install() -> Result<(HHOOK, HHOOK), String> {
    let kb = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) };
    let ms = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0) };
    match (kb, ms) {
        (Ok(kb), Ok(ms)) => Ok((kb, ms)),
        (kb, ms) => {
            for h in [kb, ms].into_iter().flatten() {
                let _ = unsafe { UnhookWindowsHookEx(h) };
            }
            Err("SetWindowsHookEx failed".to_string())
        }
    }
}

const WM_TIMER: u32 = 0x113;
const REINSTALL_EVERY_MS: u32 = 5000;
/// Posted to the hook thread when another thread set the V/E flag, so that mode is remembered for the app.
const WM_APP_MODE: u32 = WM_APP + 1;
/// Posted by the foreground WinEvent hook (see `foreground_changed`).
const WM_APP_FOREGROUND: u32 = WM_APP + 2;
/// Hook thread id, so other threads can post to it.
static HOOK_THREAD: AtomicU32 = AtomicU32::new(0);

/// Installs the hooks on a new thread and waits for the result. On failure the app keeps running without typing.
pub fn start() -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        HOOK_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
        STATE.with(|s| *s.borrow_mut() = Some(State::new()));
        let mut hooks = match install() {
            Ok(h) => h,
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };
        crate::secure_input_win::start();
        let _ = tx.send(Ok(()));
        // With no window the system picks the timer ids; WM_TIMER carries the id in wParam.
        let reinstall_timer = unsafe { SetTimer(None, 0, REINSTALL_EVERY_MS, None) };
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            if msg.message == WM_APP_FOREGROUND {
                STATE.with(|s| {
                    if let Some(state) = s.borrow_mut().as_mut() {
                        state.on_foreground_event();
                    }
                });
                continue;
            }
            if msg.message == WM_APP_MODE {
                STATE.with(|s| {
                    if let Some(state) = s.borrow_mut().as_mut() {
                        state.session.remember_current();
                    }
                });
                continue;
            }
            if msg.message == WM_TIMER && msg.wParam.0 == reinstall_timer {
                // New hooks first, so there is never a moment without one.
                if let Ok(fresh) = install() {
                    unsafe {
                        let _ = UnhookWindowsHookEx(hooks.0);
                        let _ = UnhookWindowsHookEx(hooks.1);
                    }
                    hooks = fresh;
                }
                continue;
            }
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
    rx.recv().map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    #[test]
    fn fills_date_time_and_clipboard() {
        let got = super::fill_vars("{date} {time} [{clipboard}] {x}", (4, 7, 2026, 9, 5), || Some("abc".into()));
        assert_eq!(got, "04/07/2026 09:05 [abc] {x}");
        assert_eq!(super::fill_vars("{clipboard}", (1, 1, 2026, 0, 0), || None), "");
    }

    #[test]
    fn cursor_mark_is_removed_and_counts_chars_after_it() {
        assert_eq!(super::split_cursor("log({cursor});"), ("log();".into(), Some(2)));
        assert_eq!(super::split_cursor("a{cursor}bế{cursor}c"), ("abếc".into(), Some(3)));
        assert_eq!(super::split_cursor("plain"), ("plain".into(), None));
    }
}
