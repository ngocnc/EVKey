//! Password-field detection for the Secure Input banner and the typing hook.
//!
//! A dedicated thread polls every 300 ms: UI Automation (`IsPassword` on the focused element) plus the classic
//! `ES_PASSWORD` edit style. UI Automation can block for seconds on hung apps, so it never runs inside the
//! low-level keyboard hook; the hook only reads the cached `SECURE` flag.
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetGUIThreadInfo, GetWindowLongW, GetWindowThreadProcessId, GUITHREADINFO, GWL_STYLE,
};

use crate::{banner, foreground};

const ES_PASSWORD: i32 = 0x20;
const POLL: Duration = Duration::from_millis(300);

static SECURE: AtomicBool = AtomicBool::new(false);

/// Cached result of the last poll: true while a password field has focus. Cheap; safe to read from the hook.
pub fn active() -> bool {
    SECURE.load(Ordering::Relaxed)
}

/// Starts the poll thread. Call once; the banner is notified from there.
pub fn start() {
    std::thread::spawn(|| {
        // The result is ignored on purpose: CoCreateInstance below reports the real failure.
        let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let automation: Option<IUIAutomation> =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }.ok();
        loop {
            let holder = holder_app(automation.as_ref());
            SECURE.store(holder.is_some(), Ordering::Relaxed);
            // Idempotent and cheap when unchanged, so calling it every tick also covers a notify before the banner
            // was initialised.
            banner::notify(holder);
            std::thread::sleep(POLL);
        }
    });
}

/// `Some(app)` while a password field has focus (`app` is "" when unknown), `None` otherwise.
fn holder_app(automation: Option<&IUIAutomation>) -> Option<String> {
    if !classic_password_focused() && !uia_password_focused(automation) {
        return None;
    }
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

fn uia_password_focused(automation: Option<&IUIAutomation>) -> bool {
    let Some(automation) = automation else { return false };
    unsafe {
        automation
            .GetFocusedElement()
            .and_then(|element| element.CurrentIsPassword())
            .is_ok_and(|password| password.as_bool())
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
