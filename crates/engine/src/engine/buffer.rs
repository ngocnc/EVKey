//! Typed-word buffer maintenance: key insertion, word history for backspace/restore, session reset.

use super::*;

impl Engine {
    // ---- buffer ---------------------------------------------------------------------------------

    pub(super) fn set_key_data(&mut self, index: i32, key_code: u16, is_caps: bool) {
        if !(0..MAX_BUFF as i32).contains(&index) {
            return;
        }
        self.typing_word[index as usize] = key_code as u32 | if is_caps { CAPS_MASK } else { 0 };
    }

    pub(super) fn insert_key(&mut self, key_code: u16, is_caps: bool, check_spelling: bool) {
        if self.index >= MAX_BUFF as i32 {
            self.long_word_helper.push(self.typing_word[0]); // save long word
            self.typing_word.copy_within(1..MAX_BUFF, 0); // left shift
            self.set_key_data(self.index - 1, key_code, is_caps);
        } else {
            self.set_key_data(self.index, key_code, is_caps);
            self.index += 1;
        }

        if self.cfg.check_spelling != 0 && check_spelling {
            self.check_spelling(false);
        }

        // allow d after consonant
        if key_code == KEY_D && self.index - 2 >= 0 && is_consonant(self.chr(self.index - 2)) {
            self.temp_disable_key = false;
        }
    }

    pub(super) fn insert_state(&mut self, key_code: u16, is_caps: bool) {
        let v = key_code as u32 | if is_caps { CAPS_MASK } else { 0 };
        if self.state_index >= MAX_BUFF as i32 {
            self.key_states.copy_within(1..MAX_BUFF, 0);
            self.key_states[(self.state_index - 1) as usize] = v;
        } else {
            self.key_states[self.state_index as usize] = v;
            self.state_index += 1;
        }
    }

    pub(super) fn save_word(&mut self) {
        if self.hook.code != V_REPLACE_MACRO {
            if self.index > 0 {
                if !self.long_word_helper.is_empty() {
                    // save long word first
                    let mut data: Vec<u32> = Vec::new();
                    for (i, v) in self.long_word_helper.iter().enumerate() {
                        if i != 0 && i % MAX_BUFF == 0 {
                            self.typing_states.push_back(std::mem::take(&mut data));
                        }
                        data.push(*v);
                    }
                    self.typing_states.push_back(data);
                    self.long_word_helper.clear();
                }
                // save current word
                let data = self.typing_word[..self.index as usize].to_vec();
                self.typing_states.push_back(data);
            }
        } else {
            // save macro words
            let mut data: Vec<u32> = Vec::new();
            for (i, v) in self.hook.macro_data.iter().enumerate() {
                if i != 0 && i % MAX_BUFF == 0 {
                    self.typing_states.push_back(std::mem::take(&mut data));
                }
                data.push(*v);
            }
            self.typing_states.push_back(data);
        }
    }

    pub(super) fn save_word_repeat(&mut self, key_code: u32, count: i32) {
        self.typing_states.push_back(vec![key_code; count.max(0) as usize]);
    }

    pub(super) fn save_special_char(&mut self) {
        let data = std::mem::take(&mut self.special_char);
        self.typing_states.push_back(data);
    }

    pub(super) fn restore_last_typing_state(&mut self) {
        if let Some(data) = self.typing_states.pop_back() {
            if !data.is_empty() {
                if data[0] == KEY_SPACE as u32 {
                    self.space_count = data.len() as i32;
                    self.index = 0;
                } else if CHAR_KEY_CODE.contains(&(data[0] as u16)) {
                    self.index = 0;
                    self.special_char = data;
                    self.check_spelling(false);
                } else {
                    for (i, v) in data.iter().enumerate() {
                        if i < MAX_BUFF {
                            self.typing_word[i] = *v;
                        }
                    }
                    self.index = data.len() as i32 as u8 as i32;
                }
            }
        }
    }

    /// `startNewSession`.
    /// The host lost the caret context (focus change, click): with `upper_case_on_reset` the next letter is capitalized.
    pub fn mark_sentence_start(&mut self) {
        if self.cfg.upper_case_first_char != 0 && self.cfg.upper_case_on_reset != 0 {
            self.upper_case_status = 2;
        }
    }

    pub fn start_new_session(&mut self) {
        self.index = 0;
        self.set_bpc(0);
        self.set_ncc(0);
        self.temp_disable_key = false;
        self.state_index = 0;
        self.has_handled_macro = false;
        self.has_handle_quick_consonant = false;
        self.long_word_helper.clear();
    }
}
