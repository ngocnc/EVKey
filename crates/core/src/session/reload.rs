//! Settings reload: picks up edits to `config.toml` / `macros.toml` while the host runs.
use std::time::{Duration, Instant};

use evkey_engine::hotkey::parse_hotkey;
use evkey_engine::settings;

use super::Session;

impl Session {
    /// Reloads `config.toml` / `macros.toml` when they changed (checked at most once a second). `force` skips the
    /// throttle and the mtime check. A malformed file keeps the previous settings. Everything but the hotkey waits for a word boundary.
    pub(super) fn reload(&mut self, force: bool) {
        let now = Instant::now();
        if !force && self.last_check.is_some_and(|t| now.duration_since(t) < Duration::from_secs(1)) {
            return;
        }
        self.last_check = Some(now);
        let Some(dir) = self.dir.clone() else { return };
        let stamp = settings::stamp(&dir);
        if !force && self.stamp == Some(stamp) {
            return;
        }
        let Ok(loaded) = settings::load(&dir) else {
            self.stamp = Some(stamp);
            return;
        };
        if let Ok(hk) = parse_hotkey(&loaded.settings.toggle_hotkey) {
            self.hotkey = hk;
        }
        self.translate_hotkey = parse_hotkey(&loaded.settings.translate_hotkey).ok().flatten();
        self.action_hotkey = parse_hotkey(&loaded.settings.action_hotkey).ok().flatten();
        if self.composer.is_composing() {
            // Stamp stays stale, so the next check applies the rest after the word ends.
            self.stamp = None;
            return;
        }
        self.stamp = Some(stamp);
        self.composer.apply(&loaded);
        self.settings = loaded.settings;
        // Re-evaluate the exclusion list for the current app under the new settings.
        self.excluded = self.app_id.as_deref().is_some_and(|a| self.settings.is_excluded(a));
    }
}
