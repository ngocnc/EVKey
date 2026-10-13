//! Tauri commands for the settings window. Each is a thin wrapper over
//! `evkey_engine::settings` (config files), `keyboard`, `autostart`, `foreground` or `tray`.
//! The `*_in` functions take the config directory so they can be tested with a temp dir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use evkey_engine::settings::{self, Loaded, Settings};
use evkey_engine::hotkey::parse_hotkey;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{autostart, foreground, keyboard, tray};
use crate::keyboard::mode::{self, Mode};

const GITHUB_URL: &str = "https://github.com/ngocnc/EVKey";

#[derive(Serialize)]
pub struct State {
    enabled: bool,
    mode: Mode,
    autostart: bool,
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
    settings::default_dir().ok_or_else(|| "Không tìm thấy %APPDATA%".to_string())
}

/// Replaces the engine settings and keeps the macros already on disk.
fn save_settings_in(dir: &Path, new_settings: Settings) -> Result<(), String> {
    parse_hotkey(&new_settings.toggle_hotkey).map_err(|e| format!("Phím tắt không hợp lệ: {e}"))?;
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
            return Err("Từ viết tắt không được để trống.".into());
        }
        if macros.insert(key.clone(), row.value).is_some() {
            return Err(format!("Từ viết tắt bị trùng: {key}"));
        }
    }
    let mut cfg = settings::load(dir)?;
    cfg.macros = macros;
    settings::save(dir, &cfg)
}

fn reset_config_in(dir: &Path) -> Result<(), String> {
    settings::save(dir, &Loaded::default())
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
        enabled: keyboard::is_enabled()?,
        mode: mode::get()?,
        autostart: autostart::is_enabled(),
        config_dir: dir.display().to_string(),
        config_error,
        settings: cfg.settings,
        macros: cfg.macros,
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[tauri::command]
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    keyboard::set_enabled(enabled)
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
pub fn set_autostart(on: bool) -> Result<(), String> {
    autostart::set(on)
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
pub fn show_settings(app: AppHandle) {
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
}
