//! Update check against GitHub Releases, and the install step (download, verify SHA256, run the installer).
//! The release body holds the notes plus a `SHA256: <hex>` line written by CI (.github/workflows/release.yml).

use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::time::Duration;

use evkey_engine::settings;
use parking_lot::Mutex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};

use crate::i18n::tr;
use crate::tray;

const LATEST_URL: &str = "https://api.github.com/repos/ngocnc/EVKey/releases/latest";
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(20);
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const UPDATE_WINDOW: &str = "update";

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub notes: String,
    url: String,
    sha256: Option<String>,
}

/// The release the update window is showing.
static PENDING: Mutex<Option<Release>> = Mutex::new(None);

fn parse_version(v: &str) -> Option<Vec<u64>> {
    v.trim_start_matches('v').split('.').map(|p| p.parse().ok()).collect()
}

fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

/// Splits the release body into user notes and the CI-written SHA256 (hex, 64 chars).
fn split_body(body: &str) -> (String, Option<String>) {
    let mut sha = None;
    let mut notes = Vec::new();
    for line in body.lines() {
        if let Some(rest) = line.trim().strip_prefix("SHA256:") {
            let hex = rest.trim().trim_matches('`').to_ascii_lowercase();
            if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                sha = Some(hex);
            }
        } else {
            notes.push(line);
        }
    }
    (notes.join("\n").trim().to_string(), sha)
}

fn parse_release(json: &serde_json::Value) -> Option<Release> {
    let version = json["tag_name"].as_str()?.trim_start_matches('v').to_string();
    let url = json["assets"]
        .as_array()?
        .iter()
        .find(|a| a["name"].as_str().is_some_and(|n| n.starts_with("EVKey-Setup-") && n.ends_with(".exe")))?["browser_download_url"]
        .as_str()?
        .to_string();
    let (notes, sha256) = split_body(json["body"].as_str().unwrap_or_default());
    Some(Release { version, notes, url, sha256 })
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).user_agent(concat!("EVKey/", env!("CARGO_PKG_VERSION"))).build()
}

/// The newest release if it is newer than the running version.
pub fn check() -> Result<Option<Release>, String> {
    let json: serde_json::Value = agent().get(LATEST_URL).call().map_err(|e| e.to_string())?.into_json().map_err(|e| e.to_string())?;
    Ok(parse_release(&json).filter(|r| is_newer(&r.version, env!("CARGO_PKG_VERSION"))))
}

fn show_window(app: &AppHandle, release: Release) {
    *PENDING.lock() = Some(release);
    if let Some(win) = crate::tray::lazy_window(app, UPDATE_WINDOW, "") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
        let _ = app.emit("update-show", ());
    }
}

/// Background loop: first check shortly after start, then daily. Honours `auto_check_updates` and the skipped version.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK_DELAY);
        loop {
            let cfg = settings::default_dir().and_then(|d| settings::load(&d).ok()).map(|l| l.settings);
            if let Some(s) = cfg.filter(|s| s.auto_check_updates) {
                if let Ok(Some(r)) = check() {
                    if r.version != s.skipped_update {
                        show_window(&app, r);
                    }
                }
            }
            std::thread::sleep(CHECK_EVERY);
        }
    });
}

#[derive(Serialize)]
pub struct CheckResult {
    /// `latest`, `available` (window opened) or `error`.
    status: &'static str,
    error: Option<String>,
}

/// Manual check from the About tab: ignores `auto_check_updates` and the skipped version.
#[tauri::command]
pub async fn check_update_now(app: AppHandle) -> CheckResult {
    match tauri::async_runtime::spawn_blocking(check).await {
        Ok(Ok(Some(r))) => {
            show_window(&app, r);
            CheckResult { status: "available", error: None }
        }
        Ok(Ok(None)) => CheckResult { status: "latest", error: None },
        Ok(Err(e)) => CheckResult { status: "error", error: Some(e) },
        Err(e) => CheckResult { status: "error", error: Some(e.to_string()) },
    }
}

#[tauri::command]
pub fn update_pending() -> Option<Release> {
    PENDING.lock().clone()
}

/// "Bỏ qua": remember the version so its popup never shows again.
#[tauri::command]
pub fn update_skip(app: AppHandle) {
    if let Some(r) = PENDING.lock().take() {
        tray::update_settings(|s| s.skipped_update = r.version);
    }
    update_later(app);
}

/// Closes the window; the next launch (or the daily check) shows it again.
#[tauri::command]
pub fn update_later(app: AppHandle) {
    if let Some(win) = app.get_webview_window(UPDATE_WINDOW) {
        let _ = win.destroy();
    }
}

fn download(url: &str, expect_sha: Option<&str>) -> Result<PathBuf, String> {
    let resp = agent().get(url).call().map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if let Some(expected) = expect_sha {
        let actual: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
        if actual != expected {
            return Err(tr("Tệp tải về không vượt qua kiểm tra SHA256", "The downloaded file failed its SHA256 check").into());
        }
    }
    let path = std::env::temp_dir().join("EVKey-Setup-update.exe");
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Downloads and verifies the installer, then starts it elevated (UAC) and silent. The installer stops EVKey itself,
/// installs, and relaunches it because of `/relaunch=1`. Cancelling UAC leaves this instance running.
#[tauri::command]
pub async fn update_install(app: AppHandle) -> Result<(), String> {
    let release = PENDING.lock().clone().ok_or("no pending update")?;
    let path = tauri::async_runtime::spawn_blocking(move || download(&release.url, release.sha256.as_deref())).await.map_err(|e| e.to_string())??;
    // `start` goes through ShellExecute, which shows the UAC prompt the installer manifest asks for.
    std::process::Command::new("cmd")
        .args(["/C", "start", "", path.to_string_lossy().as_ref(), "/SILENT", "/NORESTART", "/relaunch=1"])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| e.to_string())?;
    update_later(app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_compares_numerically() {
        assert!(is_newer("v2.10.0", "2.9.9"));
        assert!(!is_newer("2.2.3", "2.2.3"));
        assert!(!is_newer("2.2.2", "2.2.3"));
        assert!(!is_newer("nightly", "2.2.3"));
    }

    #[test]
    fn body_splits_notes_and_sha() {
        let hash = "a".repeat(64);
        let (notes, sha) = split_body(&format!("- Fix A\n- Fix B\n\nSHA256: `{hash}`\n"));
        assert_eq!(notes, "- Fix A\n- Fix B");
        assert_eq!(sha.as_deref(), Some(hash.as_str()));
        assert_eq!(split_body("SHA256: short").1, None);
    }

    #[test]
    fn release_needs_a_setup_asset() {
        let json = serde_json::json!({"tag_name": "v9.0.0", "body": "", "assets": [{"name": "EVKey-9.0.0.dmg", "browser_download_url": "x"}]});
        assert_eq!(parse_release(&json), None);
        let json = serde_json::json!({"tag_name": "v9.0.0", "body": "n", "assets": [{"name": "EVKey-Setup-9.0.0.exe", "browser_download_url": "u"}]});
        assert_eq!(parse_release(&json).map(|r| (r.version, r.url)), Some(("9.0.0".into(), "u".into())));
    }
}
