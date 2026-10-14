//! Vowel location and tone-mark placement (`insertMark`, `removeMark`, modern/old orthography rules).

use super::*;

impl Engine {
    pub(super) fn check_correct_vowel(&mut self, charset: &[Vec<u16>], i: usize, mark_key: u16) {
        // ignore "qu" case
        if self.index >= 2 && self.chr(self.index - 1) == KEY_U && self.chr(self.index - 2) == KEY_Q {
            self.is_corect = false;
            return;
        }
        let m: u32 = if self.cfg.quick_end_consonant != 0 { END_CONSONANT_MASK } else { 0 };
        let mut k = self.index - 1;
        let row = &charset[i];
        let mut j = row.len() as i32 - 1;
        while j >= 0 {
            if (row[j as usize] as u32 & !m) != self.chr(k) as u32 {
                self.is_corect = false;
                return;
            }
            k -= 1;
            if k < 0 {
                break;
            }
            j -= 1;
        }

        // limit mark for end consonant: "C", "T"
        if self.is_corect
            && row.len() > 1
            && (self.is_key_f(mark_key) || self.is_key_x(mark_key) || self.is_key_r(mark_key))
        {
            if row[1] == KEY_C || row[1] == KEY_T {
                self.is_corect = false;
            } else if row.len() > 2 && row[2] == KEY_T {
                self.is_corect = false;
            }
        }

        if self.is_corect && k >= 0 && self.chr(k) == self.chr(k + 1) {
            self.is_corect = false;
        }
    }

    pub(super) fn find_and_calculate_vowel(&mut self, for_grammar: bool) {
        self.vowel_count = 0;
        self.vsi = 0;
        self.vei = 0;
        let mut i = self.index - 1;
        while i >= 0 {
            if is_consonant(self.chr(i)) {
                if self.vowel_count > 0 {
                    break;
                }
            } else {
                if self.vowel_count == 0 {
                    self.vei = i;
                }
                if !for_grammar
                    && ((i - 1 >= 0 && self.chr(i) == KEY_I && self.chr(i - 1) == KEY_G)
                        || (i - 1 >= 0 && self.chr(i) == KEY_U && self.chr(i - 1) == KEY_Q))
                {
                    break;
                }
                self.vsi = i;
                self.vowel_count += 1;
            }
            i -= 1;
        }
        // don't count "u" at "q u" as a vowel
        if self.vsi - 1 >= 0 && self.chr(self.vsi) == KEY_U && self.chr(self.vsi - 1) == KEY_Q {
            self.vsi += 1;
            self.vowel_count -= 1;
        }
    }

    pub(super) fn remove_mark(&mut self) {
        self.find_and_calculate_vowel(true);
        self.is_changed = false;
        if self.index > 0 {
            let mut i = self.vsi;
            while i <= self.vei {
                if self.tw(i) & MARK_MASK != 0 {
                    self.tw_update(i, |w| w & !MARK_MASK);
                    self.is_changed = true;
                }
                i += 1;
            }
        }
        if self.is_changed {
            self.hook.code = V_WILL_PROCESS;
            self.set_bpc(0);
            let mut i = self.index - 1;
            while i >= self.vsi {
                self.set_bpc(self.bpc() + 1);
                let v = self.get(self.tw(i));
                self.set_hdata(self.index - 1 - i, v);
                i -= 1;
            }
            self.hook.new_char_count = self.hook.backspace_count;
        } else {
            self.hook.code = V_DO_NOTHING;
        }
    }

    pub(super) fn can_has_end_consonant(&self) -> bool {
        let t = tables();
        let empty = Vec::new();
        let vo = t.vowel_combine.get(&self.chr(self.vsi)).unwrap_or(&empty);
        for row in vo.iter() {
            let mut kk = self.vsi;
            let mut iii = 1usize;
            while iii < row.len() {
                if kk > self.vei
                    || ((self.chr(kk) as u32 | (self.tw(kk) & TONE_MASK) | (self.tw(kk) & TONEW_MASK)) != row[iii])
                {
                    break;
                }
                kk += 1;
                iii += 1;
            }
            if iii >= row.len() {
                return row[0] == 1;
            }
        }
        false
    }

    pub(super) fn handle_modern_mark(&mut self) {
        let (vsi, vei, idx) = (self.vsi, self.vei, self.index);
        // default
        self.vwsm = vei;
        self.set_bpc(idx - vei);

        // rule 2
        let c = |s: &Self, i: i32| s.chr(i);
        if self.vowel_count == 3
            && ((c(self, vsi) == KEY_O && c(self, vsi + 1) == KEY_A && c(self, vsi + 2) == KEY_I)
                || (c(self, vsi) == KEY_U && c(self, vsi + 1) == KEY_Y && c(self, vsi + 2) == KEY_U)
                || (c(self, vsi) == KEY_O && c(self, vsi + 1) == KEY_E && c(self, vsi + 2) == KEY_O)
                || (c(self, vsi) == KEY_U && c(self, vsi + 1) == KEY_Y && c(self, vsi + 2) == KEY_A))
        {
            self.vwsm = vsi + 1;
            self.set_bpc(idx - self.vwsm);
        } else if (c(self, vsi) == KEY_O && c(self, vsi + 1) == KEY_I)
            || (c(self, vsi) == KEY_A && c(self, vsi + 1) == KEY_I)
            || (c(self, vsi) == KEY_U && c(self, vsi + 1) == KEY_I)
        {
            self.vwsm = vsi;
            self.set_bpc(idx - self.vwsm);
        } else if c(self, vei - 1) == KEY_A && c(self, vei) == KEY_Y {
            self.vwsm = vei - 1;
            self.set_bpc((idx - vei) + 1);
        } else if c(self, vsi) == KEY_U && c(self, vsi + 1) == KEY_O {
            self.vwsm = vsi + 1;
            self.set_bpc(idx - self.vwsm);
        } else if c(self, vsi + 1) == KEY_O || c(self, vsi + 1) == KEY_U {
            self.vwsm = vei - 1;
            self.set_bpc((idx - vei) + 1);
        } else if c(self, vsi) == KEY_O || c(self, vsi) == KEY_U {
            self.vwsm = vei;
            self.set_bpc(idx - vei);
        }

        // rule 3.1
        let t = |s: &Self, i: i32| s.tw(i);
        if (c(self, vsi) == KEY_I && (t(self, vsi + 1) & (KEY_E as u32 | TONE_MASK)) != 0)
            || (c(self, vsi) == KEY_Y && (t(self, vsi + 1) & (KEY_E as u32 | TONE_MASK)) != 0)
            || (c(self, vsi) == KEY_U && t(self, vsi + 1) == (KEY_O as u32 | TONE_MASK))
            || (t(self, vsi) == (KEY_U as u32 | TONEW_MASK) && t(self, vsi + 1) == (KEY_O as u32 | TONEW_MASK))
        {
            if vsi + 2 < idx {
                let c2 = c(self, vsi + 2);
                if c2 == KEY_P
                    || c2 == KEY_T
                    || c2 == KEY_M
                    || c2 == KEY_N
                    || c2 == KEY_O
                    || c2 == KEY_U
                    || c2 == KEY_I
                    || c2 == KEY_C
                    || (vsi + 3 < idx && c2 == KEY_C && c2 == KEY_H)
                    || (vsi + 3 < idx && c2 == KEY_N && c2 == KEY_H)
                    || (vsi + 3 < idx && c2 == KEY_N && c2 == KEY_G)
                {
                    self.vwsm = vsi + 1;
                    self.set_bpc(idx - self.vwsm);
                } else {
                    self.vwsm = vsi;
                    self.set_bpc(idx - self.vwsm);
                }
            } else {
                self.vwsm = vsi;
                self.set_bpc(idx - self.vwsm);
            }
        }
        // rule 3.2
        else if (c(self, vsi) == KEY_I && c(self, vsi) == KEY_A)
            || (c(self, vsi) == KEY_Y && c(self, vsi) == KEY_A)
            || (c(self, vsi) == KEY_U && c(self, vsi) == KEY_A)
            || (c(self, vsi) == KEY_U && t(self, vsi + 1) == (KEY_U as u32 | TONEW_MASK))
        {
            self.vwsm = vsi;
            self.set_bpc(idx - self.vwsm);
        }

        // rule 4
        if self.vowel_count == 2 {
            if (c(self, vsi) == KEY_I && c(self, vsi + 1) == KEY_A)
                || (c(self, vsi) == KEY_I && c(self, vsi + 1) == KEY_U)
                || (c(self, vsi) == KEY_I && c(self, vsi + 1) == KEY_O)
            {
                if vsi == 0 || c(self, vsi - 1) != KEY_G {
                    // dont have G
                    self.vwsm = vsi;
                    self.set_bpc(idx - self.vwsm);
                } else {
                    self.vwsm = vsi + 1;
                    self.set_bpc(idx - self.vwsm);
                }
            } else if c(self, vsi) == KEY_U && c(self, vsi + 1) == KEY_A {
                if vsi == 0 || c(self, vsi - 1) != KEY_Q {
                    // dont have Q
                    if vei + 1 >= idx || !self.can_has_end_consonant() {
                        self.vwsm = vsi;
                        self.set_bpc(idx - self.vwsm);
                    }
                } else {
                    self.vwsm = vsi + 1;
                    self.set_bpc(idx - self.vwsm);
                }
            } else if c(self, vsi) == KEY_O && c(self, vsi + 1) == KEY_O {
                // thoong
                self.vwsm = vei;
                self.set_bpc(idx - self.vwsm);
            }
        }
    }

    pub(super) fn handle_old_mark(&mut self) {
        let (vsi, vei, idx) = (self.vsi, self.vei, self.index);
        // default
        if self.vowel_count == 0 && self.chr(vei) == KEY_I {
            self.vwsm = vei;
        } else {
            self.vwsm = vsi;
        }
        self.set_bpc(idx - self.vwsm);

        // rule 2
        if self.vowel_count == 3
            || (vei + 1 < idx && is_consonant(self.chr(vei + 1)) && self.can_has_end_consonant())
        {
            self.vwsm = vsi + 1;
            self.set_bpc(idx - self.vwsm);
        }

        // rule 3
        let mut ii = vsi;
        while ii <= vei {
            if (self.chr(ii) == KEY_E && self.tw(ii) & TONE_MASK != 0)
                || (self.chr(ii) == KEY_O && self.tw(ii) & TONEW_MASK != 0)
            {
                self.vwsm = ii;
                self.set_bpc(idx - self.vwsm);
                break;
            }
            ii += 1;
        }

        self.hook.new_char_count = self.hook.backspace_count;
    }

    pub(super) fn insert_mark(&mut self, mark_mask: u32, can_modify_flag: bool) {
        self.vowel_count = 0;

        if can_modify_flag {
            self.hook.code = V_WILL_PROCESS;
        }
        self.set_bpc(0);
        self.set_ncc(0);

        self.find_and_calculate_vowel(false);
        self.vwsm = 0;

        // detect mark position
        if self.vowel_count == 1 {
            self.vwsm = self.vei;
            self.set_bpc(self.index - self.vei);
        } else {
            // vowel = 2 or 3
            if self.cfg.use_modern_orthography == 0 {
                self.handle_old_mark();
            } else {
                self.handle_modern_mark();
            }
            if self.tw(self.vei) & TONE_MASK != 0 || self.tw(self.vei) & TONEW_MASK != 0 {
                self.vwsm = self.vei;
            }
        }

        // send data
        let mut kk = self.index - 1 - self.vsi;
        // if duplicate same mark -> restore
        if self.tw(self.vwsm) & mark_mask != 0 {
            self.tw_update(self.vwsm, |w| w & !MARK_MASK);
            if can_modify_flag {
                self.hook.code = V_RESTORE;
            }
            let mut ii = self.vsi;
            while ii < self.index {
                self.tw_update(ii, |w| w & !MARK_MASK);
                let v = self.get(self.tw(ii));
                self.set_hdata(kk, v);
                kk -= 1;
                ii += 1;
            }
            self.temp_disable_key = true;
        } else {
            // remove other mark
            self.tw_update(self.vwsm, |w| w & !MARK_MASK);

            // add mark
            self.tw_update(self.vwsm, |w| w | mark_mask);
            let mut ii = self.vsi;
            while ii < self.index {
                if ii != self.vwsm {
                    // remove mark for other vowel
                    self.tw_update(ii, |w| w & !MARK_MASK);
                }
                let v = self.get(self.tw(ii));
                self.set_hdata(kk, v);
                kk -= 1;
                ii += 1;
            }

            self.set_bpc(self.index - self.vsi);
        }
        self.hook.new_char_count = self.hook.backspace_count;
    }
}
