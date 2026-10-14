//! UniKey-style typing backend: a low-level keyboard hook feeds the platform-independent `evkey_core::Session`, and
//! each answer is typed back with `SendInput` (backspaces, then the new characters). No Windows keyboard entry and
//! no DLL are needed. This file only maps Win32 state (modifiers, caps lock, foreground window, password fields)
//! into session inputs and injects what the session asks for.
//!
//! Everything runs on one dedicated thread, because hook callbacks are delivered on the thread that installed them.
//! Events this module injects carry `MARKER` in `dwExtraInfo` and are ignored when they come back through the hook.
use std::cell::RefCell;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, LazyLock};

use evkey_core::session::EMPTY_CHAR;
use evkey_core::{is_modifier_vk, KeyInput, Mods, Resend, Session};
use evkey_engine::settings;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_CAPITAL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, SetTimer, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL,
};

use crate::foreground;

/// Tags our own injected events ("EVKY").
const MARKER: usize = 0x4556_4B59;
static ACTIVE: AtomicBool = AtomicBool::new(false);
/// The Vietnamese/English flag. The session on the hook thread shares it, so other threads can read and set it.
static MODE: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));

/// True once the hook is installed; the V/E mode then lives in this process (see `keyboard::mode`).
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

pub fn english() -> bool {
    MODE.load(Ordering::Relaxed)
}

pub fn set_english(on: bool) {
    MODE.store(on, Ordering::Relaxed);
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

    /// Tells the session which app is frontmost (and so which remembered V/E mode applies). The app is looked up again
    /// only when the foreground window changes, so this is cheap enough for every key press, in either mode.
    fn refresh_foreground(&mut self) {
        let fg = unsafe { GetForegroundWindow() };
        if fg != self.fg {
            self.fg = fg;
            self.session.set_foreground(foreground::exe_name(fg).as_deref());
        }
    }

    /// Runs a few times a second on the hook thread, so a switch to another app restores that app's remembered mode
    /// at once (the tray icon follows) instead of at the next key press, and a mode changed from the tray is saved.
    fn poll(&mut self) {
        let fg = unsafe { GetForegroundWindow() };
        if fg != self.fg {
            let app = foreground::exe_name(fg);
            // EVKey's own popup or settings window: the user is still typing in the app behind it.
            if !app.as_deref().is_some_and(|a| a.eq_ignore_ascii_case("evkey.exe")) {
                self.fg = fg;
                self.session.set_foreground(app.as_deref());
            }
        }
        self.session.remember_current();
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
            self.refresh_foreground();
            if !is_modifier_vk(vk) && !self.session.is_english() {
                self.refresh_secure();
            }
        }
        let caps_lock = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
        let reaction = self.session.on_key(KeyInput { vk, down, mods: async_mods(), caps_lock });
        if reaction.hotkey_toggled && self.session.settings().toggle_sound {
            crate::keyboard::play_toggle_sound(self.session.is_english());
        }
        if reaction.translate_requested {
            crate::translate::trigger();
        }
        let mut input = Vec::new();
        if let Some(edit) = reaction.edit {
            if edit.prefix_empty_char {
                input.extend(unicode(EMPTY_CHAR));
            }
            for _ in 0..edit.backspaces {
                input.extend([key(VK_BACK.0, false), key(VK_BACK.0, true)]);
            }
            for unit in edit.text.encode_utf16() {
                input.extend(unicode(unit));
            }
            match edit.resend {
                Some(Resend::Char(ch)) => input.extend(unicode(ch)),
                Some(Resend::Vk(vk)) => input.extend([key(vk, false), key(vk, true)]),
                None => {}
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
const POLL_EVERY_MS: u32 = 250;

/// Installs the hooks on a new thread and waits for the result. On failure the app keeps running without typing.
pub fn start() -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        STATE.with(|s| *s.borrow_mut() = Some(State::new()));
        let mut hooks = match install() {
            Ok(h) => h,
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };
        ACTIVE.store(true, Ordering::Relaxed);
        crate::secure_input_win::start();
        let _ = tx.send(Ok(()));
        // With no window the system picks the timer ids; WM_TIMER carries the id in wParam.
        let reinstall_timer = unsafe { SetTimer(None, 0, REINSTALL_EVERY_MS, None) };
        let poll_timer = unsafe { SetTimer(None, 0, POLL_EVERY_MS, None) };
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            if msg.message == WM_TIMER && msg.wParam.0 == poll_timer {
                STATE.with(|s| {
                    if let Some(state) = s.borrow_mut().as_mut() {
                        state.poll();
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
