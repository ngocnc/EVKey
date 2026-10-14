//! Per-app mode memory: the Vietnamese/English mode of each app, kept in `app_modes.toml` when "remember_app_mode" is on.
use evkey_engine::settings;

use super::Session;

impl Session {
    /// Records the current Mode for `app` when "remember_app_mode" is on. The file is written only on change.
    pub(super) fn remember_mode(&mut self, app: &str) {
        let english = self.is_english();
        if !self.settings.remember_app_mode || self.app_modes.get(app) == Some(&english) {
            return;
        }
        self.app_modes.insert(app.to_owned(), english);
        // ponytail: best-effort save; a failed write only loses the remembered Mode, not typing.
        if let Some(dir) = &self.dir {
            let _ = settings::save_app_modes(dir, &self.app_modes);
        }
    }

    /// Saves the current Mode for the frontmost app, so a change made by any means (hotkey, tray) survives a restart.
    pub fn remember_current(&mut self) {
        if let Some(app) = self.app_id.clone() {
            self.remember_mode(&app);
        }
    }
}
