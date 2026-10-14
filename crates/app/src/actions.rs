//! Actions on the selected text, in one small panel that never takes focus. The translate hotkey translates at once;
//! the action hotkey opens a menu (Dịch thuật, Thêm dấu, Sửa/viết lại, Chuyển mã). The selection is copied with Ctrl+C
//! and the previous clipboard text is put back.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use evkey_engine::convert;
use evkey_engine::settings::{self, Settings, TranslateAction, TranslateProvider, UiLanguage};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

use crate::ai;
use crate::clipboard;
use crate::i18n::tr;

pub const ACTIONS_WINDOW: &str = "actions";
/// Panel width in logical pixels; the height follows the content (`actions_fit`).
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
/// The text the panel acts on: the selection captured when the hotkey fired.
static SOURCE: Mutex<String> = Mutex::new(String::new());
/// Pointer position (physical pixels) when the hotkey fired.
static ANCHOR: Mutex<(f64, f64)> = Mutex::new((0.0, 0.0));
/// Bumped on every show or close, so a hide timer only fires for the panel it was started for.
static SHOWN: AtomicU64 = AtomicU64::new(0);
/// True while the action menu waits for a choice: it has no timer, so an outside click or Esc closes it.
static MENU_OPEN: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone)]
pub struct View {
    /// `menu` (choose an action), `loading`, `ok` or `error`.
    status: &'static str,
    /// The result, or the error message.
    text: String,
    /// Milliseconds before the panel hides, 0 = stays until clicked; drives the progress bar.
    ms: u32,
    ui_language: UiLanguage,
    /// A Gemini key is saved; without one the AI entries are disabled in the menu.
    gemini: bool,
    /// The result came from the action menu (an editable field): the result button replaces the selection instead of copying.
    replace: bool,
}

impl View {
    const fn new() -> View {
        View { status: "loading", text: String::new(), ms: 5000, ui_language: UiLanguage::Vi, gemini: false, replace: false }
    }
}

pub fn init(app: &AppHandle) {
    let _ = APP.set(app.clone());
    if let Some(win) = app.get_webview_window(ACTIONS_WINDOW) {
        let _ = win.set_effects(EffectsBuilder::new().effect(Effect::Acrylic).build());
        // Never takes focus, so a Replace paste goes to the app the text was selected in.
        let _ = win.set_focusable(false);
    }
}

/// Called from the hook thread when an action hotkey fires; the work runs on its own thread.
/// `translate_now` translates the selection at once (translate hotkey); otherwise the action menu opens.
pub fn trigger(translate_now: bool) {
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        open(translate_now);
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

fn open(translate_now: bool) {
    let Some(app) = APP.get() else { return };
    // Never read a password field.
    if crate::secure_input_win::active() || crate::secure_input_win::classic_password_focused() {
        return;
    }
    // The menu's actions rewrite the selection, so it only opens in an editable field; elsewhere the hotkey does nothing.
    if !translate_now && !crate::secure_input_win::editable_focused() {
        return;
    }
    let cfg = load_settings();
    if let Ok(cursor) = app.cursor_position() {
        *ANCHOR.lock() = (cursor.x, cursor.y);
    }
    wait_modifiers_up();
    let Some(source) = capture_selection() else {
        fail(app, &cfg, tr("Bạn chưa bôi đen văn bản nào.", "No text is selected.").into());
        return;
    };
    if source.chars().count() > MAX_CHARS {
        fail(app, &cfg, tr("Văn bản quá dài để xử lý.", "The text is too long to process.").into());
        return;
    }
    *SOURCE.lock() = source;
    if translate_now {
        perform(app, &cfg, "translate", false);
    } else {
        set_view(&cfg, "menu", String::new());
        show(app, false);
        MENU_OPEN.store(true, Ordering::SeqCst);
    }
}

/// Runs action `id` on the captured selection and shows the result. The `translate_action` setting applies to it.
/// `from_menu`: started from the action menu, so the result button replaces the selection.
fn perform(app: &AppHandle, cfg: &Settings, id: &str, from_menu: bool) {
    let source = SOURCE.lock().clone();
    // While loading, the view text is the action id, so the panel can say what it is doing.
    set_view(cfg, "loading", id.to_string());
    show(app, false);
    match compute(id, &source, cfg.translate_provider) {
        Ok(result) => {
            set_view(cfg, "ok", result.clone());
            VIEW.lock().replace = from_menu;
            show(app, true);
            // Pasting outside an editable field does nothing and the clipboard is put back, so copy instead.
            let action = match cfg.translate_action {
                TranslateAction::Replace if !from_menu && !crate::secure_input_win::editable_focused() => TranslateAction::Copy,
                a => a,
            };
            apply(action, &result);
        }
        Err(e) => fail(app, cfg, e),
    }
}

/// The text action `id` produces from `source`. Blocking: runs on the action thread.
/// Translate goes to Google unless Gemini is chosen; then it takes the AI path like the other actions.
fn compute(id: &str, source: &str, provider: TranslateProvider) -> Result<String, String> {
    match id {
        "translate" if provider == TranslateProvider::Google => translate_auto(source),
        "convert" => Ok(convert::to_unicode(source, convert::detect(source))),
        _ => {
            let action = ai::AiAction::from_id(id).ok_or_else(|| tr("Thao tác không tồn tại", "Unknown action").to_string())?;
            ai::run(&action, source)
        }
    }
}

fn set_view(cfg: &Settings, status: &'static str, text: String) {
    let gemini = status == "menu" && crate::secret::gemini_key().is_some();
    *VIEW.lock() = View { status, text, ms: cfg.translate_popup_ms, ui_language: cfg.ui_language, gemini, replace: false };
}

/// Shows `message` as an error in the panel.
fn fail(app: &AppHandle, cfg: &Settings, message: String) {
    set_view(cfg, "error", message);
    show(app, true);
}

/// Puts the panel next to the pointer position saved when the hotkey fired, clamped to the monitor it is on.
fn place(app: &AppHandle) {
    let Some(win) = app.get_webview_window(ACTIONS_WINDOW) else { return };
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

/// Shows the panel without taking focus. A `timed` panel (a menu, a result or an error) hides itself after the configured seconds.
fn show(app: &AppHandle, timed: bool) {
    let Some(win) = app.get_webview_window(ACTIONS_WINDOW) else { return };
    let id = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
    MENU_OPEN.store(false, Ordering::SeqCst);
    place(app);
    let _ = app.emit("actions-update", ());
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
        .map_err(|e| match e {
            // Never show ureq's message: it holds the URL, which holds the selected text.
            ureq::Error::Status(429, _) => tr("Google tạm chặn dịch, thử lại sau hoặc chuyển sang Gemini", "Google is limiting requests, try again later or switch to Gemini").to_string(),
            ureq::Error::Status(code, _) => format!("{}: HTTP {code}", tr("Lỗi Google Dịch", "Google Translate error")),
            ureq::Error::Transport(_) => tr("Không kết nối được Google Dịch, kiểm tra mạng", "Could not reach Google Translate, check your network").to_string(),
        })?
        .into_json()
        .map_err(|_| tr("Phản hồi dịch không hợp lệ.", "The translation response was not understood.").to_string())?;
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

/// The menu button `action` (an action id) was clicked.
#[tauri::command]
pub fn actions_run(action: String) {
    let Some(app) = APP.get().cloned() else { return };
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        perform(&app, &load_settings(), &action, true);
        BUSY.store(false, Ordering::SeqCst);
    });
}

#[tauri::command]
pub fn actions_state() -> View {
    VIEW.lock().clone()
}

#[tauri::command]
pub fn actions_close(app: AppHandle) {
    SHOWN.fetch_add(1, Ordering::SeqCst);
    hide(&app);
}

/// The panel was dragged: it stays open until closed (the pending auto-hide no longer matches).
#[tauri::command]
pub fn actions_keep() {
    SHOWN.fetch_add(1, Ordering::SeqCst);
}

/// Copy button on a result: puts the shown text on the clipboard and hides the panel.
#[tauri::command]
pub fn actions_copy(app: AppHandle) {
    clipboard::set_text(&VIEW.lock().text);
    actions_close(app);
}

/// Replace button on a menu result: hides the panel and pastes the result over the selection in the focused app.
#[tauri::command]
pub fn actions_replace(app: AppHandle) {
    let text = VIEW.lock().text.clone();
    actions_close(app);
    std::thread::spawn(move || apply(TranslateAction::Replace, &text));
}

/// From the hook thread: Esc (`point` None) or a click at `point` (physical pixels) outside the open menu closes it.
pub fn dismiss_menu(point: Option<(i32, i32)>) {
    if !MENU_OPEN.load(Ordering::SeqCst) {
        return;
    }
    let Some(app) = APP.get().cloned() else { return };
    // Off the hook thread: window queries go through the event loop.
    std::thread::spawn(move || {
        let Some(win) = app.get_webview_window(ACTIONS_WINDOW) else { return };
        if let (Some((x, y)), Ok(pos), Ok(size)) = (point, win.outer_position(), win.outer_size()) {
            let inside = x >= pos.x && y >= pos.y && x < pos.x + size.width as i32 && y < pos.y + size.height as i32;
            if inside {
                return;
            }
        }
        if MENU_OPEN.swap(false, Ordering::SeqCst) {
            SHOWN.fetch_add(1, Ordering::SeqCst);
            hide(&app);
        }
    });
}

/// Fits the panel height to its content and keeps it next to the pointer.
#[tauri::command]
pub fn actions_fit(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window(ACTIONS_WINDOW) {
        let _ = win.set_size(LogicalSize::new(WIDTH, height.clamp(40.0, 360.0)));
        place(&app);
    }
}

fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(ACTIONS_WINDOW) {
        let _ = win.hide();
    }
}

/// The extra step chosen in settings: copy the result, or paste it over the selection in the app that still has focus.
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
