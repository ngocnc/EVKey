//! Password-field detection for the Secure Input banner and the typing hook.
//!
//! A dedicated thread re-checks on focus and foreground changes (out-of-context WinEvent hooks, delivered through this
//! thread's message loop) and on a slow safety timer for fields that become password fields without any focus event.
//! UI Automation can block for seconds on hung apps, so it never runs inside the low-level keyboard hook; the hook
//! only reads the cached `SECURE` flag.
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, SetWinEventHook, UIA_DocumentControlTypeId, HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetForegroundWindow, GetGUIThreadInfo, GetMessageW, GetWindowLongW, GetWindowThreadProcessId,
    SetTimer, EVENT_OBJECT_FOCUS, EVENT_SYSTEM_FOREGROUND, GUITHREADINFO, GWL_STYLE, MSG, WINEVENT_OUTOFCONTEXT,
};

use crate::{banner, foreground};

const ES_PASSWORD: i32 = 0x20;
/// Safety re-check for password fields that appear without a focus or foreground event.
const SAFETY_POLL_MS: u32 = 2000;
const WM_TIMER: u32 = 0x0113;

static SECURE: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// UI Automation client of the Secure Input thread; WinEvent callbacks run on that thread.
    static AUTOMATION: RefCell<Option<IUIAutomation>> = const { RefCell::new(None) };
}

/// Cached result of the last check: true while a password field has focus. Cheap; safe to read from the hook.
pub fn active() -> bool {
    SECURE.load(Ordering::Relaxed)
}

/// Starts the Secure Input thread. Call once; the banner is notified from there.
pub fn start() {
    std::thread::spawn(|| unsafe {
        // The result is ignored on purpose: CoCreateInstance below reports the real failure.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let automation: Option<IUIAutomation> = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok();
        AUTOMATION.with(|a| *a.borrow_mut() = automation);
        for event in [EVENT_OBJECT_FOCUS, EVENT_SYSTEM_FOREGROUND] {
            SetWinEventHook(event, event, None, Some(on_focus_event), 0, 0, WINEVENT_OUTOFCONTEXT);
        }
        // With no window the system picks the timer id; WM_TIMER carries it in wParam.
        let _ = SetTimer(None, 0, SAFETY_POLL_MS, None);
        check();
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_TIMER {
                check();
                continue;
            }
            DispatchMessageW(&msg);
        }
    });
}

/// Re-reads the focused element and updates `SECURE` and the banner. Idempotent: the banner ignores unchanged state.
fn check() {
    AUTOMATION.with(|a| {
        let holder = holder_app(a.borrow().as_ref());
        SECURE.store(holder.is_some(), Ordering::Relaxed);
        // EVKey's own password field (the Gemini key) still blocks typing, but gets no banner: while EVKey is in
        // front, showing the banner activates it, the focus event that follows hides it, and the loop repeats.
        banner::notify(holder.filter(|_| !own_window_in_front()));
    });
}

fn own_window_in_front() -> bool {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut pid)) };
    pid == std::process::id()
}

unsafe extern "system" fn on_focus_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    check();
}

/// `Some(app)` while a password field has focus (`app` is "" when unknown), `None` otherwise.
fn holder_app(automation: Option<&IUIAutomation>) -> Option<String> {
    let viewport = if classic_password_focused() { None } else { Some(uia_password_focused(automation)?) };
    banner::set_anchor(viewport.flatten());
    Some(app_name(unsafe { GetForegroundWindow() }))
}

/// Classic edit controls report ES_PASSWORD. Browsers and UWP fields are only seen through UI Automation.
pub fn classic_password_focused() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        let tid = GetWindowThreadProcessId(fg, None);
        let mut info = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        if GetGUIThreadInfo(tid, &mut info).is_err() || info.hwndFocus.is_invalid() {
            return false;
        }
        GetWindowLongW(info.hwndFocus, GWL_STYLE) & ES_PASSWORD != 0
    }
}

/// `Some(viewport)` when the focused UI Automation element is a password field. `viewport` is the bounds of the
/// enclosing document (a browser's page area, which excludes vertical tab strips and sidebars), `None` if not found.
fn uia_password_focused(automation: Option<&IUIAutomation>) -> Option<Option<RECT>> {
    let automation = automation?;
    unsafe {
        let element = automation.GetFocusedElement().ok()?;
        if !element.CurrentIsPassword().ok()?.as_bool() {
            return None;
        }
        let walker = automation.ControlViewWalker().ok()?;
        let mut node = element;
        for _ in 0..40 {
            if node.CurrentControlType().is_ok_and(|t| t == UIA_DocumentControlTypeId) {
                return Some(node.CurrentBoundingRectangle().ok().filter(|r| r.right > r.left && r.bottom > r.top));
            }
            node = walker.GetParentElement(&node).ok()?;
        }
        Some(None)
    }
}

/// Executable name of `hwnd` without the `.exe` suffix ("" when unknown).
fn app_name(hwnd: HWND) -> String {
    let Some(exe) = foreground::exe_name(hwnd) else { return String::new() };
    if exe.to_ascii_lowercase().ends_with(".exe") {
        exe[..exe.len() - 4].to_string()
    } else {
        exe
    }
}
