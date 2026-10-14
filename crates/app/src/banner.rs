//! Secure Input banner (Windows): a click-through overlay at the bottom of the screen that says EVKey is paused
//! while a password field has focus. `secure_input_win` calls `notify` from its poll thread.
use std::sync::atomic::{AtomicU32, Ordering};
use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LWA_ALPHA};
use std::sync::OnceLock;

use evkey_engine::settings::{self, Settings};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

const BANNER: &str = "banner";
/// Gap between the banner and the bottom edge of the work area, in logical pixels.
const BOTTOM_GAP: f64 = 32.0;

static APP: OnceLock<AppHandle> = OnceLock::new();
/// `Some(holder)` while the banner is up; `holder` is "" when the app is unknown.
static CURRENT: Mutex<Option<String>> = Mutex::new(None);
/// Bumped on every show/hide so a running fade stops when a newer one starts.
static FADE: AtomicU32 = AtomicU32::new(0);
/// Whether the acrylic effect is currently applied; re-applying it on every show is what made the banner stutter.
static ACRYLIC: Mutex<Option<bool>> = Mutex::new(None);
const FADE_STEPS: u32 = 14;
const FADE_STEP_MS: u64 = 14;

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
        // The acrylic fills the whole window rectangle; DWM rounds it (DWMWA_WINDOW_CORNER_PREFERENCE = 33, DWMWCP_ROUND = 2).
        if let Ok(hwnd) = win.hwnd() {
            let round: u32 = 2;
            let _ = unsafe { DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(33), std::ptr::from_ref(&round).cast(), 4) };
        }
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
        None => fade(app, false),
    }
}

/// Fades the whole native window (acrylic included, which CSS opacity cannot touch) in or out on a worker thread.
/// The window is already layered (click-through), so `SetLayeredWindowAttributes` applies.
fn set_alpha(app: &AppHandle, alpha: u8) {
    if let Some(win) = app.get_webview_window(BANNER) {
        if let Ok(hwnd) = win.hwnd() {
            let _ = unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA) };
        }
    }
}

fn fade(app: &AppHandle, up: bool) {
    let id = FADE.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    std::thread::spawn(move || {
        for i in 1..=FADE_STEPS {
            if FADE.load(Ordering::SeqCst) != id {
                return;
            }
            let t = f64::from(i) / f64::from(FADE_STEPS);
            let eased = 1.0 - (1.0 - t).powi(3);
            set_alpha(&app, (255.0 * if up { eased } else { 1.0 - eased }) as u8);
            std::thread::sleep(std::time::Duration::from_millis(FADE_STEP_MS));
        }
        if !up && FADE.load(Ordering::SeqCst) == id {
            if let Some(win) = app.get_webview_window(BANNER) {
                let _ = win.hide();
            }
        }
    });
}

fn show(app: &AppHandle, holder: String) {
    let Some(win) = app.get_webview_window(BANNER) else { return };
    let state = state_for(holder);
    // Acrylic gives the glass blur; the CSS background alpha is the tint on top of it.
    let acrylic = state.blur > 0;
    if *ACRYLIC.lock() != Some(acrylic) {
        let effects = acrylic.then(|| EffectsBuilder::new().effect(Effect::Acrylic).build());
        let _ = win.set_effects(effects);
        *ACRYLIC.lock() = Some(acrylic);
    }
    let _ = app.emit_to(BANNER, "banner-state", state);
    place(app);
    // Start transparent so the first painted frame is invisible, then fade up.
    set_alpha(app, 0);
    let _ = win.show();
    fade(app, true);
}

/// Viewport (page area) of the focused browser document, set by the password poll; `None` falls back to the window.
static ANCHOR: Mutex<Option<RECT>> = Mutex::new(None);

pub fn set_anchor(rect: Option<RECT>) {
    *ANCHOR.lock() = rect;
}

/// Bottom centre of the viewport when known, else of the monitor holding the foreground window.
fn place(app: &AppHandle) {
    let Some(win) = app.get_webview_window(BANNER) else { return };
    let Ok(size) = win.outer_size() else { return };
    let anchor = *ANCHOR.lock();
    let mut rect = anchor.unwrap_or_default();
    let found = anchor.is_some() || unsafe { GetWindowRect(GetForegroundWindow(), &mut rect) }.is_ok();
    let monitor = found
        .then(|| app.monitor_from_point(f64::from((rect.left + rect.right) / 2), f64::from((rect.top + rect.bottom) / 2)))
        .and_then(Result::ok)
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let area = monitor.work_area();
    let (area_left, area_top) = (area.position.x, area.position.y);
    let (area_right, area_bottom) = (area_left + area.size.width as i32, area_top + area.size.height as i32);
    // Viewport clamped to the work area; without one, the whole work area.
    let (left, right, bottom) = match anchor {
        Some(r) => (r.left.max(area_left), r.right.min(area_right), r.bottom.min(area_bottom)),
        None => (area_left, area_right, area_bottom),
    };
    let x = f64::from(left) + f64::from(right - left - size.width as i32) / 2.0;
    let y = f64::from(bottom) - f64::from(size.height) - BOTTOM_GAP * monitor.scale_factor();
    let _ = win.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}

/// The banner page reports the size of its content so the window hugs it instead of keeping a fixed width.
#[tauri::command]
pub fn banner_fit(app: AppHandle, width: f64, height: f64) {
    if let Some(win) = app.get_webview_window(BANNER) {
        let _ = win.set_size(LogicalSize::new(width.ceil(), height.ceil()));
        place(&app);
    }
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
