//! Thin owner of an [`Engine`] for a typing session: builds it from settings and macros, and forgets the word in
//! progress when the host loses track of the caret.

use crate::consts::*;
use crate::engine::*;
use crate::macros::MacroMap;
use crate::settings::Loaded;
use crate::tables::{key_code_to_character, tables};

/// Render engine words (already passed through `get_character_code`) like OpenKey's Windows sender.
pub fn render(codes: impl IntoIterator<Item = u32>, code_table: i32) -> String {
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

pub struct Composer {
    pub engine: Engine,
}

impl Composer {
    pub fn new(cfg: Config) -> Self {
        let mut engine = Engine::new();
        engine.cfg = cfg;
        engine.init();
        Composer { engine }
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
        !self.engine.typing_word().is_empty()
    }

    /// The host lost the composition (focus change, mouse click): forget the word.
    pub fn reset(&mut self) {
        self.engine.key_event(true, 0, 0, false);
        self.engine.finish_word();
    }
}
