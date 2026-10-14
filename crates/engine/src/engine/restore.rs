//! Restore raw typed keys for invalid words and Esc (`checkRestoreIfWrongSpelling`, `restoreRawWord`), and the
//! EVKey-only English word restore.

use std::collections::HashSet;
use std::sync::LazyLock;

use super::*;

/// Common English words Telex changes into a rare Vietnamese syllable (regenerate with the `gen_english_words`
/// example of `evkey-core`). Lowercase, one per line.
static ENGLISH_WORDS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| include_str!("../../data/english_words.txt").lines().filter(|l| !l.is_empty()).collect());

impl Engine {
    pub(super) fn check_restore_if_wrong_spelling(&mut self, handle_code: u8) -> bool {
        for ii in 0..self.index {
            if !is_consonant(self.chr(ii))
                && (self.tw(ii) & MARK_MASK != 0 || self.tw(ii) & TONE_MASK != 0 || self.tw(ii) & TONEW_MASK != 0)
            {
                self.restore_raw_word(handle_code);
                return true;
            }
        }
        false
    }

    /// Rebuilds the word from the raw typed keys (`key_states`) and tells the host to show them.
    pub(super) fn restore_raw_word(&mut self, handle_code: u8) {
        self.hook.code = handle_code;
        self.set_bpc(self.index);
        self.set_ncc(self.state_index);
        for i in 0..self.state_index {
            let v = self.key_states[i as usize];
            self.set_tw(i, v);
            self.set_hdata(self.state_index - 1 - i, v);
        }
        self.index = self.state_index;
    }

    /// `restore_english`: when the keys of the word just ended spell a listed English word and Telex changed it,
    /// shows the keys as typed. Returns false (nothing done) otherwise. `undoable`: the ending key is re-sent as one
    /// typed character, so a Backspace right after can bring the Telex form back (`undo_english_restore`).
    pub(super) fn restore_english_word(&mut self, handle_code: u8, undoable: bool) -> bool {
        let n = self.raw_len;
        if self.cfg.restore_english == 0 || !(2..=RAW_MAX).contains(&n) || self.index == 0 || !self.long_word_helper.is_empty() {
            return false;
        }
        let mut buf = [0u8; RAW_MAX];
        for (b, &k) in buf.iter_mut().zip(&self.raw_keys[..n]) {
            let key = k & !CAPS_MASK;
            if !(KEY_A as u32..=KEY_Z as u32).contains(&key) {
                return false;
            }
            *b = (key as u8).to_ascii_lowercase();
        }
        let word = std::str::from_utf8(&buf[..n]).unwrap_or("");
        let unchanged = self.index as usize == n && (0..n).all(|i| self.tw(i as i32) == self.raw_keys[i]);
        let listed = |list: &[String]| list.iter().any(|w| w == word);
        if unchanged
            || listed(&self.cfg.english_words_excluded)
            || !(ENGLISH_WORDS.contains(word) || listed(&self.cfg.english_words_extra))
        {
            return false;
        }
        let shown = self.index as usize;
        if undoable && shown <= RAW_MAX {
            for i in 0..shown {
                self.undo_restore[i] = self.get(self.tw(i as i32));
            }
            self.undo_restore_len = shown;
            self.undo_restore_raw = n;
        }
        self.hook.code = handle_code;
        self.set_bpc(self.index);
        self.set_ncc(n as i32);
        for i in 0..n {
            let v = self.raw_keys[i];
            self.set_tw(i as i32, v);
            self.set_hdata((n - 1 - i) as i32, v);
        }
        self.index = n as i32;
        true
    }

    /// Backspace right after `restore_english` put an English word back: deletes it and the key that ended it, and
    /// types the Telex form again (`ứa` for `was`). The word is then finished. False when nothing was just restored.
    pub fn undo_english_restore(&mut self) -> bool {
        let n = std::mem::take(&mut self.undo_restore_len);
        if n == 0 {
            return false;
        }
        // Finish the word first: starting a new session clears the hook.
        self.start_new_session();
        self.typing_states.clear();
        self.special_char.clear();
        self.space_count = 0;
        self.hook.code = V_WILL_PROCESS;
        self.set_bpc(self.undo_restore_raw as i32 + 1);
        self.set_ncc(n as i32);
        for i in 0..n {
            let v = self.undo_restore[i];
            self.set_hdata((n - 1 - i) as i32, v);
        }
        true
    }

    /// Hoàn tác từ (Esc): shows the raw typed keys of the word in progress and leaves the rest of it unconverted.
    /// Returns false when no word is in progress, so the key passes through.
    pub fn undo_word(&mut self) -> bool {
        if self.index == 0 {
            return false;
        }
        self.restore_raw_word(V_RESTORE);
        self.temp_off_engine(true);
        true
    }
}
