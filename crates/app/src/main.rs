//! EVKey 2.0 app: CLI flags, single instance, tray icon and settings window.
#![windows_subsystem = "windows"]

mod autostart;
mod foreground;
mod hook;
mod keyboard;
mod settings_cmd;
mod tray;
mod typing;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::time::Duration;

use evkey_engine::settings::{self, Backend};
use keyboard::mode::{self, Mode};
use tauri::AppHandle;
use windows::core::w;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_INBOUND;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT};
use windows::Win32::System::Threading::CreateMutexW;

/// Named pipe a second launch uses to ask the running instance to show settings.
const PIPE_PATH: &str = r"\\.\pipe\EVKey-Show";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map_or("", String::as_str) {
        "--version" => {
            attach_console();
            println!("{}", env!("CARGO_PKG_VERSION"));
        }
        "--enable-keyboard" => exit_with(keyboard::set_enabled(true)),
        "--disable-keyboard" => exit_with(keyboard::set_enabled(false)),
        "--autostart" => exit_with(match args.get(1).map(String::as_str) {
            Some("on") => autostart::set(true),
            Some("off") => autostart::set(false),
            _ => Err("cách dùng: evkey --autostart on|off".into()),
        }),
        "--mode" => {
            attach_console();
            let arg = args.get(1).map(String::as_str);
            exit_with(if configured_backend() == Backend::Hook {
                // The mode lives inside the running app, so changes go through its pipe.
                match arg {
                    Some(a @ ("vietnamese" | "english" | "toggle")) => send_to_running_instance(&format!("mode {a}")),
                    _ => Err("backend hook: cách dùng evkey --mode vietnamese|english|toggle (không có get)".into()),
                }
            } else {
                match arg {
                    Some("get") => mode::get().map(|m| println!("{}", m.as_str())),
                    Some("vietnamese") => mode::set(Mode::Vietnamese),
                    Some("english") => mode::set(Mode::English),
                    Some("toggle") => mode::toggle().map(|_| ()),
                    _ => Err("cách dùng: evkey --mode get|vietnamese|english|toggle".into()),
                }
            })
        }
        "" | "--hidden" => run(false),
        "--settings" => run(true),
        other => exit_with(Err(format!("tham số không hợp lệ: {other}"))),
    }
}

fn run(show_settings: bool) {
    if !acquire_single_instance() {
        exit_with(send_to_running_instance("settings"));
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            settings_cmd::get_state,
            settings_cmd::set_enabled,
            settings_cmd::save_settings,
            settings_cmd::save_macros,
            settings_cmd::reset_config,
            settings_cmd::set_autostart,
            settings_cmd::open_config_dir,
            settings_cmd::open_github,
            settings_cmd::last_app,
            settings_cmd::show_settings,
            settings_cmd::quit,
            settings_cmd::set_backdrop,
        ])
        .on_window_event(tray::on_window_event)
        .setup(move |app| {
            if configured_backend() == Backend::Hook {
                start_hook();
            }
            tray::setup(app)?;
            foreground::start();
            autostart::repoint_if_enabled();
            let serve = app.handle().clone();
            std::thread::spawn(move || serve_show_requests(serve));
            // EVKEY_DEBUG_SHOW=settings opens the settings window at startup (for screenshots).
            let debug = std::env::var("EVKEY_DEBUG_SHOW").unwrap_or_default();
            if show_settings || debug == "settings" {
                tray::show_settings(app.handle());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("EVKey không chạy được");
}

/// True when this is the first instance. The mutex handle stays open until the process exits.
fn acquire_single_instance() -> bool {
    unsafe {
        let _ = CreateMutexW(None, true, w!("Local\\EVKeySingleInstance"));
        GetLastError() != ERROR_ALREADY_EXISTS
    }
}

/// Sends `command` to the running instance. Retries briefly in case it is still starting its pipe.
fn send_to_running_instance(command: &str) -> Result<(), String> {
    for _ in 0..30 {
        if let Ok(mut pipe) = OpenOptions::new().write(true).open(PIPE_PATH) {
            return pipe.write_all(command.as_bytes()).map_err(|e| e.to_string());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("EVKey đang chạy nhưng không phản hồi".into())
}

/// Hook backend: take over typing and drop the TSF keyboard entry, which would otherwise process every key twice.
fn start_hook() {
    if keyboard::is_enabled().unwrap_or(false) {
        let _ = keyboard::set_enabled(false);
    }
    let _ = hook::start();
}

fn configured_backend() -> Backend {
    settings::default_dir()
        .and_then(|d| settings::load(&d).ok())
        .map(|l| l.settings.backend)
        .unwrap_or_default()
}

/// Serves "show settings" requests from second launches. Runs forever on its own thread.
fn serve_show_requests(app: AppHandle) {
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

fn attach_console() {
    // The exe is a GUI subsystem binary; attach to the parent console so CLI output is visible.
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn exit_with(result: Result<(), String>) -> ! {
    match result {
        Ok(()) => std::process::exit(0),
        Err(msg) => {
            attach_console();
            eprintln!("evkey: {msg}");
            std::process::exit(1);
        }
    }
}
