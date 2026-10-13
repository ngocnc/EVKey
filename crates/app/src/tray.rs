//! Tray icon (V/E), its menu, and the settings window.
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, Window, WindowEvent, Wry};

use evkey_engine::settings::{self, CodeTable, InputKind, Settings};

use crate::keyboard::mode::{self, Mode};

const TRAY_ID: &str = "evkey";
const ICON_V: &[u8] = include_bytes!("../icons/tray-v.png");
const ICON_E: &[u8] = include_bytes!("../icons/tray-e.png");
const MODE_POLL: Duration = Duration::from_millis(400);

struct MenuItems {
    vi: CheckMenuItem<Wry>,
    input: Vec<(InputKind, CheckMenuItem<Wry>)>,
    table: Vec<(CodeTable, CheckMenuItem<Wry>)>,
}

static MENU: OnceLock<MenuItems> = OnceLock::new();

const INPUT_KINDS: [(InputKind, &str); 4] = [
    (InputKind::Telex, "Telex"),
    (InputKind::Vni, "VNI"),
    (InputKind::SimpleTelex1, "Simple Telex 1"),
    (InputKind::SimpleTelex2, "Simple Telex 2"),
];

const CODE_TABLES: [(CodeTable, &str); 4] = [
    (CodeTable::Unicode, "Unicode"),
    (CodeTable::Tcvn3, "TCVN3 (ABC)"),
    (CodeTable::VniWindows, "VNI Windows"),
    (CodeTable::UnicodeCompound, "Unicode tổ hợp"),
];

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let vi = CheckMenuItem::with_id(app, "mode", "Tiếng Việt", true, true, None::<&str>)?;
    let input = INPUT_KINDS
        .iter()
        .enumerate()
        .map(|(i, (k, label))| Ok((*k, CheckMenuItem::with_id(app, format!("input:{i}"), *label, true, false, None::<&str>)?)))
        .collect::<tauri::Result<Vec<_>>>()?;
    let table = CODE_TABLES
        .iter()
        .enumerate()
        .map(|(i, (t, label))| Ok((*t, CheckMenuItem::with_id(app, format!("table:{i}"), *label, true, false, None::<&str>)?)))
        .collect::<tauri::Result<Vec<_>>>()?;
    let input_refs: Vec<&dyn IsMenuItem<Wry>> = input.iter().map(|(_, i)| i as &dyn IsMenuItem<Wry>).collect();
    let table_refs: Vec<&dyn IsMenuItem<Wry>> = table.iter().map(|(_, i)| i as &dyn IsMenuItem<Wry>).collect();
    let input_menu = Submenu::with_items(app, "Kiểu gõ", true, &input_refs)?;
    let table_menu = Submenu::with_items(app, "Bảng mã", true, &table_refs)?;
    let settings = MenuItem::with_id(app, "settings", "Cài đặt…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát EVKey", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&vi, &input_menu, &table_menu, &sep, &settings, &quit])?;
    let _ = MENU.set(MenuItems { vi, input, table });
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_E)?)
        .tooltip("EVKey")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id.as_ref();
            let index = |prefix: &str| id.strip_prefix(prefix).and_then(|n| n.parse::<usize>().ok());
            if id == "settings" {
                show_settings(app);
            } else if id == "quit" {
                app.exit(0);
            } else if id == "mode" {
                let _ = mode::toggle();
            } else if let Some((kind, _)) = index("input:").and_then(|i| INPUT_KINDS.get(i)) {
                update_settings(|s| s.input_type = *kind);
            } else if let Some((table, _)) = index("table:").and_then(|i| CODE_TABLES.get(i)) {
                update_settings(|s| s.code_table = *table);
            }
            refresh_icon(app);
        })
        .on_tray_icon_event(|tray, event| {
            // Left click flips V/E; right click opens the menu (handled by the shell).
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                if mode::toggle().is_ok() {
                    refresh_icon(tray.app_handle());
                }
            }
        })
        .build(app)?;
    refresh_icon(app.handle());
    let watch = app.handle().clone();
    std::thread::spawn(move || watch_mode(watch));
    Ok(())
}

/// Applies `f` to `config.toml` and writes it back (macros are kept). The running engine picks it up within a second.
fn update_settings(f: impl FnOnce(&mut Settings)) {
    let Some(dir) = settings::default_dir() else { return };
    if let Ok(mut loaded) = settings::load(&dir) {
        f(&mut loaded.settings);
        let _ = settings::save(&dir, &loaded);
    }
}

/// Ticks the menu entries that match the current mode and settings.
fn sync_menu(mode: Mode) {
    let Some(items) = MENU.get() else { return };
    let current = settings::default_dir().and_then(|d| settings::load(&d).ok()).map(|l| l.settings).unwrap_or_default();
    let _ = items.vi.set_checked(mode == Mode::Vietnamese);
    for (kind, item) in &items.input {
        let _ = item.set_checked(*kind == current.input_type);
    }
    for (table, item) in &items.table {
        let _ = item.set_checked(*table == current.code_table);
    }
}

pub fn show_settings(app: &AppHandle) {
    let Some(win) = app.get_webview_window("settings") else {
        return;
    };
    let _ = win.show();
    let _ = win.unminimize();
    let _ = win.set_focus();
    let _ = app.emit("settings-show", ());
}

/// Syncs the tray icon and tooltip with the shared V/E mode.
pub fn refresh_icon(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let Ok(mode) = mode::get() else {
        return;
    };
    let icon = match mode {
        Mode::Vietnamese => ICON_V,
        Mode::English => ICON_E,
    };
    if let Ok(img) = Image::from_bytes(icon) {
        let _ = tray.set_icon(Some(img));
    }
    let tip = tooltip(mode);
    let _ = tray.set_tooltip(Some(tip.as_str()));
    sync_menu(mode);
    debug_dump(mode, &tip);
}

fn tooltip(mode: Mode) -> String {
    match mode {
        Mode::Vietnamese => "EVKey - Tieng Viet".into(),
        Mode::English => {
            let hotkey = settings::default_dir()
                .and_then(|dir| settings::load(&dir).ok())
                .map_or_else(|| Settings::default().toggle_hotkey, |l| l.settings.toggle_hotkey);
            format!("EVKey - English (hotkey: {hotkey})")
        }
    }
}

/// Polls the shared mode, so changes from the IME, the CLI or another process reach the icon.
// ponytail: polling every 400 ms; ITfCompartmentEventSink would make it event-driven if the delay matters.
fn watch_mode(app: AppHandle) {
    let mut shown = None;
    loop {
        std::thread::sleep(MODE_POLL);
        if let Ok(mode) = mode::get() {
            if shown != Some(mode) {
                shown = Some(mode);
                refresh_icon(&app);
            }
        }
    }
}

/// Appends each refresh to the file named by `EVKEY_DEBUG_TRAY`, so the icon state can be checked from outside.
fn debug_dump(mode: Mode, tip: &str) {
    let Some(path) = std::env::var_os("EVKEY_DEBUG_TRAY") else {
        return;
    };
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let line = format!("{ms} mode={} tooltip={tip}\n", mode.as_str());
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    // Closing only hides: the tray app keeps running and the settings window is reused.
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
    }
}
