//! Restore raw typed keys for invalid words and Esc (`checkRestoreIfWrongSpelling`, `restoreRawWord`).

use super::*;

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
