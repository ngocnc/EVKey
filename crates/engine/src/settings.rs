//! User settings: `config.toml` (engine options, excluded apps) and `macros.toml` (abbreviations).
//! Missing files mean defaults; a malformed file is an error so the host can keep the old settings.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Deserialize;

use crate::engine::Config;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    #[default]
    Telex,
    Vni,
    SimpleTelex1,
    SimpleTelex2,
}

/// Where the tone mark goes on a two-vowel rhyme: new style `hòa`, old style `hoà`.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TonePlacement {
    #[default]
    New,
    Old,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeTable {
    #[default]
    Unicode,
    Tcvn3,
    VniWindows,
    UnicodeCompound,
}

/// Key whose press mid-word leaves the rest of that word unconverted.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TempOffKey {
    #[default]
    None,
    Ctrl,
    Alt,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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
    pub upper_case_first_char: bool,
    pub allow_consonant_zfwj: bool,
    /// `Btw` expands to `By the way`, `BTW` to `BY THE WAY`.
    pub auto_caps_macro: bool,
    /// Executable names (e.g. `game.exe`) where the IME never converts keys.
    pub excluded_apps: Vec<String>,
    /// Tap this key while typing a word to leave the rest of the word unconverted.
    pub temp_off_key: TempOffKey,
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
            upper_case_first_char: false,
            allow_consonant_zfwj: false,
            auto_caps_macro: false,
            excluded_apps: Vec::new(),
            temp_off_key: TempOffKey::None,
        }
    }
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
            allow_consonant_zfwj: self.allow_consonant_zfwj as i32,
            quick_start_consonant: self.quick_start_consonant as i32,
            quick_end_consonant: self.quick_end_consonant as i32,
        }
    }

    pub fn is_excluded(&self, exe_name: &str) -> bool {
        self.excluded_apps.iter().any(|a| a.eq_ignore_ascii_case(exe_name))
    }
}

/// Both files, parsed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Loaded {
    pub settings: Settings,
    pub macros: BTreeMap<String, String>,
}

/// `%APPDATA%\EVKey`.
pub fn default_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("EVKey"))
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
