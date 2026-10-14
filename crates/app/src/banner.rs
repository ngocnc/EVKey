//! Secure Input banner (Windows): a click-through overlay at the bottom of the screen that says EVKey is paused
//! while a password field has focus. `secure_input_win` calls `notify` from its poll thread.
use std::sync::OnceLock;

use evkey_engine::settings::{self, Settings};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

const BANNER: &str = "banner";
/// Gap between the banner and the bottom edge of the work area, in logical pixels.
const BOTTOM_GAP: f64 = 32.0;

static APP: OnceLock<AppHandle> = OnceLock::new();
/// `Some(holder)` while the banner is up; `holder` is "" when the app is unknown.
static CURRENT: Mutex<Option<String>> = Mutex::new(None);

/// What `banner-view.js` needs to draw the banner.
#[derive(Serialize, Clone)]
pub struct BannerState {
    app: String,
    opacity: u8,
    blur: u8,
    bg: String,
    border: String,
    text: String,
}

fn current_settings() -> Settings {
    settings::default_dir().and_then(|d| settings::load(&d).ok()).map(|l| l.settings).unwrap_or_default()
}

fn state_for(holder: String) -> BannerState {
    let s = current_settings();
    BannerState { app: holder, opacity: s.banner_opacity, blur: s.banner_blur, bg: s.banner_bg, border: s.banner_border, text: s.banner_text }
}

/// Makes the window click-through and unfocusable, and keeps the handle for `notify`.
pub fn init(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(BANNER) {
        let _ = win.set_ignore_cursor_events(true);
        let _ = win.set_focusable(false);
    }
    let _ = APP.set(app.clone());
    // EVKEY_DEBUG_BANNER=<app name> shows the banner at startup (for screenshots).
    if let Some(name) = std::env::var_os("EVKEY_DEBUG_BANNER") {
        notify(Some(name.to_string_lossy().into_owned()));
    }
}

/// `Some(app)` shows the banner for that app ("" = unknown), `None` hides it. Cheap when nothing changed; does
/// nothing before `init`.
pub fn notify(state: Option<String>) {
    let Some(app) = APP.get() else { return };
    {
        let mut current = CURRENT.lock();
        if *current == state {
            return;
        }
        current.clone_from(&state);
    }
    match state {
        Some(holder) => show(app, holder),
        None => {
            if let Some(win) = app.get_webview_window(BANNER) {
                let _ = win.hide();
            }
        }
    }
}

fn show(app: &AppHandle, holder: String) {
    let Some(win) = app.get_webview_window(BANNER) else { return };
    let state = state_for(holder);
    // Acrylic gives the glass blur; the CSS background alpha is the tint on top of it.
    let effects = (state.blur > 0).then(|| EffectsBuilder::new().effect(Effect::Acrylic).build());
    let _ = win.set_effects(effects);
    let _ = app.emit_to(BANNER, "banner-state", state);
    place(app);
    let _ = win.show();
}

/// Bottom centre of the monitor that holds the foreground window (primary monitor when unknown).
fn place(app: &AppHandle) {
    let Some(win) = app.get_webview_window(BANNER) else { return };
    let Ok(size) = win.outer_size() else { return };
    let mut rect = RECT::default();
    let found = unsafe { GetWindowRect(GetForegroundWindow(), &mut rect) }.is_ok();
    let monitor = found
        .then(|| app.monitor_from_point(f64::from((rect.left + rect.right) / 2), f64::from((rect.top + rect.bottom) / 2)))
        .and_then(Result::ok)
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let area = monitor.work_area();
    let x = f64::from(area.position.x) + (f64::from(area.size.width) - f64::from(size.width)) / 2.0;
    let bottom = f64::from(area.position.y) + f64::from(area.size.height);
    let y = bottom - f64::from(size.height) - BOTTOM_GAP * monitor.scale_factor();
    let _ = win.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}

/// The banner page asks for the current state when it loads, in case it missed the event.
#[tauri::command]
pub fn banner_state() -> BannerState {
    state_for(CURRENT.lock().clone().unwrap_or_default())
}

/// Factory look for the "Đặt lại mặc định" button, so the UI never hardcodes the defaults.
#[tauri::command]
pub fn banner_defaults() -> BannerState {
    let s = Settings::default();
    BannerState { app: String::new(), opacity: s.banner_opacity, blur: s.banner_blur, bg: s.banner_bg, border: s.banner_border, text: s.banner_text }
}
