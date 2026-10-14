//! Per-key decision for letters and mark keys with the modern/old input rules (`handleMainKey`).

use super::*;

impl Engine {
    pub(super) fn handle_main_key(&mut self, data: u16, is_caps: bool) {
        let t = tables();

        // if is Z key, remove mark
        if self.is_key_z(data) {
            self.remove_mark();
            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        if data == KEY_LEFT_BRACKET {
            // standalone key [
            self.check_for_standalone_char(data, is_caps, KEY_O);
            return;
        }

        if data == KEY_RIGHT_BRACKET {
            // standalone key }
            self.check_for_standalone_char(data, is_caps, KEY_U);
            return;
        }

        // if is D key
        if self.is_key_d(data) {
            self.is_corect = false;
            self.is_changed = false;
            for i in 0..t.consonant_d.len() {
                if (self.index as usize) < t.consonant_d[i].len() {
                    continue;
                }
                self.is_corect = true;
                self.check_correct_vowel(&t.consonant_d, i, data);

                // allow d after consonant
                if !self.is_corect
                    && self.index - 2 >= 0
                    && self.chr(self.index - 1) == KEY_D
                    && is_consonant(self.chr(self.index - 2))
                {
                    self.is_corect = true;
                }
                if self.is_corect {
                    self.is_changed = true;
                    self.insert_d();
                    break;
                }
            }

            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        // if is mark key
        if self.is_mark_key(data) {
            for charset in t.vowel_for_mark.values() {
                self.is_corect = false;
                self.is_changed = false;
                for l in 0..charset.len() {
                    if (self.index as usize) < charset[l].len() {
                        continue;
                    }
                    self.is_corect = true;
                    self.check_correct_vowel(charset, l, data);

                    if self.is_corect {
                        self.is_changed = true;
                        if self.is_key_s(data) {
                            self.insert_mark(MARK1_MASK, true);
                        } else if self.is_key_f(data) {
                            self.insert_mark(MARK2_MASK, true);
                        } else if self.is_key_r(data) {
                            self.insert_mark(MARK3_MASK, true);
                        } else if self.is_key_x(data) {
                            self.insert_mark(MARK4_MASK, true);
                        } else if self.is_key_j(data) {
                            self.insert_mark(MARK5_MASK, true);
                        }
                        break;
                    }
                }

                if self.is_corect {
                    break;
                }
            }

            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        // check Vowel
        if self.is_vni() {
            let mut i = self.index - 1;
            while i >= 0 {
                let c = self.chr(i);
                if c == KEY_O || c == KEY_A || c == KEY_E {
                    self.vei = i;
                    break;
                }
                i -= 1;
            }
        }

        let key_for_aeo: u16 = if !self.is_vni() {
            data
        } else if data == KEY_7 || data == KEY_8 {
            KEY_W
        } else if data == KEY_6 {
            self.tw(self.vei) as u16
        } else {
            data
        };
        let empty = Vec::new();
        let charset = t.vowel.get(&key_for_aeo).unwrap_or(&empty);
        self.is_corect = false;
        self.is_changed = false;
        for i in 0..charset.len() {
            if (self.index as usize) < charset[i].len() {
                continue;
            }
            self.is_corect = true;
            self.check_correct_vowel(charset, i, data);

            if self.is_corect {
                self.is_changed = true;
                if self.is_key_double(data) {
                    self.insert_aoe(key_for_aeo);
                } else if self.is_key_w(data) {
                    if self.is_vni() {
                        let mut j = self.index - 1;
                        while j >= 0 {
                            let c = self.chr(j);
                            if c == KEY_O || c == KEY_U || c == KEY_A || c == KEY_E {
                                self.vei = j;
                                break;
                            }
                            j -= 1;
                        }
                        if (data == KEY_7
                            && self.chr(self.vei) == KEY_A
                            && (if self.vei - 1 >= 0 { self.chr(self.vei - 1) != KEY_U } else { true }))
                            || (data == KEY_8 && (self.chr(self.vei) == KEY_O || self.chr(self.vei) == KEY_U))
                        {
                            break;
                        }
                    }
                    self.insert_w();
                }
                break;
            }
        }

        if !self.is_changed {
            if data == KEY_W && self.cfg.input_type != V_SIMPLE_TELEX1 {
                self.check_for_standalone_char(data, is_caps, KEY_U);
            } else {
                self.insert_key(data, is_caps, true);
            }
        }
    }
}
