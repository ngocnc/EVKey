//! EVKey 2.0 app: CLI flags, single instance, tray icon and settings window.
#![windows_subsystem = "windows"]

mod actions;
mod ai;
mod autostart;
mod banner;
mod clipboard;
mod foreground;
mod hook;
mod i18n;
mod ipc;
mod keyboard;
mod popup_cmd;
mod secret;
mod secure_input_win;
mod settings_cmd;
mod tray;
mod updater;

use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map_or("", String::as_str) {
        "--version" => {
            attach_console();
            println!("{}", env!("CARGO_PKG_VERSION"));
        }
        "--autostart" => exit_with(match args.get(1).map(String::as_str) {
            Some("on") => autostart::set(true),
            Some("off") => autostart::set(false),
            _ => Err(i18n::tr("cách dùng: evkey --autostart on|off", "usage: evkey --autostart on|off").into()),
        }),
        "--mode" => {
            attach_console();
            // The mode lives inside the running app, so changes go through its pipe.
            exit_with(match args.get(1).map(String::as_str) {
                Some(a @ ("vietnamese" | "english" | "toggle")) => ipc::send_to_running_instance(&format!("mode {a}")),
                _ => Err(i18n::tr("cách dùng: evkey --mode vietnamese|english|toggle", "usage: evkey --mode vietnamese|english|toggle").into()),
            })
        }
        "" | "--hidden" => run(false),
        "--settings" => run(true),
        other => exit_with(Err(format!("{}{other}", i18n::tr("tham số không hợp lệ: ", "invalid argument: ")))),
    }
}

fn run(show_settings: bool) {
    if !ipc::acquire_single_instance() {
        exit_with(ipc::send_to_running_instance("settings"));
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            settings_cmd::get_state,
            settings_cmd::save_settings,
            settings_cmd::save_macros,
            settings_cmd::reset_config,
            settings_cmd::import_macros,
            settings_cmd::export_config,
            settings_cmd::import_config,
            settings_cmd::set_autostart,
            settings_cmd::open_config_dir,
            settings_cmd::open_github,
            settings_cmd::last_app,
            settings_cmd::show_settings,
            settings_cmd::quit,
            actions::actions_state,
            actions::actions_close,
            actions::actions_keep,
            actions::actions_fit,
            actions::actions_run,
            updater::check_update_now,
            updater::update_pending,
            updater::update_skip,
            updater::update_later,
            updater::update_install,
            popup_cmd::popup_state,
            popup_cmd::popup_set_input,
            popup_cmd::popup_set_table,
            popup_cmd::popup_toggle_mode,
            popup_cmd::popup_open_settings,
            popup_cmd::popup_hide,
            popup_cmd::popup_resize,
            banner::banner_state,
            banner::banner_defaults,
            banner::banner_fit,
            ai::gemini_key_status,
            ai::gemini_key_set,
            ai::gemini_key_delete,
            ai::ai_run,
        ])
        .on_window_event(tray::on_window_event)
        .setup(move |app| {
            keyboard::remove_legacy_keyboard();
            let _ = hook::start();
            tray::setup(app)?;
            banner::init(app.handle());
            updater::start(app.handle().clone());
            actions::init(app.handle());
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
