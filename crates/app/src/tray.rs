//! Tray icon (V/E) and the popup and settings windows.
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, PhysicalPosition, Window, WindowEvent};

use crate::keyboard;

const TRAY_ID: &str = "evkey";
const ICON_V: &[u8] = include_bytes!("../icons/tray-v.png");
const ICON_E: &[u8] = include_bytes!("../icons/tray-e.png");

/// Clicking the tray blurs the popup (hides it), then the click arrives and would reopen it.
/// Clicks within this window after a hide are ignored.
const REOPEN_GUARD: Duration = Duration::from_millis(300);
static POPUP_HIDDEN_AT: Mutex<Option<Instant>> = Mutex::new(None);

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Cài đặt…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát EVKey", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &quit])?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_E)?)
        .tooltip("EVKey")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                if !hidden_recently() {
                    show_popup(tray.app_handle(), Some(position));
                }
            }
        })
        .build(app)?;
    refresh_icon(app.handle());
    Ok(())
}

/// Shows the popup above the tray click point (`at`), or at its default place when `None`.
pub fn show_popup(app: &AppHandle, at: Option<PhysicalPosition<f64>>) {
    let Some(win) = app.get_webview_window("popup") else {
        return;
    };
    refresh_icon(app);
    if let (Some(at), Ok(size)) = (at, win.outer_size()) {
        let x = (at.x - f64::from(size.width) / 2.0).max(0.0);
        let y = (at.y - f64::from(size.height) - 8.0).max(0.0);
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
    let _ = win.show();
    let _ = win.set_focus();
    let _ = app.emit("popup-show", ());
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

/// Syncs the tray icon with the current V/E state.
pub fn refresh_icon(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let icon = if keyboard::is_enabled().unwrap_or(false) { ICON_V } else { ICON_E };
    if let Ok(img) = Image::from_bytes(icon) {
        let _ = tray.set_icon(Some(img));
    }
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(false) if window.label() == "popup" => {
            let _ = window.hide();
            if let Ok(mut hidden_at) = POPUP_HIDDEN_AT.lock() {
                *hidden_at = Some(Instant::now());
            }
        }
        // Closing only hides: the tray app keeps running and the windows are reused.
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
        }
        _ => {}
    }
}

fn hidden_recently() -> bool {
    POPUP_HIDDEN_AT
        .lock()
        .ok()
        .and_then(|t| *t)
        .is_some_and(|t| t.elapsed() < REOPEN_GUARD)
}
