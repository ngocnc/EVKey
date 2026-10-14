//! User settings: `config.toml` (engine options, excluded apps) and `macros.toml` (abbreviations).
//! Missing files mean defaults; a malformed file is an error so the host can keep the old settings.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::engine::Config;
use crate::hotkey::parse_hotkey;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    #[default]
    Telex,
    Vni,
    SimpleTelex1,
    SimpleTelex2,
}

/// Where the tone mark goes on a two-vowel rhyme: new style `hòa`, old style `hoà`.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TonePlacement {
    #[default]
    New,
    Old,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeTable {
    #[default]
    Unicode,
    Tcvn3,
    VniWindows,
    UnicodeCompound,
}

/// Key whose press mid-word leaves the rest of that word unconverted.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TempOffKey {
    #[default]
    None,
    Ctrl,
    Alt,
}

/// Display language of every EVKey screen. Not the V/E Chế độ.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiLanguage {
    #[default]
    Vi,
    En,
}

/// What EVKey does with a translation besides showing it in the popup.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TranslateAction {
    /// Only show the popup.
    #[default]
    Show,
    /// Also put the translation on the clipboard.
    Copy,
    /// Also paste the translation over the selected text.
    Replace,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub input_type: InputKind,
    pub tone_placement: TonePlacement,
    pub code_table: CodeTable,
    /// Allow typing a tone mark after the syllable is finished.
    pub free_mark: bool,
    pub check_spelling: bool,
    /// Restore the typed keys when the syllable is not valid Vietnamese (`window` stays `window`).
    pub restore_if_wrong_spelling: bool,
    /// `cc`→`ch`, `gg`→`gi`, `kk`→`kh`, `nn`→`ng`, `qq`→`qu`, `pp`→`ph`, `tt`→`th`.
    pub quick_telex: bool,
    /// `f`→`ph`, `j`→`gi`, `w`→`qu` at the start of a syllable.
    pub quick_start_consonant: bool,
    /// `g`→`ng`, `h`→`nh`, `k`→`ch` at the end of a syllable.
    pub quick_end_consonant: bool,
    /// Letters (`a`-`z`) the engine accepts as a syllable start while `allow_consonant_zfwj` is on.
    pub custom_consonants: Vec<String>,
    pub upper_case_first_char: bool,
    /// When `upper_case_first_char` is on: capitalize after `.`, `?` and `!`, after Enter, and the first letter typed
    /// after a focus change or click.
    pub upper_case_after_punct: bool,
    pub upper_case_after_enter: bool,
    pub upper_case_first_letter: bool,
    pub allow_consonant_zfwj: bool,
    /// `Btw` expands to `By the way`, `BTW` to `BY THE WAY`.
    pub auto_caps_macro: bool,
    /// Windows executable names (e.g. `game.exe`) where the IME never converts keys.
    pub excluded_apps: Vec<String>,
    /// macOS bundle identifiers (e.g. `com.apple.Terminal`) where the IME never converts keys.
    pub excluded_apps_macos: Vec<String>,
    /// Tap this key while typing a word to leave the rest of the word unconverted.
    pub temp_off_key: TempOffKey,
    /// Key or modifier chord that toggles Vietnamese/English (`alt+z`, `ctrl+shift`, `none` to disable).
    pub toggle_hotkey: String,
    /// Removed (ADR 0006: Windows always uses the keyboard hook). Still accepted so older `config.toml` files load;
    /// dropped on the next save.
    #[serde(rename = "backend", skip_serializing)]
    #[doc(hidden)]
    pub legacy_backend: Option<String>,
    /// Secure Input banner: background opacity, 0-100 (100 = opaque).
    pub banner_opacity: u8,
    /// Secure Input banner: glass blur, 0-100; 0 turns the glass effect off.
    pub banner_blur: u8,
    /// Secure Input banner colors as `#rrggbb`.
    pub banner_bg: String,
    pub banner_border: String,
    pub banner_text: String,
    /// Optional gradient end colors (`#rrggbb`); empty keeps the matching color solid.
    pub banner_bg_end: String,
    pub banner_border_end: String,
    pub banner_text_end: String,
    /// Remember the V/E mode per app (`app_modes.toml`) and restore it when the app is focused again.
    pub remember_app_mode: bool,
    /// Play a short sound on a user-initiated V/E switch, not on automatic per-app restores.
    pub toggle_sound: bool,
    /// Esc while a word is composing restores its raw keys and leaves the rest of the word unconverted.
    pub undo_word: bool,
    /// Display language of the settings window, popup, tray menu, banner and alerts.
    pub ui_language: UiLanguage,
    /// Check GitHub Releases for a newer EVKey at startup and every 24 hours.
    pub auto_check_updates: bool,
    /// Version (`2.3.0`) the user chose to skip; its update popup is not shown again.
    pub skipped_update: String,
    /// Key chord (`ctrl+alt+t`, must end in a key; `none` disables) that translates the selected text.
    pub translate_hotkey: String,
    /// What happens to the translation after it is shown.
    pub translate_action: TranslateAction,
    /// Milliseconds the translation popup stays on screen; 0 keeps it until it is clicked. Otherwise 100-600000.
    pub translate_popup_ms: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            input_type: InputKind::Telex,
            tone_placement: TonePlacement::New,
            code_table: CodeTable::Unicode,
            free_mark: false,
            check_spelling: true,
            restore_if_wrong_spelling: true,
            quick_telex: false,
            quick_start_consonant: false,
            quick_end_consonant: false,
            custom_consonants: ["z", "f", "w", "j"].map(String::from).to_vec(),
            upper_case_first_char: false,
            upper_case_after_punct: true,
            upper_case_after_enter: true,
            upper_case_first_letter: false,
            allow_consonant_zfwj: false,
            auto_caps_macro: false,
            excluded_apps: Vec::new(),
            excluded_apps_macos: Vec::new(),
            temp_off_key: TempOffKey::None,
            toggle_hotkey: if cfg!(target_os = "macos") { "ctrl+shift" } else { "alt+z" }.to_string(),
            legacy_backend: None,
            banner_opacity: 80,
            banner_blur: 60,
            banner_bg: "#1c1c1e".to_string(),
            banner_border: "#ff9f0a".to_string(),
            banner_text: "#ffffff".to_string(),
            banner_bg_end: String::new(),
            banner_border_end: String::new(),
            banner_text_end: String::new(),
            remember_app_mode: true,
            toggle_sound: false,
            undo_word: true,
            ui_language: UiLanguage::Vi,
            auto_check_updates: true,
            skipped_update: String::new(),
            translate_hotkey: "ctrl+alt+t".to_string(),
            translate_action: TranslateAction::Show,
            translate_popup_ms: 5000,
        }
    }
}

fn is_letter(s: &str) -> bool {
    s.len() == 1 && s.bytes().all(|b| b.is_ascii_lowercase())
}

impl Settings {
    /// Engine configuration. `use_macro` is on iff there are macros.
    pub fn to_config(&self, has_macros: bool) -> Config {
        Config {
            input_type: self.input_type as i32,
            free_mark: self.free_mark as i32,
            code_table: self.code_table as i32,
            check_spelling: self.check_spelling as i32,
            // The engine's variable is named after the opposite style: 1 means `hoà`.
            use_modern_orthography: (self.tone_placement == TonePlacement::Old) as i32,
            quick_telex: self.quick_telex as i32,
            restore_if_wrong_spelling: self.restore_if_wrong_spelling as i32,
            use_macro: has_macros as i32,
            auto_caps_macro: self.auto_caps_macro as i32,
            upper_case_first_char: self.upper_case_first_char as i32,
            upper_case_after_period: self.upper_case_after_punct as i32,
            upper_case_after_question: self.upper_case_after_punct as i32,
            upper_case_after_exclaim: self.upper_case_after_punct as i32,
            upper_case_after_enter: self.upper_case_after_enter as i32,
            upper_case_on_reset: self.upper_case_first_letter as i32,
            allow_consonant_zfwj: self.allow_consonant_zfwj as i32,
            quick_start_consonant: self.quick_start_consonant as i32,
            quick_end_consonant: self.quick_end_consonant as i32,
            // Engine key codes are the uppercase ASCII letters.
            custom_consonants: self.custom_consonants.iter().filter(|s| is_letter(s)).map(|s| s.as_bytes()[0].to_ascii_uppercase() as u16).collect(),
        }
    }

    pub fn is_excluded(&self, app_id: &str) -> bool {
        self.is_excluded_on(cfg!(target_os = "macos"), app_id)
    }

    /// Picks the platform's list: `macos` selects bundle IDs, otherwise Windows exe names.
    fn is_excluded_on(&self, macos: bool, app_id: &str) -> bool {
        let list = if macos { &self.excluded_apps_macos } else { &self.excluded_apps };
        list.iter().any(|a| a.eq_ignore_ascii_case(app_id))
    }

    /// Rejects banner values the UI cannot show; checked on load and save.
    pub fn validate(&self) -> Result<(), String> {
        if self.banner_opacity > 100 || self.banner_blur > 100 {
            return Err("banner_opacity và banner_blur phải từ 0 đến 100".into());
        }
        for (name, color) in [("banner_bg", &self.banner_bg), ("banner_border", &self.banner_border), ("banner_text", &self.banner_text)] {
            if !is_hex_color(color) {
                return Err(format!("{name} phải có dạng #rrggbb: {color}"));
            }
        }
        for (name, color) in [
            ("banner_bg_end", &self.banner_bg_end),
            ("banner_border_end", &self.banner_border_end),
            ("banner_text_end", &self.banner_text_end),
        ] {
            if !color.is_empty() && !is_hex_color(color) {
                return Err(format!("{name} phải để trống hoặc có dạng #rrggbb: {color}"));
            }
        }
        if let Some(bad) = self.custom_consonants.iter().find(|s| !is_letter(s)) {
            return Err(format!("custom_consonants: mỗi phụ âm phải là 1 chữ cái a-z: {bad}"));
        }
        match parse_hotkey(&self.translate_hotkey) {
            Ok(Some(hk)) if hk.vk.is_none() => return Err(format!("translate_hotkey phải kết thúc bằng một phím: {}", self.translate_hotkey)),
            Err(e) => return Err(format!("translate_hotkey: {e}")),
            _ => {}
        }
        if self.translate_popup_ms != 0 && !(100..=600_000).contains(&self.translate_popup_ms) {
            return Err("translate_popup_ms phải là 0 (không tự đóng) hoặc từ 100 đến 600000".into());
        }
        Ok(())
    }
}

/// Both files, parsed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Loaded {
    pub settings: Settings,
    pub macros: BTreeMap<String, String>,
}

/// `%APPDATA%\EVKey` on Windows, `~/Library/Application Support/EVKey` on macOS.
pub fn default_dir() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support/EVKey"))
    } else {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("EVKey"))
    }
}

fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

fn read_optional(path: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

pub fn load(dir: &Path) -> Result<Loaded, String> {
    let mut loaded = Loaded::default();
    let cfg = dir.join("config.toml");
    if let Some(text) = read_optional(&cfg)? {
        loaded.settings = toml::from_str(&text).map_err(|e| format!("{}: {e}", cfg.display()))?;
        crate::hotkey::parse_hotkey(&loaded.settings.toggle_hotkey)
            .map_err(|e| format!("{}: toggle_hotkey: {e}", cfg.display()))?;
        loaded.settings.validate().map_err(|e| format!("{}: {e}", cfg.display()))?;
    }
    let mac = dir.join("macros.toml");
    if let Some(text) = read_optional(&mac)? {
        loaded.macros = toml::from_str(&text).map_err(|e| format!("{}: {e}", mac.display()))?;
    }
    Ok(loaded)
}

/// Modification times of the two files; compare with a previous stamp to detect edits.
pub fn stamp(dir: &Path) -> [Option<SystemTime>; 2] {
    let m = |f: &str| std::fs::metadata(dir.join(f)).and_then(|m| m.modified()).ok();
    [m("config.toml"), m("macros.toml")]
}

fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Write both files (creating `dir`). Comments in hand-edited files are not preserved.
pub fn save(dir: &Path, loaded: &Loaded) -> Result<(), String> {
    loaded.settings.validate()?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let cfg = toml::to_string_pretty(&loaded.settings).map_err(|e| e.to_string())?;
    write_atomic(&dir.join("config.toml"), &cfg)?;
    let mac = toml::to_string_pretty(&loaded.macros).map_err(|e| e.to_string())?;
    write_atomic(&dir.join("macros.toml"), &mac)
}

const APP_MODES_FILE: &str = "app_modes.toml";

/// Per-app Mode (`true` = English) for "Chế độ theo ứng dụng", in `app_modes.toml`. Missing or malformed: empty.
pub fn load_app_modes(dir: &Path) -> BTreeMap<String, bool> {
    std::fs::read_to_string(dir.join(APP_MODES_FILE)).ok().and_then(|t| toml::from_str(&t).ok()).unwrap_or_default()
}

pub fn save_app_modes(dir: &Path, modes: &BTreeMap<String, bool>) -> Result<(), String> {
    let text = toml::to_string_pretty(modes).map_err(|e| e.to_string())?;
    write_atomic(&dir.join(APP_MODES_FILE), &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excluded_list_follows_platform() {
        let s = Settings {
            excluded_apps: vec!["game.exe".into()],
            excluded_apps_macos: vec!["com.apple.Terminal".into()],
            ..Settings::default()
        };
        assert!(s.is_excluded_on(false, "GAME.exe"));
        assert!(!s.is_excluded_on(false, "com.apple.Terminal"));
        assert!(s.is_excluded_on(true, "com.apple.terminal"));
        assert!(!s.is_excluded_on(true, "game.exe"));
    }

    #[test]
    fn translate_popup_time_is_validated_and_action_round_trips() {
        let mut s = Settings { translate_popup_ms: 99, ..Settings::default() };
        assert!(s.validate().is_err());
        s.translate_popup_ms = 600_001;
        assert!(s.validate().is_err());
        s.translate_popup_ms = 0;
        assert!(s.validate().is_ok());
        s.translate_popup_ms = 30_000;
        assert!(s.validate().is_ok());
        let toml_text = toml::to_string(&Settings { translate_action: TranslateAction::Replace, ..Settings::default() }).unwrap();
        assert!(toml_text.contains("translate_action = \"replace\""));
        assert_eq!(toml::from_str::<Settings>(&toml_text).unwrap().translate_action, TranslateAction::Replace);
    }

    #[test]
    fn banner_values_are_validated() {
        let mut s = Settings::default();
        assert!(s.validate().is_ok());
        s.banner_bg = "red".into();
        assert!(s.validate().is_err());
        s.banner_bg = "#12345G".into();
        assert!(s.validate().is_err());
        s.banner_bg = "#12ab34".into();
        s.banner_blur = 101;
        assert!(s.validate().is_err());
        s.banner_blur = 60;
        s.banner_bg_end = "#ff00aa".into();
        assert!(s.validate().is_ok());
        s.banner_text_end = "pink".into();
        assert!(s.validate().is_err());
    }
}
