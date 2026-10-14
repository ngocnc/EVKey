//! Translate the selected text: the hotkey copies the selection (Ctrl+C) and Google Translate's public endpoint
//! translates it Vietnamese <-> other (auto-detected). A small popup that never takes focus shows only the result for
//! a few seconds; the `translate_action` setting can also copy it or paste it over the selection (Ctrl+V).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use evkey_engine::settings::{self, Settings, TranslateAction, UiLanguage};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition};
use tauri::window::{Effect, EffectsBuilder};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

use crate::clipboard;
use crate::i18n::tr;

pub const TRANSLATE_WINDOW: &str = "translate";
/// Popup width in logical pixels; the height follows the text (`translate_fit`).
const WIDTH: f64 = 340.0;
const ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";
/// The endpoint takes the text in the URL; stay well under typical URL limits.
const MAX_CHARS: usize = 4000;
const VK_CONTROL: u16 = 0x11;
const VK_C: u16 = 0x43;
const VK_V: u16 = 0x56;
/// Modifier virtual keys: Shift, Ctrl, Alt, Left Win, Right Win.
const MODIFIERS: [i32; 5] = [0x10, 0x11, 0x12, 0x5B, 0x5C];

static APP: OnceLock<AppHandle> = OnceLock::new();
static BUSY: AtomicBool = AtomicBool::new(false);
static VIEW: Mutex<View> = Mutex::new(View::new());
/// Pointer position (physical pixels) when the hotkey fired.
static ANCHOR: Mutex<(f64, f64)> = Mutex::new((0.0, 0.0));
/// Bumped on every show or close, so a hide timer only fires for the popup it was started for.
static SHOWN: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Clone)]
pub struct View {
    /// `loading`, `ok` or `error`.
    status: &'static str,
    /// The translation, or the error message.
    text: String,
    /// Milliseconds before the popup hides, 0 = stays until clicked; drives the progress bar.
    ms: u32,
    ui_language: UiLanguage,
}

impl View {
    const fn new() -> View {
        View { status: "loading", text: String::new(), ms: 5000, ui_language: UiLanguage::Vi }
    }
}

pub fn init(app: &AppHandle) {
    let _ = APP.set(app.clone());
    if let Some(win) = app.get_webview_window(TRANSLATE_WINDOW) {
        let _ = win.set_effects(EffectsBuilder::new().effect(Effect::Acrylic).build());
        // Never takes focus, so a Replace paste goes to the app the text was selected in.
        let _ = win.set_focusable(false);
        if let Ok(hwnd) = win.hwnd() {
            // Rounded corners (DWMWA_WINDOW_CORNER_PREFERENCE = 33, DWMWCP_ROUND = 2), like the banner.
            let round: u32 = 2;
            let _ = unsafe { DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(33), std::ptr::from_ref(&round).cast(), 4) };
        }
    }
}

/// Called from the hook thread when the translate hotkey fires; the work runs on its own thread.
pub fn trigger() {
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        run();
        BUSY.store(false, Ordering::SeqCst);
    });
}

fn load_settings() -> Settings {
    settings::default_dir().and_then(|d| settings::load(&d).ok()).map(|l| l.settings).unwrap_or_default()
}

fn modifiers_down() -> bool {
    MODIFIERS.iter().any(|&vk| unsafe { GetAsyncKeyState(vk) } < 0)
}

/// Waits for the hotkey's modifiers to be released, else Ctrl+C would arrive as Ctrl+Alt+C.
fn wait_modifiers_up() {
    let deadline = Instant::now() + Duration::from_secs(1);
    while modifiers_down() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(15));
    }
}

/// Copies the current selection and puts the previous clipboard text back. `None` when nothing was copied.
// ponytail: only text is restored; a non-text clipboard (image, files) is lost. Per-format backup if that matters.
fn capture_selection() -> Option<String> {
    let previous = clipboard::get_text();
    let seq = clipboard::sequence();
    crate::hook::send_combo(VK_CONTROL, VK_C);
    let deadline = Instant::now() + Duration::from_millis(600);
    while clipboard::sequence() == seq {
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    let selected = clipboard::get_text();
    if let Some(previous) = previous {
        clipboard::set_text(&previous);
    }
    selected.filter(|s| !s.trim().is_empty())
}

fn run() {
    let Some(app) = APP.get() else { return };
    // Never read a password field.
    if crate::secure_input_win::active() || crate::secure_input_win::classic_password_focused() {
        return;
    }
    let cfg = load_settings();
    let view = |status, text: String| View { status, text, ms: cfg.translate_popup_ms, ui_language: cfg.ui_language };
    if let Ok(cursor) = app.cursor_position() {
        *ANCHOR.lock() = (cursor.x, cursor.y);
    }
    wait_modifiers_up();
    let Some(source) = capture_selection() else {
        *VIEW.lock() = view("error", tr("Bạn chưa bôi đen văn bản nào.", "No text is selected.").into());
        show(app, true);
        return;
    };
    if source.chars().count() > MAX_CHARS {
        *VIEW.lock() = view("error", tr("Văn bản quá dài để dịch.", "The text is too long to translate.").into());
        show(app, true);
        return;
    }
    *VIEW.lock() = view("loading", String::new());
    show(app, false);
    match translate_auto(&source) {
        Ok(result) => {
            *VIEW.lock() = view("ok", result.clone());
            show(app, true);
            apply(cfg.translate_action, &result);
        }
        Err(e) => {
            *VIEW.lock() = view("error", e);
            show(app, true);
        }
    }
}

/// Puts the popup next to the pointer position saved when the hotkey fired, clamped to the monitor it is on.
fn place(app: &AppHandle) {
    let Some(win) = app.get_webview_window(TRANSLATE_WINDOW) else { return };
    let Ok(size) = win.outer_size() else { return };
    let (cx, cy) = *ANCHOR.lock();
    let (w, h) = (f64::from(size.width), f64::from(size.height));
    let (mut x, mut y) = (cx + 12.0, cy + 16.0);
    if let Ok(Some(monitor)) = app.monitor_from_point(cx, cy) {
        let area = monitor.work_area();
        let (left, top) = (f64::from(area.position.x), f64::from(area.position.y));
        let (right, bottom) = (left + f64::from(area.size.width), top + f64::from(area.size.height));
        x = x.clamp(left, (right - w).max(left));
        y = if y + h > bottom { (cy - h - 16.0).max(top) } else { y };
    }
    let _ = win.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}

/// Shows the popup without taking focus. A `timed` popup (a result or an error) hides itself after the configured seconds.
fn show(app: &AppHandle, timed: bool) {
    let Some(win) = app.get_webview_window(TRANSLATE_WINDOW) else { return };
    let id = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
    place(app);
    let _ = app.emit("translate-update", ());
    let _ = win.show();
    let ms = u64::from(VIEW.lock().ms);
    if timed && ms > 0 {
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(ms));
            if SHOWN.load(Ordering::SeqCst) == id {
                hide(&app);
            }
        });
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).user_agent(concat!("EVKey/", env!("CARGO_PKG_VERSION"))).build()
}

/// Text and detected source language from a `translate_a/single` response.
fn parse_response(json: &serde_json::Value) -> Option<(String, String)> {
    let text: String = json.get(0)?.as_array()?.iter().filter_map(|seg| seg.get(0)?.as_str()).collect();
    let from = json.get(2)?.as_str()?.to_string();
    Some((text, from))
}

fn google(text: &str, to: &str) -> Result<(String, String), String> {
    let json: serde_json::Value = agent()
        .get(ENDPOINT)
        .query("client", "gtx")
        .query("sl", "auto")
        .query("tl", to)
        .query("dt", "t")
        .query("q", text)
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    parse_response(&json).ok_or_else(|| tr("Phản hồi dịch không hợp lệ.", "The translation response was not understood.").into())
}

/// Vietnamese text goes to English, everything else to Vietnamese.
fn translate_auto(text: &str) -> Result<String, String> {
    let (result, from) = google(text, "vi")?;
    if from.starts_with("vi") {
        return google(text, "en").map(|(english, _)| english);
    }
    Ok(result)
}

#[tauri::command]
pub fn translate_state() -> View {
    VIEW.lock().clone()
}

#[tauri::command]
pub fn translate_close(app: AppHandle) {
    SHOWN.fetch_add(1, Ordering::SeqCst);
    hide(&app);
}

/// Fits the popup height to its content and keeps it next to the pointer.
#[tauri::command]
pub fn translate_fit(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window(TRANSLATE_WINDOW) {
        let _ = win.set_size(LogicalSize::new(WIDTH, height.clamp(40.0, 360.0)));
        place(&app);
    }
}

fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(TRANSLATE_WINDOW) {
        let _ = win.hide();
    }
}

/// The extra step chosen in settings: copy the translation, or paste it over the selection in the app that still has focus.
fn apply(action: TranslateAction, result: &str) {
    match action {
        TranslateAction::Show => {}
        TranslateAction::Copy => {
            clipboard::set_text(result);
        }
        TranslateAction::Replace => {
            wait_modifiers_up();
            let previous = clipboard::get_text();
            if clipboard::set_text(result) {
                crate::hook::send_combo(VK_CONTROL, VK_V);
                std::thread::sleep(Duration::from_millis(300));
            }
            if let Some(previous) = previous {
                clipboard::set_text(&previous);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_joins_segments_and_reads_source() {
        let json = serde_json::json!([[["Xin chào ", "Hello ", null], ["thế giới", "world", null]], null, "en"]);
        assert_eq!(parse_response(&json), Some(("Xin chào thế giới".to_string(), "en".to_string())));
        assert_eq!(parse_response(&serde_json::json!([[]])), None);
    }
}
