//! Tray icon (V/E), the tray popup window, and the settings window.
use std::fs::OpenOptions;
use std::io::Write;
use parking_lot::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::window::{Effect, EffectsBuilder};
use tauri::{App, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WebviewWindowBuilder, Window, WindowEvent};

use evkey_engine::settings::{self, Settings};

use crate::keyboard::mode::{self, Mode};

const TRAY_ID: &str = "evkey";
const POPUP: &str = "popup";
const ICON_V: &[u8] = include_bytes!("../icons/tray-v.png");
const ICON_E: &[u8] = include_bytes!("../icons/tray-e.png");
const MODE_POLL: Duration = Duration::from_millis(400);
/// Clicking the tray while the popup is open blurs (hides) it first; the click that follows must not reopen it.
const BLUR_GRACE: Duration = Duration::from_millis(300);

/// Tray icon rect in physical pixels, kept so the popup can be re-anchored after it resizes.
static ANCHOR: Mutex<Option<(PhysicalPosition<f64>, PhysicalSize<f64>)>> = Mutex::new(None);
static LAST_BLUR_HIDE: Mutex<Option<Instant>> = Mutex::new(None);

pub fn setup(app: &mut App) -> tauri::Result<()> {
    if let Some(popup) = app.get_webview_window(POPUP) {
        let _ = popup.set_effects(EffectsBuilder::new().effect(Effect::Acrylic).build());
    }
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_E)?)
        .tooltip("EVKey")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left | MouseButton::Right,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let scale = tray.app_handle().get_webview_window(POPUP).and_then(|w| w.scale_factor().ok()).unwrap_or(1.0);
                let position = rect.position.to_physical::<f64>(scale);
                let size = rect.size.to_physical::<f64>(scale);
                *ANCHOR.lock() = Some((position, size));
                toggle_popup(tray.app_handle());
            }
        })
        .build(app)?;
    refresh_icon(app.handle());
    let watch = app.handle().clone();
    std::thread::spawn(move || watch_mode(watch));
    Ok(())
}

/// Applies `f` to `config.toml` and writes it back (macros are kept). The running engine picks it up within a second.
pub(crate) fn update_settings(f: impl FnOnce(&mut Settings)) {
    let Some(dir) = settings::default_dir() else { return };
    if let Ok(mut loaded) = settings::load(&dir) {
        f(&mut loaded.settings);
        let _ = settings::save(&dir, &loaded);
    }
}

/// Windows with `"create": false` in tauri.conf.json: built on first show, destroyed on close so no WebView2 renderer idles in the background.
const LAZY: [&str; 2] = ["settings", crate::updater::UPDATE_WINDOW];

/// Returns the window `label`, building it from its config entry if needed; `init` runs in the page before its own scripts.
pub fn lazy_window(app: &AppHandle, label: &str, init: &str) -> Option<WebviewWindow> {
    if let Some(win) = app.get_webview_window(label) {
        return Some(win);
    }
    let cfg = app.config().app.windows.iter().find(|w| w.label == label)?;
    WebviewWindowBuilder::from_config(app, cfg).ok()?.initialization_script(init).build().ok()
}

pub fn show_settings(app: &AppHandle) {
    open_settings(app, None);
}

fn open_settings(app: &AppHandle, tab: Option<&str>) {
    let existed = app.get_webview_window("settings").is_some();
    // A freshly built page has no listener yet, so it reads the tab from this global instead of the event.
    let init = tab.map(|t| format!("window.__evkeyTab = {t:?};")).unwrap_or_default();
    let Some(win) = lazy_window(app, "settings", &init) else {
        return;
    };
    let _ = win.show();
    let _ = win.unminimize();
    let _ = win.set_focus();
    if existed {
        let _ = app.emit("settings-show", ());
        if let Some(tab) = tab {
            let _ = app.emit("settings-tab", tab);
        }
    }
}

/// Opens the settings window on `tab` (a `data-tab` name such as `macro`).
pub fn show_settings_tab(app: &AppHandle, tab: &str) {
    open_settings(app, Some(tab));
}

/// EVKEY_DEBUG_SHOW=popup: opens the popup near the top-right of the primary monitor (for screenshots).
pub fn debug_show_popup(app: &AppHandle) {
    if let Ok(Some(m)) = app.primary_monitor() {
        let w = f64::from(m.size().width);
        *ANCHOR.lock() = Some((PhysicalPosition::new(w - 400.0, 0.0), PhysicalSize::new(24.0, 48.0)));
    }
    toggle_popup(app);
}

fn toggle_popup(app: &AppHandle) {
    let Some(win) = app.get_webview_window(POPUP) else {
        return;
    };
    if win.is_visible().unwrap_or(false) {
        hide_popup(app);
        return;
    }
    let just_blurred = LAST_BLUR_HIDE.lock().is_some_and(|t| t.elapsed() < BLUR_GRACE);
    if just_blurred {
        return;
    }
    place_popup(app);
    let _ = win.show();
    let _ = win.set_focus();
    let _ = app.emit("popup-show", ());
}

pub fn hide_popup(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(POPUP) {
        let _ = win.hide();
    }
}

/// Puts the popup above the tray icon (taskbar at the bottom), clamped to the monitor the icon is on.
pub fn place_popup(app: &AppHandle) {
    let (Some(win), Some((pos, size))) = (app.get_webview_window(POPUP), *ANCHOR.lock()) else {
        return;
    };
    let Ok(popup) = win.outer_size() else {
        return;
    };
    let (w, h) = (f64::from(popup.width), f64::from(popup.height));
    let cx = pos.x + size.width / 2.0;
    let mut x = cx - w / 2.0;
    let mut y = pos.y - h;
    if let Ok(Some(monitor)) = app.monitor_from_point(cx, pos.y) {
        let area = monitor.work_area();
        let (left, top) = (f64::from(area.position.x), f64::from(area.position.y));
        let (right, bottom) = (left + f64::from(area.size.width), top + f64::from(area.size.height));
        x = x.clamp(left, (right - w).max(left));
        y = y.clamp(top, (bottom - h).max(top));
    }
    let _ = win.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
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
    debug_dump(mode, &tip);
}

fn tooltip(mode: Mode) -> String {
    match mode {
        Mode::Vietnamese => crate::i18n::tr("EVKey - Tiếng Việt", "EVKey - Vietnamese").into(),
        Mode::English => {
            let hotkey = settings::default_dir()
                .and_then(|dir| settings::load(&dir).ok())
                .map_or_else(|| Settings::default().toggle_hotkey, |l| l.settings.toggle_hotkey);
            format!("EVKey - English (hotkey: {hotkey})")
        }
    }
}

/// Polls the shared mode, so changes from the IME, the CLI, another process or a per-app switch reach the icon and the open popup.
// ponytail: polling every 400 ms; ITfCompartmentEventSink would make it event-driven if the delay matters.
fn watch_mode(app: AppHandle) {
    let mut shown = None;
    loop {
        std::thread::sleep(MODE_POLL);
        if let Ok(mode) = mode::get() {
            if shown != Some(mode) {
                shown = Some(mode);
                crate::popup_cmd::changed(&app);
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
    match event {
        // Closing only hides: the tray app keeps running and the windows are reused. Lazy windows really close.
        WindowEvent::CloseRequested { api, .. } if !LAZY.contains(&window.label()) => {
            api.prevent_close();
            let _ = window.hide();
        }
        // The popup behaves like a menu: clicking anywhere else dismisses it.
        WindowEvent::Focused(false) if window.label() == POPUP && std::env::var_os("EVKEY_DEBUG_POPUP_STICKY").is_none() => {
            let _ = window.hide();
            *LAST_BLUR_HIDE.lock() = Some(Instant::now());
        }
        _ => {}
    }
}
