//! UI language for Rust-side text: tray tooltip, errors shown in the settings window, CLI messages.
//! The web UI has its own dictionary in `ui/i18n.js`.
use evkey_engine::settings::{self, UiLanguage};

/// Returns the `vi` or `en` text for the saved `ui_language`. Reads config.toml on each call so a
/// change applies at once; falls back to Vietnamese when the file is missing or unreadable.
pub fn tr(vi: &'static str, en: &'static str) -> &'static str {
    let lang = settings::default_dir()
        .and_then(|dir| settings::load(&dir).ok())
        .map(|cfg| cfg.settings.ui_language)
        .unwrap_or_default();
    match lang {
        UiLanguage::Vi => vi,
        UiLanguage::En => en,
    }
}
