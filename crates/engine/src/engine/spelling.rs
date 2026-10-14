//! `checkSpelling` and `checkGrammar` (OpenKey `Engine.cpp`).

use super::*;

impl Engine {
    // ---- spelling ---------------------------------------------------------------------------

    pub(super) fn check_spelling(&mut self, force_check_vowel: bool) {
        let t = tables();
        self.spelling_ok = false;
        self.spelling_vowel_ok = true;
        self.spelling_end_index = self.index;

        if self.index > 0 && self.chr(self.index - 1) == KEY_RIGHT_BRACKET {
            self.spelling_end_index = self.index - 1;
        }
        let end = self.spelling_end_index;

        if end > 0 {
            let mut j: i32 = 0;
            // check first consonant
            if is_consonant(self.chr(0)) {
                let m1: u32 = if self.cfg.quick_start_consonant != 0 { END_CONSONANT_MASK } else { 0 };
                let allow = self.cfg.allow_consonant_zfwj != 0;
                for row in t.consonant_table.iter() {
                    let mut flag = false;
                    if (end as usize) < row.len() {
                        flag = true;
                    }
                    j = 0;
                    while (j as usize) < row.len() {
                        let e = row[j as usize] as u32;
                        let c = self.chr(j) as u32;
                        let base = e & !CONSONANT_ALLOW_MASK;
                        let extra = allow && e & CONSONANT_ALLOW_MASK != 0 && self.cfg.custom_consonants.contains(&(base as u16));
                        if end > j && (e & !m1) != c && !(extra && base == c) {
                            flag = true;
                            break;
                        }
                        j += 1;
                    }
                    if flag {
                        continue;
                    }
                    break;
                }
            }

            if j == end {
                // for "d" case
                self.spelling_ok = true;
            }

            // check next vowel
            let mut k = j;
            self.vsi = k;
            if self.chr(self.vsi) == KEY_U && k > 0 && k < end - 1 && self.chr(self.vsi - 1) == KEY_Q {
                k += 1;
                j = k;
                self.vsi = k;
            } else if self.index >= 2
                && self.chr(0) == KEY_G
                && self.chr(1) == KEY_I
                && is_consonant(self.chr(2))
            {
                self.vsi = 1;
                k = 1;
                j = 1;
            }
            for _ in 0..3 {
                if k < end && !is_consonant(self.chr(k)) {
                    k += 1;
                    self.vei = k;
                }
            }
            if k > j {
                // has vowel
                self.spelling_vowel_ok = false;
                if k - j > 1 && force_check_vowel {
                    let empty = Vec::new();
                    let vowel_set = t.vowel_combine.get(&self.chr(j)).unwrap_or(&empty);
                    for set in vowel_set.iter() {
                        let mut flag = false;
                        let mut ii: i32 = 1;
                        while (ii as usize) < set.len() {
                            let at = j + ii - 1;
                            if at < end
                                && set[ii as usize]
                                    != (self.chr(at) as u32
                                        | (self.tw(at) & TONEW_MASK)
                                        | (self.tw(at) & TONE_MASK))
                            {
                                flag = true;
                                break;
                            }
                            ii += 1;
                        }
                        if flag
                            || (k < end && set[0] == 0)
                            || (j + ii - 1 < end && !is_consonant(self.chr(j + ii - 1)))
                        {
                            continue;
                        }
                        self.spelling_vowel_ok = true;
                        break;
                    }
                } else if !is_consonant(self.chr(j)) {
                    self.spelling_vowel_ok = true;
                }

                // continue check last consonant
                let m: u32 = if self.cfg.quick_end_consonant != 0 { END_CONSONANT_MASK } else { 0 };
                for row in t.end_consonant_table.iter() {
                    let mut flag = false;
                    let mut jj: i32 = 0;
                    while (jj as usize) < row.len() {
                        if end > k + jj && (row[jj as usize] as u32 & !m) != self.chr(k + jj) as u32 {
                            flag = true;
                            break;
                        }
                        jj += 1;
                    }
                    if flag {
                        continue;
                    }
                    if k + jj >= end {
                        self.spelling_ok = true;
                        break;
                    }
                }

                // limit: end consonant "ch", "t" can not use with "~", "`", "?"
                if self.spelling_ok {
                    let idx = self.index;
                    let bad = |w: u32| !((w & MARK1_MASK) != 0 || (w & MARK5_MASK) != 0 || (w & MARK_MASK) == 0);
                    if idx >= 3
                        && self.chr(idx - 1) == KEY_H
                        && self.chr(idx - 2) == KEY_C
                        && bad(self.tw(idx - 3))
                    {
                        self.spelling_ok = false;
                    } else if idx >= 2 && self.chr(idx - 1) == KEY_T && bad(self.tw(idx - 2)) {
                        self.spelling_ok = false;
                    }
                }
            }
        } else {
            self.spelling_ok = true;
        }
        self.temp_disable_key = !(self.spelling_ok && self.spelling_vowel_ok);
    }

    pub(super) fn check_grammar(&mut self, delta_back_space: i32) {
        if self.index <= 1 || self.index >= MAX_BUFF as i32 {
            return;
        }
        self.find_and_calculate_vowel(true);
        if self.vowel_count == 0 {
            return;
        }
        let mut checked = false;
        let l = self.vsi;

        // if N key for case: "thuơn", "ưoi", "ưom", "ưoc"
        if self.index >= 3 {
            let mut i = self.index - 1;
            while i >= 0 {
                let c = self.chr(i);
                if c == KEY_N || c == KEY_C || c == KEY_I || c == KEY_M || c == KEY_P || c == KEY_T {
                    if i - 2 >= 0 && self.chr(i - 1) == KEY_O && self.chr(i - 2) == KEY_U {
                        if (self.tw(i - 1) & TONEW_MASK) ^ (self.tw(i - 2) & TONEW_MASK) != 0 {
                            self.tw_update(i - 2, |w| w | TONEW_MASK);
                            self.tw_update(i - 1, |w| w | TONEW_MASK);
                            checked = true;
                            break;
                        }
                    }
                }
                i -= 1;
            }
        }

        // check mark
        if self.index >= 2 {
            let mut i = l;
            while i <= self.vei {
                if self.tw(i) & MARK_MASK != 0 {
                    let mark = self.tw(i) & MARK_MASK;
                    self.tw_update(i, |w| w & !MARK_MASK);
                    self.insert_mark(mark, false);
                    if i != self.vwsm {
                        checked = true;
                    }
                    break;
                }
                i += 1;
            }
        }

        // re-arrange data to sendback
        if checked {
            if self.hook.code == V_DO_NOTHING {
                self.hook.code = V_WILL_PROCESS;
            }
            self.set_bpc(0);
            let mut i = self.index - 1;
            while i >= l {
                self.set_bpc(self.bpc() + 1);
                let v = self.get(self.tw(i));
                self.set_hdata(self.index - 1 - i, v);
                i -= 1;
            }
            self.hook.new_char_count = self.hook.backspace_count;
            self.set_bpc(self.bpc() + delta_back_space);
            self.hook.ext_code = 4;
        }
    }
}
