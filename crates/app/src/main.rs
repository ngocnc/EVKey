//! EVKey 2.0 app: CLI flags, single instance, tray icon and settings window.
#![windows_subsystem = "windows"]

mod autostart;
mod banner;
mod foreground;
mod hook;
mod ipc;
mod keyboard;
mod popup_cmd;
mod secure_input_win;
mod settings_cmd;
mod tray;

use evkey_engine::settings::{self, Backend};
use keyboard::mode::{self, Mode};
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

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
                    Some(a @ ("vietnamese" | "english" | "toggle")) => ipc::send_to_running_instance(&format!("mode {a}")),
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
    if !ipc::acquire_single_instance() {
        exit_with(ipc::send_to_running_instance("settings"));
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
            popup_cmd::popup_state,
            popup_cmd::popup_set_input,
            popup_cmd::popup_set_table,
            popup_cmd::popup_toggle_mode,
            popup_cmd::popup_open_settings,
            popup_cmd::popup_hide,
            popup_cmd::popup_resize,
            banner::banner_state,
            banner::banner_defaults,
        ])
        .on_window_event(tray::on_window_event)
        .setup(move |app| {
            if configured_backend() == Backend::Hook {
                start_hook();
            }
            tray::setup(app)?;
            banner::init(app.handle());
            foreground::start();
            autostart::repoint_if_enabled();
            let serve = app.handle().clone();
            std::thread::spawn(move || ipc::serve_show_requests(serve));
            // EVKEY_DEBUG_SHOW=settings opens the settings window at startup (for screenshots).
            let debug = std::env::var("EVKEY_DEBUG_SHOW").unwrap_or_default();
            if show_settings || debug == "settings" {
                tray::show_settings(app.handle());
            }
            if debug == "popup" {
                tray::debug_show_popup(app.handle());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("EVKey không chạy được");
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
