//! Tauri commands for the settings window. Each is a thin wrapper over
//! `evkey_engine::settings` (config files), `autostart`, `foreground` or `tray`.
//! The `*_in` functions take the config directory so they can be tested with a temp dir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use evkey_engine::import;
use evkey_engine::settings::{self, Loaded, Settings};
use evkey_engine::hotkey::parse_hotkey;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{admin, autostart, foreground, tray};
use crate::keyboard::mode::{self, Mode};

const GITHUB_URL: &str = "https://github.com/ngocnc/EVKey";

#[derive(Serialize)]
pub struct State {
    mode: Mode,
    autostart: bool,
    /// "Chạy với quyền quản trị": the elevated sign-in task exists.
    admin: bool,
    /// This process runs elevated.
    elevated: bool,
    config_dir: String,
    /// Set when config.toml or macros.toml is malformed; `settings`/`macros` are then defaults.
    config_error: Option<String>,
    settings: Settings,
    macros: BTreeMap<String, String>,
    version: &'static str,
}

#[derive(Deserialize)]
pub struct MacroRow {
    key: String,
    value: String,
}

fn config_dir() -> Result<PathBuf, String> {
    settings::default_dir().ok_or_else(|| crate::i18n::tr("Không tìm thấy thư mục cấu hình", "Config folder not found").to_string())
}

/// Replaces the engine settings and keeps the macros already on disk.
fn save_settings_in(dir: &Path, new_settings: Settings) -> Result<(), String> {
    parse_hotkey(&new_settings.toggle_hotkey).map_err(|e| format!("{}{e}", crate::i18n::tr("Phím tắt không hợp lệ: ", "Invalid hotkey: ")))?;
    let mut cfg = settings::load(dir)?;
    cfg.settings = new_settings;
    settings::save(dir, &cfg)
}

/// Validates macro rows, then replaces the macros and keeps the settings already on disk.
fn save_macros_in(dir: &Path, rows: Vec<MacroRow>) -> Result<(), String> {
    let mut macros = BTreeMap::new();
    for row in rows {
        let key = row.key.trim().to_string();
        if key.is_empty() {
            return Err(crate::i18n::tr("Từ viết tắt không được để trống.", "An abbreviation cannot be empty.").into());
        }
        if macros.insert(key.clone(), row.value).is_some() {
            return Err(format!("{}{key}", crate::i18n::tr("Từ viết tắt bị trùng: ", "Duplicate abbreviation: ")));
        }
    }
    let mut cfg = settings::load(dir)?;
    cfg.macros = macros;
    settings::save(dir, &cfg)
}

fn reset_config_in(dir: &Path) -> Result<(), String> {
    settings::save(dir, &Loaded::default())
}

/// Imports UniKey/OpenKey macros. `replace` drops the current macros; otherwise imported keys
/// overwrite same-named macros and the rest are kept. Returns the number of macros imported.
/// `bytes` is the file content: the settings page reads the file the user picked with `<input type="file">`.
fn import_macros_in(dir: &Path, bytes: &[u8], replace: bool) -> Result<usize, String> {
    let mut imported = BTreeMap::new();
    for (key, text) in import::parse_macros(bytes)? {
        if imported.insert(key.clone(), text).is_some() {
            return Err(format!("{}{key}", crate::i18n::tr("Từ viết tắt bị trùng: ", "Duplicate abbreviation: ")));
        }
    }
    let count = imported.len();
    let mut cfg = settings::load(dir)?;
    if replace {
        cfg.macros.clear();
    }
    cfg.macros.extend(imported);
    settings::save(dir, &cfg)?;
    Ok(count)
}

/// Writes `config.toml` and `macros.toml` as one document. Contains no secrets.
fn export_config_in(dir: &Path, path: &Path) -> Result<(), String> {
    let text = import::encode_config(&settings::load(dir)?)?;
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Validates the whole document first; nothing is written unless it passes.
fn import_config_in(dir: &Path, text: &str) -> Result<(), String> {
    settings::save(dir, &import::decode_config(text)?)
}

fn explorer(target: &str) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_state() -> Result<State, String> {
    let dir = config_dir()?;
    let (cfg, config_error) = match settings::load(&dir) {
        Ok(cfg) => (cfg, None),
        Err(e) => (Loaded::default(), Some(e)),
    };
    Ok(State {
        mode: mode::get(),
        autostart: autostart::is_enabled(),
        admin: admin::task_exists(),
        elevated: admin::is_elevated(),
        config_dir: dir.display().to_string(),
        config_error,
        settings: cfg.settings,
        macros: cfg.macros,
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    save_settings_in(&config_dir()?, settings)?;
    // The tooltip shows the hotkey, so it changes with the settings.
    tray::refresh_icon(&app);
    Ok(())
}

#[tauri::command]
pub fn save_macros(rows: Vec<MacroRow>) -> Result<(), String> {
    save_macros_in(&config_dir()?, rows)
}

#[tauri::command]
pub fn reset_config() -> Result<(), String> {
    reset_config_in(&config_dir()?)
}

#[tauri::command]
pub fn import_macros(content: Vec<u8>, replace: bool) -> Result<usize, String> {
    import_macros_in(&config_dir()?, &content, replace)
}

/// Saves the export as `EVKey-config.toml` in the user's Downloads folder, selects it in Explorer and returns its path.
// ponytail: fixed folder instead of a save dialog (no dialog plugin); add tauri-plugin-dialog if users want to choose.
#[tauri::command]
pub fn export_config() -> Result<String, String> {
    let downloads = std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("Downloads")).ok_or_else(|| {
        crate::i18n::tr("Không tìm thấy thư mục Downloads", "Downloads folder not found").to_string()
    })?;
    let path = downloads.join("EVKey-config.toml");
    export_config_in(&config_dir()?, &path)?;
    let _ = Command::new("explorer.exe").arg(format!("/select,{}", path.display())).spawn();
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn import_config(app: AppHandle, content: String) -> Result<(), String> {
    import_config_in(&config_dir()?, &content)?;
    // The hotkey may have changed, and the tooltip shows it.
    tray::refresh_icon(&app);
    Ok(())
}

#[tauri::command]
pub fn set_autostart(on: bool) -> Result<(), String> {
    autostart::set(on)
}

/// Asks for UAC and turns the elevated sign-in task on or off. Turning it on restarts EVKey elevated.
#[tauri::command]
pub fn set_admin(on: bool) -> Result<(), String> {
    admin::request(on)
}

#[tauri::command]
pub fn open_config_dir() -> Result<(), String> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    explorer(&dir.display().to_string())
}

#[tauri::command]
pub fn open_github() -> Result<(), String> {
    explorer(GITHUB_URL)
}

#[tauri::command]
pub fn last_app() -> Option<String> {
    foreground::last_app()
}

#[tauri::command]
pub async fn show_settings(app: AppHandle) {
    tray::show_settings(&app);
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(key: &str, value: &str) -> MacroRow {
        MacroRow {
            key: key.into(),
            value: value.into(),
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("evkey-app-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn settings_save_keeps_macros() {
        let dir = temp_dir("settings");
        save_macros_in(&dir, vec![row("btw", "by the way")]).unwrap();
        let new_settings = Settings {
            free_mark: true,
            ..Settings::default()
        };
        save_settings_in(&dir, new_settings).unwrap();
        let cfg = settings::load(&dir).unwrap();
        assert!(cfg.settings.free_mark);
        assert_eq!(cfg.macros.get("btw").map(String::as_str), Some("by the way"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn macro_rows_are_validated_before_writing() {
        let dir = temp_dir("macros");
        assert!(save_macros_in(&dir, vec![row("  ", "x")]).is_err());
        assert!(save_macros_in(&dir, vec![row("a", "1"), row(" a ", "2")]).is_err());
        assert!(!dir.exists());
    }

    #[test]
    fn hotkey_is_validated_before_saving() {
        let dir = temp_dir("hotkey-save");
        let bad = Settings {
            toggle_hotkey: "alt+".into(),
            ..Settings::default()
        };
        assert!(save_settings_in(&dir, bad).is_err());
        assert!(!dir.exists());
        let good = Settings {
            toggle_hotkey: "ctrl+shift".into(),
            ..Settings::default()
        };
        save_settings_in(&dir, good).unwrap();
        assert_eq!(settings::load(&dir).unwrap().settings.toggle_hotkey, "ctrl+shift");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bad_hotkey_in_config_is_a_load_error() {
        let dir = temp_dir("hotkey-load");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.toml"), "toggle_hotkey = \"alt+\"\n").unwrap();
        assert!(settings::load(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_macros_merges_or_replaces() {
        let dir = temp_dir("import-macros");
        save_macros_in(&dir, vec![row("btw", "old"), row("keep", "kept")]).unwrap();
        let file = ";[DO NOT DELETE THIS LINE]***version=1\nbtw:by the way\nvn:Việt Nam\n".as_bytes();
        assert_eq!(import_macros_in(&dir, file, false).unwrap(), 2);
        let m = settings::load(&dir).unwrap().macros;
        assert_eq!(m.get("btw").map(String::as_str), Some("by the way"));
        assert_eq!(m.get("keep").map(String::as_str), Some("kept"));
        assert_eq!(import_macros_in(&dir, file, true).unwrap(), 2);
        let m = settings::load(&dir).unwrap().macros;
        assert_eq!(m.len(), 2);
        assert!(!m.contains_key("keep"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bad_macro_file_changes_nothing() {
        let dir = temp_dir("import-macros-bad");
        save_macros_in(&dir, vec![row("btw", "by the way")]).unwrap();
        let err = import_macros_in(&dir, b"ok:fine\nbroken line\n", true).unwrap_err();
        assert!(err.starts_with("line 2:"), "{err}");
        assert_eq!(settings::load(&dir).unwrap().macros.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_export_then_import_restores_settings_and_macros() {
        let src = temp_dir("export-src");
        save_macros_in(&src, vec![row("btw", "by the way")]).unwrap();
        save_settings_in(&src, Settings { free_mark: true, ..Settings::default() }).unwrap();
        let file = src.join("evkey-config.toml");
        export_config_in(&src, &file).unwrap();
        let dst = temp_dir("export-dst");
        import_config_in(&dst, &std::fs::read_to_string(&file).unwrap()).unwrap();
        let cfg = settings::load(&dst).unwrap();
        assert!(cfg.settings.free_mark);
        assert_eq!(cfg.macros.get("btw").map(String::as_str), Some("by the way"));
        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dst);
    }

    #[test]
    fn invalid_config_import_writes_nothing() {
        let dir = temp_dir("import-config-bad");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(import_config_in(&dir, "format_version = 1\n[settings]\ntoggle_hotkey = \"alt+\"\n[macros]\n").is_err());
        assert!(!dir.join("config.toml").exists());
        assert!(!dir.join("macros.toml").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
