//! Sentence-start capitalization (`upperCaseFirstCharacter`).

use super::*;

impl Engine {
    pub(super) fn upper_case_first_character(&mut self) {
        if self.typing_word[0] & CAPS_MASK == 0 {
            self.hook.code = V_WILL_PROCESS;
            self.set_bpc(0);
            self.set_ncc(1);
            self.typing_word[0] |= CAPS_MASK;
            let v = self.get(self.typing_word[0]);
            self.set_hdata(0, v);
            self.upper_case_status = 0;
            if self.cfg.use_macro != 0 {
                if let Some(k) = self.hook.macro_key.first_mut() {
                    *k |= CAPS_MASK;
                }
            }
        }
    }
}
