//! Single instance plus a tiny command channel: a second launch (or `--mode`) talks to the running app.
//! Commands: `settings`, `mode vietnamese|english|toggle`.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::time::Duration;

use tauri::AppHandle;
use windows::core::w;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_INBOUND;
use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT};
use windows::Win32::System::Threading::CreateMutexW;

use crate::keyboard::mode::{self, Mode};
use crate::tray;

/// Named pipe a second launch uses to ask the running instance to show settings.
const PIPE_PATH: &str = r"\\.\pipe\EVKey-Show";

/// True when this is the first instance. The mutex handle stays open until the process exits.
pub fn acquire_single_instance() -> bool {
    unsafe {
        let _ = CreateMutexW(None, true, w!("Local\\EVKeySingleInstance"));
        GetLastError() != ERROR_ALREADY_EXISTS
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
    loop {
        let pipe = unsafe {
            CreateNamedPipeW(
                w!("\\\\.\\pipe\\EVKey-Show"),
                PIPE_ACCESS_INBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                1,
                0,
                64,
                0,
                None,
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
                "mode vietnamese" => drop(mode::set(Mode::Vietnamese)),
                "mode english" => drop(mode::set(Mode::English)),
                "mode toggle" => drop(mode::toggle()),
                _ => {}
            }
            tray::refresh_icon(&app);
        }
    }
}
