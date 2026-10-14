//! One typing session: the platform-independent part of the keyboard hook. Hosts feed it key events with
//! Windows virtual-key codes and inject the returned [`Output`]; no OS calls happen here.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use evkey_engine::composer::Composer;
use evkey_engine::consts::CAPS_MASK;
use evkey_engine::engine::V_RESTORE_AND_START_NEW_SESSION;
use evkey_engine::hotkey::{parse_hotkey, Hotkey};
use evkey_engine::keys::KEY_ESC;
use evkey_engine::settings::{self, Settings, TempOffKey};
use evkey_engine::tables::key_code_to_character;

use crate::typing::edit_for;

/// Zero-width-ish character typed and deleted first so browser autocomplete cannot swallow the edit (as OpenKey does).
pub const EMPTY_CHAR: u16 = 0x202F;

/// Modifier groups in `Hotkey` field order: ctrl, alt, shift, win.
type Groups = [bool; 4];

/// Physical modifier state at event time, supplied by the host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

impl Mods {
    fn groups(self) -> Groups {
        [self.ctrl, self.alt, self.shift, self.win]
    }
}

/// One key event. `vk` is a Windows virtual-key code (hosts map their native codes).
#[derive(Clone, Copy, Debug)]
pub struct KeyInput {
    pub vk: u16,
    pub down: bool,
    pub mods: Mods,
    pub caps_lock: bool,
}

/// What the host must type when a key was converted: an optional [`EMPTY_CHAR`], `backspaces` backspaces, `text`
/// as UTF-16 units, then the optional `resend`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub prefix_empty_char: bool,
    pub backspaces: usize,
    pub text: String,
    pub resend: Option<Resend>,
}

/// The trigger key typed again after the edit: a Unicode character, or the raw virtual key when it has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resend {
    Char(u16),
    Vk(u16),
}

/// The host's instructions for one event.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reaction {
    /// Do not pass the key to the focused app.
    pub swallow: bool,
    pub edit: Option<Output>,
    /// The Vietnamese/English mode flipped on this event.
    pub hotkey_toggled: bool,
    /// Alt-hotkey: inject a throwaway key (VK 0xFF down/up) so releasing Alt does not open the window menu.
    pub dummy_key: bool,
}

/// Group index of a modifier virtual key (the hook reports the left/right variants); `None` for other keys.
fn group(vk: u16) -> Option<usize> {
    match vk {
        0xA2 | 0xA3 | 0x11 => Some(0),
        0xA4 | 0xA5 | 0x12 => Some(1),
        0xA0 | 0xA1 | 0x10 => Some(2),
        0x5B | 0x5C => Some(3),
        _ => None,
    }
}

/// True for the modifier virtual keys (either side). Hosts skip foreground and Secure Input checks for these.
pub fn is_modifier_vk(vk: u16) -> bool {
    group(vk).is_some()
}

fn wanted(hk: &Hotkey) -> Groups {
    [hk.ctrl, hk.alt, hk.shift, hk.win]
}

pub struct Session {
    composer: Composer,
    settings: Settings,
    dir: Option<PathBuf>,
    stamp: Option<[Option<SystemTime>; 2]>,
    last_check: Option<Instant>,
    hotkey: Option<Hotkey>,
    /// Modifier groups currently held, from the host's modifier state.
    held: Groups,
    /// Modifier-only chord (`ctrl+shift`): armed while exactly its modifiers are held and nothing else was pressed.
    chord_armed: bool,
    /// A lone modifier was pressed and nothing else since: releasing it is a "tap" (the temp-off key).
    tap_armed: bool,
    app_id: Option<String>,
    /// Mode per app (`true` = English) when "remember_app_mode" is on; kept in `app_modes.toml`.
    app_modes: BTreeMap<String, bool>,
    excluded: bool,
    secure: bool,
    seen_english: bool,
    english: Arc<AtomicBool>,
    /// Backing storage for the UTF-16 text handed out by the FFI; valid until the next FFI call.
    pub(crate) text16: Vec<u16>,
}

impl Session {
    /// `dir` holds `config.toml` and `macros.toml`; `None` runs with defaults and never reloads.
    pub fn new(dir: Option<PathBuf>) -> Session {
        Self::with_mode(dir, Arc::new(AtomicBool::new(false)))
    }

    /// Like [`Session::new`], but the Vietnamese/English flag is shared with the caller (see [`Session::mode_handle`]).
    pub fn with_mode(dir: Option<PathBuf>, english: Arc<AtomicBool>) -> Session {
        let app_modes = dir.as_deref().map(settings::load_app_modes).unwrap_or_default();
        let mut s = Session {
            composer: Composer::new(Settings::default().to_config(false)),
            settings: Settings::default(),
            dir,
            stamp: None,
            last_check: None,
            hotkey: None,
            held: [false; 4],
            chord_armed: false,
            tap_armed: false,
            app_id: None,
            app_modes,
            excluded: false,
            secure: false,
            seen_english: false,
            english,
            text16: Vec::new(),
        };
        s.reload(true);
        s
    }

    /// The shared Vietnamese/English flag, readable and settable from any thread.
    pub fn mode_handle(&self) -> Arc<AtomicBool> {
        self.english.clone()
    }

    pub fn is_english(&self) -> bool {
        self.english.load(Ordering::Relaxed)
    }

    /// Sets the mode. The word in progress is dropped on the next key event.
    pub fn set_english(&mut self, on: bool) {
        self.english.store(on, Ordering::Relaxed);
        if let Some(app) = self.app_id.clone() {
            self.remember_mode(&app);
        }
    }

    /// Records the current Mode for `app` when "remember_app_mode" is on. The file is written only on change.
    fn remember_mode(&mut self, app: &str) {
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

    /// Flips the mode and drops the word in progress.
    pub fn toggle(&mut self) {
        self.set_english(!self.is_english());
        self.composer.reset();
    }

    pub fn settings(&self) -> Settings {
        self.settings.clone()
    }

    /// The host lost the composition (mouse click can move the caret): forget the word.
    pub fn on_mouse_click(&mut self) {
        self.composer.reset();
    }

    /// The frontmost app changed. `app_id` is the Windows exe name or macOS bundle ID; `None` when unknown.
    pub fn set_foreground(&mut self, app_id: Option<&str>) {
        self.composer.reset();
        let next = app_id.map(str::to_owned);
        if next != self.app_id && self.settings.remember_app_mode {
            if let Some(prev) = self.app_id.clone() {
                self.remember_mode(&prev);
            }
            let restored = next.as_ref().and_then(|a| self.app_modes.get(a).copied());
            if let Some(english) = restored {
                self.english.store(english, Ordering::Relaxed);
            }
        }
        self.app_id = next;
        self.excluded = self.app_id.as_deref().is_some_and(|a| self.settings.is_excluded(a));
    }

    /// The focused control is a password field (or Secure Input is on): pass keys through untouched.
    pub fn set_secure(&mut self, on: bool) {
        self.secure = on;
    }

    /// Handles one key event and says what the host must do with it.
    pub fn on_key(&mut self, input: KeyInput) -> Reaction {
        let KeyInput { vk, down, mods, caps_lock } = input;
        if self.is_english() != self.seen_english {
            self.seen_english = self.is_english();
            self.composer.reset();
        }
        if let Some(g) = group(vk) {
            let hotkey_toggled = self.on_modifier(g, down, mods);
            return Reaction { hotkey_toggled, ..Reaction::default() };
        }
        if !down {
            return Reaction::default();
        }
        self.tap_armed = false;
        self.chord_armed = false;
        self.held = mods.groups();
        self.reload(false);

        if let Some(hk) = self.hotkey {
            if hk.vk == Some(vk) && self.held == wanted(&hk) {
                self.toggle();
                return Reaction { swallow: true, hotkey_toggled: true, dummy_key: hk.alt, ..Reaction::default() };
            }
        }
        if self.is_english() || self.excluded || self.secure {
            return Reaction::default();
        }

        let shift = self.held[2];
        let caps: u8 = if shift && caps_lock {
            0
        } else if shift {
            1
        } else if caps_lock {
            2
        } else {
            0
        };
        let other_control = self.held[0] || self.held[1] || self.held[3];
        let engine = &mut self.composer.engine;
        // Hoàn tác từ: Esc shows the raw keys of the word in progress; Esc itself is swallowed, so no resend.
        if self.settings.undo_word && vk == KEY_ESC && !other_control && engine.undo_word() {
            let Some(edit) = edit_for(&engine.hook, engine.cfg.code_table) else {
                return Reaction::default();
            };
            return Reaction {
                swallow: true,
                edit: Some(Output {
                    prefix_empty_char: true,
                    backspaces: edit.backspaces + 1,
                    text: edit.text,
                    resend: None,
                }),
                ..Reaction::default()
            };
        }
        engine.key_event(false, vk, caps, other_control);
        let Some(edit) = edit_for(&engine.hook, engine.cfg.code_table) else {
            return Reaction::default();
        };
        if engine.hook.code == V_RESTORE_AND_START_NEW_SESSION {
            engine.start_new_session();
        }
        let resend = edit.resend_key.then(|| {
            let ch = key_code_to_character(vk as u32 | if caps != 0 { CAPS_MASK } else { 0 });
            if ch != 0 { Resend::Char(ch) } else { Resend::Vk(vk) }
        });
        Reaction {
            swallow: true,
            edit: Some(Output {
                prefix_empty_char: true,
                // ponytail: OpenKey's "fix recommend browser" for every app; per-app opt-out if some app dislikes it.
                backspaces: edit.backspaces + 1,
                text: edit.text,
                resend,
            }),
            ..Reaction::default()
        }
    }

    /// Returns whether this event toggled the mode (a modifier-only chord fires on release).
    fn on_modifier(&mut self, g: usize, down: bool, mods: Mods) -> bool {
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

    /// Reloads `config.toml` / `macros.toml` when they changed (checked at most once a second). A malformed file
    /// keeps the previous settings. Everything but the hotkey waits for a word boundary.
    fn reload(&mut self, force: bool) {
        let now = Instant::now();
        if !force && self.last_check.is_some_and(|t| now.duration_since(t) < Duration::from_secs(1)) {
            return;
        }
        self.last_check = Some(now);
        let Some(dir) = self.dir.clone() else { return };
        let stamp = settings::stamp(&dir);
        if self.stamp == Some(stamp) {
            return;
        }
        let Ok(loaded) = settings::load(&dir) else {
            self.stamp = Some(stamp);
            return;
        };
        if let Ok(hk) = parse_hotkey(&loaded.settings.toggle_hotkey) {
            self.hotkey = hk;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use evkey_engine::settings::Loaded;
    use std::collections::BTreeMap;

    const CTRL: u16 = 0xA2;
    const SHIFT: u16 = 0xA0;
    const ALT: u16 = 0x12;
    const A: u16 = 0x41;
    const X: u16 = 0x58;
    const Z: u16 = 0x5A;

    fn mods(ctrl: bool, alt: bool, shift: bool) -> Mods {
        Mods { ctrl, alt, shift, win: false }
    }

    fn key(vk: u16, down: bool, mods: Mods) -> KeyInput {
        KeyInput { vk, down, mods, caps_lock: false }
    }

    /// Writes `s` into a fresh temp dir and returns it; the session loads it on construction.
    fn dir_with(name: &str, s: Settings) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("evkey-core-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        settings::save(&dir, &Loaded { settings: s, macros: BTreeMap::new() }).unwrap();
        dir
    }

    fn with_hotkey(hk: &str) -> Settings {
        Settings { toggle_hotkey: hk.into(), ..Settings::default() }
    }

    #[test]
    fn modifier_chord_toggles_on_release() {
        let dir = dir_with("chord", with_hotkey("ctrl+shift"));
        let mut s = Session::new(Some(dir.clone()));
        s.on_key(key(CTRL, true, mods(true, false, false)));
        s.on_key(key(SHIFT, true, mods(true, false, true)));
        assert!(!s.is_english(), "toggle must wait for release");
        let r = s.on_key(key(SHIFT, false, mods(true, false, false)));
        assert!(r.hotkey_toggled && s.is_english());
        assert!(!r.swallow, "modifiers are never swallowed");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn modifier_chord_does_not_toggle_when_a_key_came_between() {
        let dir = dir_with("chord-key", with_hotkey("ctrl+shift"));
        let mut s = Session::new(Some(dir.clone()));
        s.on_key(key(CTRL, true, mods(true, false, false)));
        s.on_key(key(SHIFT, true, mods(true, false, true)));
        s.on_key(key(A, true, mods(true, false, true)));
        let r = s.on_key(key(SHIFT, false, mods(true, false, false)));
        assert!(!r.hotkey_toggled && !s.is_english());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn alt_z_hotkey_swallows_and_requests_dummy_key() {
        let dir = dir_with("alt-z", with_hotkey("alt+z"));
        let mut s = Session::new(Some(dir.clone()));
        s.on_key(key(ALT, true, mods(false, true, false)));
        let r = s.on_key(key(Z, true, mods(false, true, false)));
        assert!(r.swallow && r.hotkey_toggled && r.dummy_key);
        assert!(s.is_english());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn english_mode_passes_keys_through() {
        let mut s = Session::new(None);
        s.set_english(true);
        assert_eq!(s.on_key(key(A, true, mods(false, false, false))), Reaction::default());
    }

    #[test]
    fn excluded_app_passes_keys_through() {
        let app = "example.app";
        let excluded = Settings {
            excluded_apps: vec![app.into()],
            excluded_apps_macos: vec![app.into()],
            ..Settings::default()
        };
        let dir = dir_with("excluded", excluded);
        let mut s = Session::new(Some(dir.clone()));
        s.set_foreground(Some(app));
        assert!(!s.on_key(key(A, true, mods(false, false, false))).swallow);
        assert!(!s.on_key(key(A, true, mods(false, false, false))).swallow, "excluded app: second vowel not converted");
        s.set_foreground(Some("other.app"));
        s.on_key(key(A, true, mods(false, false, false)));
        assert!(s.on_key(key(A, true, mods(false, false, false))).swallow, "other app: second vowel converted to â");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn secure_input_passes_keys_through() {
        let mut s = Session::new(None);
        s.on_key(key(A, true, mods(false, false, false)));
        s.set_secure(true);
        assert!(!s.on_key(key(A, true, mods(false, false, false))).swallow);
        s.set_secure(false);
        assert!(s.on_key(key(A, true, mods(false, false, false))).swallow);
    }

    #[test]
    fn temp_off_tap_leaves_rest_of_word_unconverted() {
        let dir = dir_with("temp-off", Settings { temp_off_key: TempOffKey::Ctrl, ..Settings::default() });
        let mut s = Session::new(Some(dir.clone()));
        s.on_key(key(A, true, mods(false, false, false)));
        s.on_key(key(CTRL, true, mods(true, false, false)));
        s.on_key(key(CTRL, false, mods(false, false, false)));
        assert!(!s.on_key(key(A, true, mods(false, false, false))).swallow);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn settings_reload_applies_hotkey_now_and_other_settings_after_the_word() {
        let dir = dir_with("reload", with_hotkey("ctrl+shift"));
        let mut s = Session::new(Some(dir.clone()));
        s.on_key(key(A, true, mods(false, false, false)));
        // Change the hotkey and input method while "a" is being typed.
        settings::save(
            &dir,
            &Loaded { settings: Settings { toggle_hotkey: "alt+x".into(), input_type: settings::InputKind::Vni, ..Settings::default() }, macros: BTreeMap::new() },
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(1100));
        s.on_key(key(A, true, mods(false, false, false)));
        assert_eq!(s.settings().input_type, settings::InputKind::Telex, "deferred mid-word");
        assert_eq!(s.settings().toggle_hotkey, "ctrl+shift", "deferred mid-word");
        // The hotkey is applied at once, even mid-word; the rest waits.
        s.on_key(key(ALT, true, mods(false, true, false)));
        assert!(s.on_key(key(X, true, mods(false, true, false))).hotkey_toggled, "hotkey applies immediately");
        s.on_key(key(0x20, true, mods(false, false, false)));
        std::thread::sleep(Duration::from_millis(1100));
        s.on_key(key(A, true, mods(false, false, false)));
        assert_eq!(s.settings().input_type, settings::InputKind::Vni);
        let _ = std::fs::remove_dir_all(dir);
    }

    const S: u16 = 0x53;

    /// Presses `vk` and applies the reaction to a fake text field the way a host does; `native` is the typed char.
    fn press(s: &mut Session, field: &mut String, vk: u16, native: char) {
        let r = s.on_key(key(vk, true, mods(false, false, false)));
        match r.edit {
            None if !r.swallow => field.push(native),
            None => {}
            Some(out) => {
                // The empty char is typed first and deleted by the first backspace.
                for _ in 0..out.backspaces - usize::from(out.prefix_empty_char) {
                    field.pop();
                }
                field.push_str(&out.text);
                if out.resend.is_some() {
                    field.push(native);
                }
            }
        }
    }

    #[test]
    fn esc_restores_raw_keys_and_stops_conversion_for_the_word() {
        let mut s = Session::new(None);
        let mut field = String::new();
        for ch in "vieejt".chars() {
            press(&mut s, &mut field, ch.to_ascii_uppercase() as u16, ch);
        }
        assert_eq!(field, "việt");
        press(&mut s, &mut field, KEY_ESC, '\u{1b}');
        assert_eq!(field, "vieejt", "Esc shows the raw typed keys");
        press(&mut s, &mut field, S, 's');
        assert_eq!(field, "vieejts", "the rest of the word is not converted");
    }

    #[test]
    fn esc_passes_through_when_no_word_is_composing() {
        let mut s = Session::new(None);
        let r = s.on_key(key(KEY_ESC, true, mods(false, false, false)));
        assert!(!r.swallow && r.edit.is_none());
    }

    #[test]
    fn app_mode_is_remembered_per_app_and_survives_restart() {
        let dir = dir_with("app-mode", Settings { remember_app_mode: true, ..Settings::default() });
        let mut s = Session::new(Some(dir.clone()));
        s.set_foreground(Some("notepad.exe"));
        s.set_english(true);
        s.set_foreground(Some("code.exe"));
        assert!(s.is_english(), "an app with no record keeps the current mode");
        s.set_english(false);
        s.set_foreground(Some("notepad.exe"));
        assert!(s.is_english(), "notepad comes back in English");
        s.set_foreground(Some("code.exe"));
        assert!(!s.is_english(), "code comes back in Vietnamese");
        let mut again = Session::new(Some(dir.clone()));
        again.set_foreground(Some("notepad.exe"));
        assert!(again.is_english(), "the record survives a restart");
        let _ = std::fs::remove_dir_all(dir);
    }
}
