//! Single instance plus a tiny command channel: a second launch (or `--mode`) talks to the running app.
//! Commands: `settings`, `mode vietnamese|english|toggle`, `quit` (an elevated `--admin on` taking over).

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Duration;

use tauri::AppHandle;
use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::Security::Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1};
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_INBOUND;
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_WAIT,
};
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};

use crate::keyboard::mode::{self, Mode};
use crate::tray;

/// Named pipe a second launch uses to ask the running instance to show settings.
const PIPE_PATH: &str = r"\\.\pipe\EVKey-Show";

/// The single-instance mutex while this process holds it (0 = not held).
static INSTANCE: AtomicIsize = AtomicIsize::new(0);

/// True when this is the first instance (or already holds the mutex). The mutex handle stays open until the process
/// exits or `release_single_instance`. A mutex made by an elevated instance cannot be opened from a non-elevated
/// process (access denied), which also means one is running.
pub fn acquire_single_instance() -> bool {
    if INSTANCE.load(Ordering::SeqCst) != 0 {
        return true;
    }
    unsafe {
        let Ok(handle) = CreateMutexW(None, true, w!("Local\\EVKeySingleInstance")) else { return false };
        if GetLastError() == ERROR_ALREADY_EXISTS {
            // Keeping this handle would keep the mutex alive after the other instance exits.
            let _ = CloseHandle(handle);
            return false;
        }
        INSTANCE.store(handle.0 as isize, Ordering::SeqCst);
        true
    }
}

/// Lets another process become the instance (a non-elevated launch handing over to the elevated task).
pub fn release_single_instance() {
    let raw = INSTANCE.swap(0, Ordering::SeqCst);
    if raw != 0 {
        let handle = HANDLE(raw as *mut _);
        unsafe {
            let _ = ReleaseMutex(handle);
            let _ = CloseHandle(handle);
        }
    }
}

/// Sends `command` to the running instance. Retries briefly in case it is still starting its pipe.
pub fn send_to_running_instance(command: &str) -> Result<(), String> {
    for _ in 0..30 {
        if let Ok(mut pipe) = OpenOptions::new().write(true).open(PIPE_PATH) {
            return pipe.write_all(command.as_bytes()).map_err(|e| e.to_string());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(crate::i18n::tr("EVKey đang chạy nhưng không phản hồi", "EVKey is running but not responding").into())
}

/// Serves "show settings" requests from second launches. Runs forever on its own thread.
pub fn serve_show_requests(app: AppHandle) {
    // Explicit DACL: an elevated instance's default one would refuse writes from the user's normal (non-elevated)
    // launches. SYSTEM and Administrators: full; interactive users: read/write. Kept for the process lifetime.
    let mut sd = PSECURITY_DESCRIPTOR::default();
    let sddl_ok = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            w!("D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;IU)"),
            SDDL_REVISION_1,
            &mut sd,
            None,
        )
    }
    .is_ok();
    let sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.0,
        bInheritHandle: false.into(),
    };
    loop {
        let pipe = unsafe {
            CreateNamedPipeW(
                w!("\\\\.\\pipe\\EVKey-Show"),
                PIPE_ACCESS_INBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                0,
                64,
                0,
                sddl_ok.then_some(&sa as *const _),
            )
        };
        if pipe.is_invalid() {
            return;
        }
        unsafe {
            let _ = ConnectNamedPipe(pipe, None);
        }
        // `File` owns the handle: dropping it closes the pipe and disconnects the client.
        let mut file = unsafe { File::from_raw_handle(pipe.0) };
        let mut buf = [0u8; 64];
        if let Ok(n) = file.read(&mut buf) {
            let cmd = String::from_utf8_lossy(&buf[..n]);
            match cmd.as_ref() {
                "settings" => tray::show_settings(&app),
                "mode vietnamese" => mode::set(Mode::Vietnamese),
                "mode english" => mode::set(Mode::English),
                "mode toggle" => drop(mode::toggle()),
                "quit" => app.exit(0),
                _ => {}
            }
            tray::refresh_icon(&app);
        }
    }
}
