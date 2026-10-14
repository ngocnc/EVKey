//! UI language for Rust-side text: tray tooltip, errors shown in the settings window, CLI messages.
//! The web UI has its own dictionary in `ui/i18n.js`.
use std::time::SystemTime;

use evkey_engine::settings::{self, UiLanguage};
use parking_lot::Mutex;

/// Last language read from config.toml, with the file stamps it was read at.
static CACHE: Mutex<Option<([Option<SystemTime>; 2], bool)>> = Mutex::new(None);

/// Whether the saved `ui_language` is English. Re-reads config.toml only when `settings::stamp` changes, so a save
/// applies at once without parsing the file on every call.
fn english() -> bool {
    let Some(dir) = settings::default_dir() else { return false };
    let stamp = settings::stamp(&dir);
    let mut cache = CACHE.lock();
    if let Some((seen, english)) = *cache {
        if seen == stamp {
            return english;
        }
    }
    let english = settings::load(&dir).is_ok_and(|cfg| matches!(cfg.settings.ui_language, UiLanguage::En));
    *cache = Some((stamp, english));
    english
}

/// Returns the `vi` or `en` text for the saved `ui_language`. Falls back to Vietnamese when the file is missing or
/// unreadable.
pub fn tr(vi: &'static str, en: &'static str) -> &'static str {
    if english() { en } else { vi }
}
