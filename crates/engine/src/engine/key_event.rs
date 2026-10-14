//! Entry points: `vEnglishMode`, `vKeyHandleEvent` and the temp-off flags.

use super::*;

impl Engine {
    /// `vTempOffSpellChecking`.
    pub fn temp_off_spell_checking(&mut self) {
        if self.use_spell_checking_before {
            self.cfg.check_spelling = if self.cfg.check_spelling != 0 { 0 } else { 1 };
        }
    }

    /// `vSetCheckSpelling`.
    pub fn set_check_spelling(&mut self) {
        self.use_spell_checking_before = self.cfg.check_spelling != 0;
    }

    /// `vTempOffEngine`.
    pub fn temp_off_engine(&mut self, off: bool) {
        self.will_temp_off_engine = off;
    }

    /// `vEnglishMode`.
    pub fn english_mode(&mut self, is_mouse_down: bool, data: u16, is_caps: bool, other_control_key: bool) {
        self.hook.code = V_DO_NOTHING;
        if is_mouse_down || (other_control_key && !is_caps) {
            self.hook.macro_key.clear();
            self.will_temp_off_engine = false;
        } else if data == KEY_SPACE {
            if !self.has_handled_macro && self.find_macro() {
                self.hook.code = V_REPLACE_MACRO;
                self.hook.backspace_count = self.hook.macro_key.len() as u8;
            }
            self.hook.macro_key.clear();
            self.will_temp_off_engine = false;
        } else if data == KEY_DELETE {
            if !self.hook.macro_key.is_empty() {
                self.hook.macro_key.pop();
            } else {
                self.will_temp_off_engine = false;
            }
        } else if self.is_word_break(false, data) && !CHAR_KEY_CODE.contains(&data) {
            self.hook.macro_key.clear();
            self.will_temp_off_engine = false;
        } else if !self.will_temp_off_engine {
            self.hook.macro_key.push(data as u32 | if is_caps { CAPS_MASK } else { 0 });
        }
    }

    /// `vKeyHandleEvent` for a keyboard key-down (`is_mouse = false`) or a mouse event.
    pub fn key_event(&mut self, is_mouse: bool, data: u16, caps_status: u8, other_control_key: bool) {
        self.ended_word_len = -1;
        self.undo_restore_len = 0;
        self.is_caps = caps_status == 1 || caps_status == 2;
        let is_caps = self.is_caps;
        if (is_number_key(data) && caps_status == 1)
            || other_control_key
            || self.is_word_break(is_mouse, data)
            || (self.index == 0 && is_number_key(data))
        {
            self.hook.code = V_DO_NOTHING;
            self.set_bpc(0);
            self.set_ncc(0);
            self.hook.ext_code = 1; // word break

            // check macro feature
            if self.cfg.use_macro != 0
                && self.is_macro_break_code(data)
                && !self.has_handled_macro
                && self.find_macro()
            {
                self.hook.code = V_REPLACE_MACRO;
                self.hook.backspace_count = self.hook.macro_key.len() as u8;
                self.has_handled_macro = true;
            } else if (self.cfg.quick_start_consonant != 0 || self.cfg.quick_end_consonant != 0)
                && !self.temp_disable_key
                && self.is_macro_break_code(data)
            {
                self.check_quick_consonant();
            } else if self.cfg.restore_if_wrong_spelling != 0 && self.is_word_break(is_mouse, data) {
                // restore key if wrong spelling with break-key
                if !self.temp_disable_key && self.cfg.check_spelling != 0 {
                    self.check_spelling(true); // force check spelling
                }
                if self.temp_disable_key && !self.check_restore_if_wrong_spelling(V_RESTORE_AND_START_NEW_SESSION) {
                    self.hook.code = V_DO_NOTHING;
                }
            }
            // EVKey addition (off by default, so OpenKey parity holds): a mouse click moved the caret, never retype then.
            if self.hook.code == V_DO_NOTHING && !is_mouse && !other_control_key && !self.has_handled_macro {
                // Punctuation comes back as one typed character; Enter, Tab or arrows do not, so no undo for them.
                self.restore_english_word(V_RESTORE_AND_START_NEW_SESSION, CHAR_KEY_CODE.contains(&data));
            }

            let is_char_key_code = !is_mouse && CHAR_KEY_CODE.contains(&data);
            if !is_char_key_code {
                // clear all line cache
                self.special_char.clear();
                self.typing_states.clear();
            } else {
                // check and save current word
                if self.space_count > 0 {
                    self.save_word_repeat(KEY_SPACE as u32, self.space_count);
                    self.space_count = 0;
                } else {
                    self.save_word();
                }
                self.special_char.push(data as u32 | if is_caps { CAPS_MASK } else { 0 });
                self.hook.ext_code = 3; // normal word
            }

            if self.hook.code == V_DO_NOTHING {
                self.start_new_session();
                self.cfg.check_spelling = self.use_spell_checking_before as i32;
                self.will_temp_off_engine = false;
            } else if self.hook.code == V_REPLACE_MACRO || self.has_handle_quick_consonant {
                self.ended_word_len = self.index;
                self.index = 0;
            }

            // insert key for macro function
            if self.cfg.use_macro != 0 {
                if is_char_key_code {
                    self.hook.macro_key.push(data as u32 | if is_caps { CAPS_MASK } else { 0 });
                } else {
                    self.hook.macro_key.clear();
                }
            }

            if self.cfg.upper_case_first_char != 0 {
                // A sentence ends at `.`, `?` (Shift+/) or `!` (Shift+1), and the next word is capitalized after a
                // space; a new line (Enter) starts a sentence at once. Each case can be switched off.
                let shift = caps_status == 1;
                if (data == KEY_DOT && self.cfg.upper_case_after_period != 0)
                    || (shift && data == KEY_SLASH && self.cfg.upper_case_after_question != 0)
                    || (shift && data == KEY_1 && self.cfg.upper_case_after_exclaim != 0)
                {
                    self.upper_case_status = 1;
                } else if (data == KEY_ENTER || data == KEY_RETURN) && self.cfg.upper_case_after_enter != 0 {
                    self.upper_case_status = 2;
                } else {
                    self.upper_case_status = 0;
                }
            }
        } else if data == KEY_SPACE {
            if !self.temp_disable_key && self.cfg.check_spelling != 0 {
                self.check_spelling(true); // force check spelling
            }
            if self.cfg.use_macro != 0 && !self.has_handled_macro && self.find_macro() {
                // macro
                self.hook.code = V_REPLACE_MACRO;
                self.hook.backspace_count = self.hook.macro_key.len() as u8;
                self.space_count += 1;
                self.has_handled_macro = true;
            } else if (self.cfg.quick_start_consonant != 0 || self.cfg.quick_end_consonant != 0)
                && !self.temp_disable_key
                && self.check_quick_consonant()
            {
                self.space_count += 1;
            } else if self.cfg.restore_if_wrong_spelling != 0 && self.temp_disable_key && !self.has_handled_macro {
                // restore key if wrong spelling
                if !self.check_restore_if_wrong_spelling(V_RESTORE) {
                    self.hook.code = V_DO_NOTHING;
                }
                self.space_count += 1;
            } else {
                // do nothing with SPACE KEY
                self.hook.code = V_DO_NOTHING;
                self.space_count += 1;
            }
            if self.hook.code == V_DO_NOTHING && !self.has_handled_macro {
                self.restore_english_word(V_RESTORE, true);
            }
            if self.cfg.use_macro != 0 {
                self.hook.macro_key.clear();
            }
            if self.cfg.upper_case_first_char != 0 && self.upper_case_status == 1 {
                self.upper_case_status = 2;
            }
            // save word
            if self.space_count == 1 {
                if !self.special_char.is_empty() {
                    self.save_special_char();
                } else {
                    self.save_word();
                }
            }
            self.cfg.check_spelling = self.use_spell_checking_before as i32;
            self.will_temp_off_engine = false;
        } else if data == KEY_DELETE {
            self.hook.code = V_DO_NOTHING;
            self.hook.ext_code = 2; // delete
            if !self.special_char.is_empty() {
                self.special_char.pop();
                if self.special_char.is_empty() {
                    self.restore_last_typing_state();
                }
            } else if self.space_count > 0 {
                // previous char is space
                self.space_count -= 1;
                if self.space_count == 0 {
                    // restore word
                    self.restore_last_typing_state();
                }
            } else {
                if self.state_index > 0 {
                    self.state_index -= 1;
                }
                // An edited word no longer matches what was typed, so restore_english leaves it alone.
                self.raw_len = RAW_MAX + 1;
                if self.index > 0 {
                    self.index -= 1;
                    if let Some(back) = self.long_word_helper.pop() {
                        // right shift
                        self.typing_word.copy_within(0..MAX_BUFF - 1, 1);
                        self.typing_word[0] = back;
                        self.index += 1;
                    }
                    if self.cfg.check_spelling != 0 {
                        self.check_spelling(false);
                    }
                }
                if self.cfg.use_macro != 0 && !self.hook.macro_key.is_empty() {
                    self.hook.macro_key.pop();
                }

                self.set_bpc(0);
                self.set_ncc(0);
                self.hook.ext_code = 2; // delete key
                if self.index == 0 {
                    self.start_new_session();
                    self.special_char.clear();
                    self.restore_last_typing_state();
                } else {
                    // continue check grammar
                    self.check_grammar(1);
                }
            }
        } else {
            // START AND CHECK KEY
            if self.will_temp_off_engine {
                self.hook.code = V_DO_NOTHING;
                self.hook.ext_code = 3;
                return;
            }
            if self.space_count > 0 {
                self.set_bpc(0);
                self.set_ncc(0);
                self.hook.ext_code = 0;
                self.start_new_session();
                // continue save space
                self.save_word_repeat(KEY_SPACE as u32, self.space_count);
                self.space_count = 0;
            } else if !self.special_char.is_empty() {
                self.save_special_char();
            }

            self.insert_state(data, is_caps); // save state

            if !self.is_special_key(data) || self.temp_disable_key {
                // do nothing
                if self.cfg.quick_telex != 0 && self.is_quick_telex_key(data) {
                    self.handle_quick_telex(data, is_caps);
                    return;
                } else {
                    self.hook.code = V_DO_NOTHING;
                    self.set_bpc(0);
                    self.set_ncc(0);
                    self.hook.ext_code = 3; // normal key
                    self.insert_key(data, is_caps, true);
                }
            } else {
                // check and update key
                self.hook.code = V_DO_NOTHING;
                self.hook.ext_code = 3; // normal key
                self.handle_main_key(data, is_caps);
            }

            if self.cfg.free_mark == 0 && !self.is_key_d(data) {
                if self.hook.code == V_DO_NOTHING {
                    self.check_grammar(-1);
                } else {
                    self.check_grammar(0);
                }
            }

            if self.hook.code == V_RESTORE {
                self.insert_key(data, is_caps, true);
                self.state_index -= 1;
            }

            // insert or replace key for macro feature
            if self.cfg.use_macro != 0 {
                if self.hook.code == V_DO_NOTHING {
                    self.hook.macro_key.push(data as u32 | if is_caps { CAPS_MASK } else { 0 });
                } else if self.hook.code == V_WILL_PROCESS || self.hook.code == V_RESTORE {
                    for _ in 0..self.bpc() {
                        self.hook.macro_key.pop();
                    }
                    let start = self.index - self.bpc();
                    let end = self.hook.new_char_count as i32 + start;
                    let mut i = start;
                    while i < end {
                        let w = self.tw(i);
                        self.hook.macro_key.push(w);
                        i += 1;
                    }
                }
            }

            if self.cfg.upper_case_first_char != 0 {
                if self.index == 1 && self.upper_case_status == 2 {
                    self.upper_case_first_character();
                }
                self.upper_case_status = 0;
            }

            // case [ ]
            if is_bracket_key(data)
                && (is_bracket_key(self.hook.char_data[0] as u16)
                    || self.cfg.input_type == V_SIMPLE_TELEX1
                    || self.cfg.input_type == V_SIMPLE_TELEX2)
            {
                let sub = if self.hook.code == V_WILL_PROCESS { self.bpc() } else { 0 };
                if self.index - sub > 0 {
                    self.index -= 1;
                    self.save_word();
                }
                self.index = 0;
                self.temp_disable_key = false;
                self.state_index = 0;
                self.raw_len = 0;
                self.hook.ext_code = 3;
                self.special_char.push(data as u32 | if is_caps { CAPS_MASK } else { 0 });
            }
        }
    }
}
