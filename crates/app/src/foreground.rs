//! Remembers the last foreground app outside EVKey and the shell, so settings can exclude it in one click.
//! Shell windows (taskbar, desktop) belong to explorer.exe, so ignoring explorer.exe covers them.
use std::sync::Mutex;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, GetWindowThreadProcessId, TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG,
    WINEVENT_OUTOFCONTEXT,
};

static LAST_APP: Mutex<Option<String>> = Mutex::new(None);

pub fn last_app() -> Option<String> {
    LAST_APP.lock().ok()?.clone()
}

/// Starts the foreground hook on its own thread (out-of-context hooks need a message loop).
pub fn start() {
    std::thread::spawn(|| unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(on_foreground),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

unsafe extern "system" fn on_foreground(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    crate::hook::foreground_changed();
    let Some(name) = exe_name(hwnd) else { return };
    if name.eq_ignore_ascii_case("evkey.exe") || name.eq_ignore_ascii_case("explorer.exe") {
        return;
    }
    if let Ok(mut last) = LAST_APP.lock() {
        *last = Some(name);
    }
}

/// Executable file name (e.g. `chrome.exe`) of the process owning `hwnd`.
pub fn exe_name(hwnd: HWND) -> Option<String> {
    let mut pid = 0u32;
    let mut buf = [0u16; 512];
    let mut len = buf.len() as u32;
    let ok = unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(process);
        ok
    };
    let path = String::from_utf16_lossy(&buf[..len as usize]);
    ok.then(|| path.rsplit('\\').next().unwrap_or(&path).to_string())
}
