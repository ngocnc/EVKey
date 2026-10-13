//! UniKey-style typing backend: a low-level keyboard hook feeds the engine, and each answer is typed back with
//! `SendInput` (backspaces, then the new characters). No Windows keyboard entry and no DLL are needed.
//!
//! Everything runs on one dedicated thread, because hook callbacks are delivered on the thread that installed them.
//! Events this module injects carry `MARKER` in `dwExtraInfo` and are ignored when they come back through the hook.
use std::cell::RefCell;
use std::mem::size_of;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};

use evkey_engine::composer::Composer;
use evkey_engine::consts::CAPS_MASK;
use evkey_engine::hotkey::{parse_hotkey, Hotkey};
use evkey_engine::settings::{self, Settings, TempOffKey};
use evkey_engine::tables::key_code_to_character;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_CAPITAL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetGUIThreadInfo, GetMessageW, GetWindowLongW,
    GetWindowThreadProcessId, SetWindowsHookExW, TranslateMessage, GUITHREADINFO, GWL_STYLE, KBDLLHOOKSTRUCT, MSG,
    WH_KEYBOARD_LL, WH_MOUSE_LL,
};

use crate::foreground;
use crate::typing::edit_for;

/// Tags our own injected events ("EVKY").
const MARKER: usize = 0x4556_4B59;
/// Zero-width-ish character typed and deleted first so browser autocomplete cannot swallow the edit (as OpenKey does).
const EMPTY_CHAR: u16 = 0x202F;
const ES_PASSWORD: i32 = 0x20;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static ENGLISH: AtomicBool = AtomicBool::new(false);

/// True once the hook is installed; the V/E mode then lives in this process (see `keyboard::mode`).
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

pub fn english() -> bool {
    ENGLISH.load(Ordering::Relaxed)
}

pub fn set_english(on: bool) {
    ENGLISH.store(on, Ordering::Relaxed);
}

/// Modifier groups in `Hotkey` field order: ctrl, alt, shift, win.
type Groups = [bool; 4];

/// Group index of a modifier virtual key (the hook reports the left/right variants); `None` for other keys.
fn group(vk: u16) -> Option<usize> {
    match vk {
        0xA2 | 0xA3 | 0x11 => Some(0),
        0xA4 | 0xA5 | 0x12 => Some(1),
        0xA0 | 0xA1 | 0x10 => Some(2),
        0x5B | 0x5C => Some(3),
        _ => None,
    }
}

fn wanted(hk: &Hotkey) -> Groups {
    [hk.ctrl, hk.alt, hk.shift, hk.win]
}

/// Physical state of the modifier groups, read from the OS (used when a non-modifier key arrives).
fn async_groups() -> Groups {
    let down = |vks: &[u16]| vks.iter().any(|&v| unsafe { GetAsyncKeyState(v as i32) } as u16 & 0x8000 != 0);
    [down(&[0xA2, 0xA3]), down(&[0xA4, 0xA5]), down(&[0xA0, 0xA1]), down(&[0x5B, 0x5C])]
}

struct State {
    composer: Composer,
    settings: Settings,
    dir: Option<PathBuf>,
    stamp: Option<[Option<SystemTime>; 2]>,
    last_check: Option<Instant>,
    hotkey: Option<Hotkey>,
    /// Modifier groups currently held, tracked from the hook's own key events.
    held: Groups,
    /// Modifier-only chord (`ctrl+shift`): armed while exactly its modifiers are held and nothing else was pressed.
    chord_armed: bool,
    /// A lone modifier was pressed and nothing else since: releasing it is a "tap" (the temp-off key).
    tap_armed: bool,
    fg: HWND,
    excluded: bool,
    seen_english: bool,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

impl State {
    fn new() -> Self {
        let dir = settings::default_dir();
        let mut s = State {
            composer: Composer::new(Settings::default().to_config(false)),
            settings: Settings::default(),
            dir,
            stamp: None,
            last_check: None,
            hotkey: None,
            held: [false; 4],
            chord_armed: false,
            tap_armed: false,
            fg: HWND::default(),
            excluded: false,
            seen_english: false,
        };
        s.reload(true);
        s
    }

    /// Reloads `config.toml` / `macros.toml` when they changed (checked at most once a second). A malformed file
    /// keeps the previous settings. Everything but the hotkey waits for a word boundary.
    fn reload(&mut self, force: bool) {
        let now = Instant::now();
        if !force && self.last_check.is_some_and(|t| now.duration_since(t) < Duration::from_secs(1)) {
            return;
        }
        self.last_check = Some(now);
        let Some(dir) = self.dir.clone() else { return };
        let stamp = settings::stamp(&dir);
        if self.stamp == Some(stamp) {
            return;
        }
        let Ok(loaded) = settings::load(&dir) else {
            self.stamp = Some(stamp);
            return;
        };
        if let Ok(hk) = parse_hotkey(&loaded.settings.toggle_hotkey) {
            self.hotkey = hk;
        }
        if self.composer.is_composing() {
            // Stamp stays stale, so the next check applies the rest after the word ends.
            self.stamp = None;
            return;
        }
        self.stamp = Some(stamp);
        self.composer.apply(&loaded);
        self.settings = loaded.settings;
        self.fg = HWND::default(); // re-evaluate the exclusion list for the current window
    }

    fn toggle(&mut self) {
        set_english(!english());
        self.composer.reset();
    }

    /// Resets the word when the foreground window changed, and refreshes the exclusion flag for it.
    fn track_foreground(&mut self) {
        let fg = unsafe { GetForegroundWindow() };
        if fg != self.fg {
            self.fg = fg;
            self.composer.reset();
            self.excluded = foreground::exe_name(fg).is_some_and(|n| self.settings.is_excluded(&n));
        }
    }

    /// Handles one key event. Returns whether to swallow it, plus input to inject.
    fn on_key(&mut self, vk: u16, down: bool) -> (bool, Vec<INPUT>) {
        if english() != self.seen_english {
            self.seen_english = english();
            self.composer.reset();
        }
        if let Some(g) = group(vk) {
            self.on_modifier(g, down);
            return (false, Vec::new());
        }
        if !down {
            return (false, Vec::new());
        }
        self.tap_armed = false;
        self.chord_armed = false;
        self.held = async_groups();
        self.reload(false);

        if let Some(hk) = self.hotkey {
            if hk.vk == Some(vk) && self.held == wanted(&hk) {
                self.toggle();
                // Alt released with no key in between would open the window menu; a throwaway key prevents it.
                let dummy = if hk.alt { vec![key(0xFF, false), key(0xFF, true)] } else { Vec::new() };
                return (true, dummy);
            }
        }
        if english() {
            return (false, Vec::new());
        }
        self.track_foreground();
        if self.excluded || password_field_focused() {
            return (false, Vec::new());
        }
        let shift = self.held[2];
        let caps_lock = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
        let caps: u8 = if shift && caps_lock { 0 } else if shift { 1 } else if caps_lock { 2 } else { 0 };
        let other_control = self.held[0] || self.held[1] || self.held[3];
        let engine = &mut self.composer.engine;
        engine.key_event(false, vk, caps, other_control);
        let Some(edit) = edit_for(&engine.hook, engine.cfg.code_table) else {
            return (false, Vec::new());
        };
        let restart = engine.hook.code == evkey_engine::engine::V_RESTORE_AND_START_NEW_SESSION;
        if restart {
            engine.start_new_session();
        }
        let mut input = Vec::new();
        // ponytail: OpenKey's "fix recommend browser" for every app; per-app opt-out if some app dislikes it.
        input.extend(unicode(EMPTY_CHAR));
        for _ in 0..edit.backspaces + 1 {
            input.extend([key(VK_BACK.0, false), key(VK_BACK.0, true)]);
        }
        for unit in edit.text.encode_utf16() {
            input.extend(unicode(unit));
        }
        if edit.resend_key {
            let ch = key_code_to_character(vk as u32 | if caps != 0 { CAPS_MASK } else { 0 });
            if ch != 0 {
                input.extend(unicode(ch));
            } else {
                input.extend([key(vk, false), key(vk, true)]);
            }
        }
        (true, input)
    }

    fn on_modifier(&mut self, g: usize, down: bool) {
        let hk = self.hotkey;
        if down {
            self.held[g] = true;
            self.chord_armed = hk.is_some_and(|h| h.vk.is_none() && self.held == wanted(&h));
            self.tap_armed = self.held.iter().filter(|&&h| h).count() == 1;
            return;
        }
        // `held` for the released group is only cleared once no key of that group remains down.
        self.held = async_groups_after_release(g);
        if let Some(h) = hk {
            if h.vk.is_none() && self.chord_armed && wanted(&h)[g] {
                self.chord_armed = false;
                self.toggle();
            }
        }
        let temp_group = match self.settings.temp_off_key {
            TempOffKey::Ctrl => Some(0),
            TempOffKey::Alt => Some(1),
            TempOffKey::None => None,
        };
        if self.tap_armed && temp_group == Some(g) {
            self.tap_armed = false;
            self.composer.engine.temp_off_engine(true);
        }
    }
}

/// OS state of all groups, but group `g` forced up: its release event is being handled and the OS may not show it yet.
fn async_groups_after_release(g: usize) -> Groups {
    let mut h = async_groups();
    h[g] = false;
    h
}

/// Best effort: classic edit controls report ES_PASSWORD. Browsers and UWP fields are not detectable here.
fn password_field_focused() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        let tid = GetWindowThreadProcessId(fg, None);
        let mut info = GUITHREADINFO { cbSize: size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        if GetGUIThreadInfo(tid, &mut info).is_err() || info.hwndFocus.is_invalid() {
            return false;
        }
        GetWindowLongW(info.hwndFocus, GWL_STYLE) & ES_PASSWORD != 0
    }
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
                state.composer.reset();
            }
        });
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Installs the hooks on a new thread and waits for the result. On failure the app keeps running without typing.
pub fn start() -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        STATE.with(|s| *s.borrow_mut() = Some(State::new()));
        let kb = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) };
        let ms = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0) };
        match (kb, ms) {
            (Ok(_), Ok(_)) => {
                ACTIVE.store(true, Ordering::Relaxed);
                let _ = tx.send(Ok(()));
            }
            (kb, ms) => {
                let e = kb.err().or(ms.err()).map(|e| e.to_string()).unwrap_or_default();
                let _ = tx.send(Err(format!("SetWindowsHookEx: {e}")));
                return;
            }
        }
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
    rx.recv().map_err(|e| e.to_string())?
}
