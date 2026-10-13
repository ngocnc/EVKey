//! IME-facing layer over [`Engine`]: turns key events into "set composition text / commit" actions.
//!
//! The hook-style engine output (backspace count + new chars) is not used here; the composition
//! is always re-rendered from the engine's word buffer, so the host never edits committed text.

use crate::consts::*;
use crate::engine::*;
use crate::keys::*;
use crate::macros::MacroMap;
use crate::settings::Loaded;
use crate::tables::{get_character_code, key_code_to_character, tables};

/// What the host (TSF text service) must do for one key-down.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Not ours: leave the key and the composition alone.
    Pass,
    /// Eat the key and show `text` as the (non-empty) composition.
    Compose(String),
    /// Replace the composition with `text`, end it, and eat the key only if `eaten`.
    Commit { text: String, eaten: bool },
}

/// Render engine words (already passed through `get_character_code`) like OpenKey's Windows sender.
fn render(codes: impl IntoIterator<Item = u32>, code_table: i32) -> String {
    let mut out: Vec<u16> = Vec::new();
    for c in codes {
        if c & PURE_CHARACTER_MASK != 0 {
            out.push(c as u16);
        } else if c & CHAR_CODE_MASK == 0 {
            // raw key
            let ch = key_code_to_character(c);
            out.push(if ch == 0 { c as u16 } else { ch });
        } else {
            match code_table {
                1 | 2 | 4 => {
                    // TCVN3 / VNI Windows / CP1258: one or two single-byte codes
                    out.push((c & 0xFF) as u16);
                    let hi = ((c >> 8) & 0xFF) as u16;
                    if hi > 32 {
                        out.push(hi);
                    }
                }
                3 => {
                    // Unicode compound: base letter + combining mark
                    let v = c as u16;
                    let hi = (v >> 13) as usize;
                    out.push(v & 0x1FFF);
                    if hi > 0 {
                        out.push(tables().unicode_compound_mark[hi - 1]);
                    }
                }
                _ => out.push(c as u16),
            }
        }
    }
    String::from_utf16_lossy(&out)
}

pub fn is_modifier_key(vk: u16) -> bool {
    matches!(vk, 0x10..=0x12 | 0x14 | 0x5B | 0x5C | 0x90 | 0x91 | 0xA0..=0xA5)
}

/// Keys that extend a word: letters, top-row digits, and the `[` `]` keys.
fn is_word_key(vk: u16) -> bool {
    (KEY_A..=KEY_Z).contains(&vk) || (KEY_0..=KEY_9).contains(&vk) || vk == KEY_LEFT_BRACKET || vk == KEY_RIGHT_BRACKET
}

/// The character a word key types when the engine does not transform it.
fn raw_char(vk: u16, caps: u8) -> char {
    let ch = key_code_to_character(vk as u32 | if caps != 0 { CAPS_MASK } else { 0 });
    char::from_u32(ch as u32).filter(|&c| c != '\0').unwrap_or(vk as u8 as char)
}

pub struct Composer {
    pub engine: Engine,
    /// "Temporarily off" for the rest of this word: keys are appended to `tail` untouched.
    temp_off: bool,
    tail: String,
}

impl Composer {
    pub fn new(cfg: Config) -> Self {
        let mut engine = Engine::new();
        engine.cfg = cfg;
        engine.init();
        Composer { engine, temp_off: false, tail: String::new() }
    }

    /// Replace configuration and macros (e.g. after the settings files changed).
    pub fn apply(&mut self, loaded: &Loaded) {
        self.reset();
        let cfg = loaded.settings.to_config(!loaded.macros.is_empty());
        let table = cfg.code_table as usize;
        self.engine.cfg = cfg;
        self.engine.set_check_spelling();
        self.engine.macros = MacroMap::new();
        for (text, content) in &loaded.macros {
            self.engine.macros.add(text, content, table);
        }
    }

    pub fn is_composing(&self) -> bool {
        !self.engine.typing_word().is_empty() || !self.tail.is_empty()
    }

    /// Leave the rest of the current word untouched (typed keys are not converted until the word ends).
    pub fn temp_off(&mut self) {
        if self.is_composing() {
            self.temp_off = true;
        }
    }

    fn codes(&self, words: &[u32]) -> Vec<u32> {
        words.iter().map(|&w| get_character_code(self.engine.cfg.code_table as usize, w)).collect()
    }

    fn composition(&self) -> String {
        let mut s = render(self.codes(self.engine.typing_word()), self.engine.cfg.code_table);
        s.push_str(&self.tail);
        s
    }

    fn end_word(&mut self) {
        self.engine.finish_word();
        self.tail.clear();
        self.temp_off = false;
    }

    /// Would `key_down` do anything for this key? Pure; backs `OnTestKeyDown`.
    pub fn wants_key(&self, vk: u16) -> bool {
        !is_modifier_key(vk) && (self.is_composing() || is_word_key(vk) && !(KEY_0..=KEY_9).contains(&vk))
    }

    /// The host lost the composition (focus change, mouse click): forget the word.
    pub fn reset(&mut self) {
        self.engine.key_event(true, 0, 0, false);
        self.end_word();
    }

    /// Process one key-down. `ctrl_alt` is true when Ctrl, Alt or Win is held.
    pub fn key_down(&mut self, vk: u16, shift: bool, caps_lock: bool, ctrl_alt: bool) -> Action {
        if is_modifier_key(vk) {
            return Action::Pass;
        }
        let caps: u8 = if shift && caps_lock { 0 } else if shift { 1 } else if caps_lock { 2 } else { 0 };
        let composing = self.is_composing();
        let digit = (KEY_0..=KEY_9).contains(&vk);
        let extends_word = is_word_key(vk) && !ctrl_alt && !(digit && (caps == 1 || !composing));

        if self.temp_off {
            if vk == KEY_DELETE && !ctrl_alt && self.tail.pop().is_some() {
                return Action::Compose(self.composition());
            }
            if extends_word {
                self.tail.push(raw_char(vk, caps));
                return Action::Compose(self.composition());
            }
        }

        if vk == KEY_DELETE && !ctrl_alt {
            if !composing {
                return Action::Pass;
            }
            self.engine.key_event(false, vk, caps, false);
            let text = self.composition();
            if text.is_empty() {
                self.end_word();
                return Action::Commit { text, eaten: true };
            }
            return Action::Compose(text);
        }

        if extends_word {
            let pre = self.composition();
            self.engine.key_event(false, vk, caps, false);
            if self.is_composing() {
                return Action::Compose(self.composition());
            }
            // The engine ended the word on a raw `[` / `]`: it stays as plain text after the word.
            let mut text = pre;
            text.push(raw_char(vk, caps));
            self.end_word();
            return Action::Commit { text, eaten: true };
        }

        // Anything else ends the word: space, punctuation, Enter, arrows, Ctrl/Alt chords, ...
        let pre = self.composition();
        self.engine.key_event(false, vk, caps, ctrl_alt);
        let code_table = self.engine.cfg.code_table;
        let text = if self.temp_off {
            pre
        } else {
            match self.engine.hook.code {
                V_REPLACE_MACRO => render(self.codes(&self.engine.hook.macro_data), code_table),
                V_RESTORE | V_RESTORE_AND_START_NEW_SESSION => {
                    render(self.codes(self.engine.word_after_event()), code_table)
                }
                _ => pre,
            }
        };
        self.end_word();
        if composing { Action::Commit { text, eaten: false } } else { Action::Pass }
    }
}
