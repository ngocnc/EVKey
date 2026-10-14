//! Modifier-only handling: the `ctrl+shift`-style toggle chord (fires on release) and the temp-off tap key.
use evkey_engine::settings::TempOffKey;

use super::keys::{wanted, Mods};
use super::Session;

impl Session {
    /// Returns whether this event toggled the mode (a modifier-only chord fires on release).
    pub(super) fn on_modifier(&mut self, g: usize, down: bool, mods: Mods) -> bool {
        let hk = self.hotkey;
        if down {
            self.held[g] = true;
            self.chord_armed = hk.is_some_and(|h| h.vk.is_none() && self.held == wanted(&h));
            self.tap_armed = self.held.iter().filter(|&&h| h).count() == 1;
            return false;
        }
        // The released group is forced up: the host may still report it as held.
        self.held = mods.groups();
        self.held[g] = false;
        let mut toggled = false;
        if let Some(h) = hk {
            if h.vk.is_none() && self.chord_armed && wanted(&h)[g] {
                self.chord_armed = false;
                self.toggle();
                toggled = true;
            }
        }
        let temp_group = match self.settings.temp_off_key {
            TempOffKey::Ctrl => Some(0),
            TempOffKey::Alt => Some(1),
            TempOffKey::None => None,
        };
        if self.tap_armed && temp_group == Some(g) {
            self.tap_armed = false;
            self.composer.engine.temp_off_engine(true);
        }
        toggled
    }
}
