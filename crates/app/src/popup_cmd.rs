//! Commands behind the tray popup window (ui/popup.html).
use evkey_engine::settings::{self, CodeTable, InputKind, Settings, UiLanguage};
use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalSize, Manager};

use crate::keyboard::mode::{self, Mode};
use crate::tray;

/// Popup width in logical pixels; its height follows the content (`popup_resize`).
const POPUP_WIDTH: f64 = 320.0;

#[derive(Serialize)]
pub struct PopupState {
    mode: Mode,
    input_type: InputKind,
    code_table: CodeTable,
    ui_language: UiLanguage,
}

fn snapshot() -> PopupState {
    let current = load_settings();
    PopupState {
        mode: mode::get().unwrap_or(Mode::English),
        input_type: current.input_type,
        code_table: current.code_table,
        ui_language: current.ui_language,
    }
}

fn load_settings() -> Settings {
    settings::default_dir().and_then(|d| settings::load(&d).ok()).map(|l| l.settings).unwrap_or_default()
}

/// Tells the popup to re-read its state and keeps the tray icon in sync.
pub(crate) fn changed(app: &AppHandle) {
    tray::refresh_icon(app);
    let _ = app.emit("popup-show", ());
}

#[tauri::command]
pub fn popup_state() -> PopupState {
    snapshot()
}

#[tauri::command]
pub fn popup_set_input(app: AppHandle, kind: InputKind) {
    tray::update_settings(|s| s.input_type = kind);
    changed(&app);
}

#[tauri::command]
pub fn popup_set_table(app: AppHandle, table: CodeTable) {
    tray::update_settings(|s| s.code_table = table);
    changed(&app);
}

#[tauri::command]
pub fn popup_toggle_mode(app: AppHandle) {
    if let Ok(next) = mode::toggle() {
        if load_settings().toggle_sound {
            crate::keyboard::play_toggle_sound(next == Mode::English);
        }
    }
    changed(&app);
}

/// Opens the settings window, optionally on a given tab, and closes the popup.
#[tauri::command]
pub async fn popup_open_settings(app: AppHandle, tab: Option<String>) {
    tray::hide_popup(&app);
    match tab {
        Some(tab) => tray::show_settings_tab(&app, &tab),
        None => tray::show_settings(&app),
    }
}

/// Hides the popup (Esc).
#[tauri::command]
pub fn popup_hide(app: AppHandle) {
    tray::hide_popup(&app);
}

/// Fits the popup height to its content and keeps it anchored under or above the tray icon.
#[tauri::command]
pub fn popup_resize(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.set_size(LogicalSize::new(POPUP_WIDTH, height.ceil()));
        tray::place_popup(&app);
    }
}

