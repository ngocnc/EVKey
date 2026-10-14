//! Tone keys that rewrite the typed word: `d`, circumflex `aoe`, and horn/breve `w` (`insertD`, `insertAOE`, `insertW`).

use super::*;

impl Engine {
    pub(super) fn insert_d(&mut self) {
        self.hook.code = V_WILL_PROCESS;
        self.set_bpc(0);
        let mut ii = self.index - 1;
        while ii >= 0 {
            self.set_bpc(self.bpc() + 1);
            if self.chr(ii) == KEY_D {
                // reverse unicode char
                if self.tw(ii) & TONE_MASK != 0 {
                    // restore and disable temporary
                    self.hook.code = V_RESTORE;
                    self.tw_update(ii, |w| w & !TONE_MASK);
                    self.set_hdata(self.index - 1 - ii, self.tw(ii));
                    self.temp_disable_key = true;
                    break;
                } else {
                    self.tw_update(ii, |w| w | TONE_MASK);
                    let v = self.get(self.tw(ii));
                    self.set_hdata(self.index - 1 - ii, v);
                }
                break;
            } else {
                // preresent old char
                let v = self.get(self.tw(ii));
                self.set_hdata(self.index - 1 - ii, v);
            }
            ii -= 1;
        }
        self.hook.new_char_count = self.hook.backspace_count;
    }

    pub(super) fn insert_aoe(&mut self, data: u16) {
        self.find_and_calculate_vowel(false);

        // remove W tone
        let mut ii = self.vsi;
        while ii <= self.vei {
            self.tw_update(ii, |w| w & !TONEW_MASK);
            ii += 1;
        }

        self.hook.code = V_WILL_PROCESS;
        self.set_bpc(0);

        let mut ii = self.index - 1;
        while ii >= 0 {
            self.set_bpc(self.bpc() + 1);
            if self.chr(ii) == data {
                // reverse unicode char
                if self.tw(ii) & TONE_MASK != 0 {
                    // restore and disable temporary
                    self.hook.code = V_RESTORE;
                    self.tw_update(ii, |w| w & !TONE_MASK);
                    self.set_hdata(self.index - 1 - ii, self.tw(ii));
                    if data != KEY_O {
                        // case thoòng
                        self.temp_disable_key = true;
                    }
                    break;
                } else {
                    self.tw_update(ii, |w| w | TONE_MASK);
                    if !self.is_key_d(data) {
                        self.tw_update(ii, |w| w & !TONEW_MASK);
                    }
                    let v = self.get(self.tw(ii));
                    self.set_hdata(self.index - 1 - ii, v);
                }
                break;
            } else {
                // preresent old char
                let v = self.get(self.tw(ii));
                self.set_hdata(self.index - 1 - ii, v);
            }
            ii -= 1;
        }
        self.hook.new_char_count = self.hook.backspace_count;
    }

    pub(super) fn insert_w(&mut self) {
        let mut is_restored_w = false;

        self.find_and_calculate_vowel(false);

        // remove ^ tone
        let mut ii = self.vsi;
        while ii <= self.vei {
            self.tw_update(ii, |w| w & !TONE_MASK);
            ii += 1;
        }

        let vsi = self.vsi;
        if self.vowel_count > 1 {
            self.set_bpc(self.index - vsi);
            self.hook.new_char_count = self.hook.backspace_count;

            if ((self.tw(vsi) & TONEW_MASK) != 0 && (self.tw(vsi + 1) & TONEW_MASK) != 0)
                || ((self.tw(vsi) & TONEW_MASK) != 0 && self.chr(vsi + 1) == KEY_I)
                || ((self.tw(vsi) & TONEW_MASK) != 0 && self.chr(vsi + 1) == KEY_A)
            {
                // restore and disable temporary
                self.hook.code = V_RESTORE;

                let mut ii = vsi;
                while ii < self.index {
                    self.tw_update(ii, |w| w & !TONEW_MASK);
                    let v = self.get(self.tw(ii)) & !STANDALONE_MASK;
                    self.set_hdata(self.index - 1 - ii, v);
                    ii += 1;
                }
                self.temp_disable_key = true;
            } else {
                self.hook.code = V_WILL_PROCESS;

                if self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_O {
                    if vsi - 2 >= 0
                        && self.tw(vsi - 2) == KEY_T as u32
                        && self.tw(vsi - 1) == KEY_H as u32
                    {
                        self.tw_update(vsi + 1, |w| w | TONEW_MASK);
                        if vsi + 2 < self.index && self.chr(vsi + 2) == KEY_N {
                            self.tw_update(vsi, |w| w | TONEW_MASK);
                        }
                    } else if vsi - 1 >= 0 && self.tw(vsi - 1) == KEY_Q as u32 {
                        self.tw_update(vsi + 1, |w| w | TONEW_MASK);
                    } else {
                        self.tw_update(vsi, |w| w | TONEW_MASK);
                        self.tw_update(vsi + 1, |w| w | TONEW_MASK);
                    }
                } else if (self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_A)
                    || (self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_I)
                    || (self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_U)
                    || (self.chr(vsi) == KEY_O && self.chr(vsi + 1) == KEY_I)
                {
                    self.tw_update(vsi, |w| w | TONEW_MASK);
                } else if (self.chr(vsi) == KEY_I && self.chr(vsi + 1) == KEY_O)
                    || (self.chr(vsi) == KEY_O && self.chr(vsi + 1) == KEY_A)
                {
                    self.tw_update(vsi + 1, |w| w | TONEW_MASK);
                } else {
                    // don't do anything
                    self.temp_disable_key = true;
                    self.is_changed = false;
                    self.hook.code = V_DO_NOTHING;
                }

                let mut ii = vsi;
                while ii < self.index {
                    let v = self.get(self.tw(ii));
                    self.set_hdata(self.index - 1 - ii, v);
                    ii += 1;
                }
            }
            return;
        }

        self.hook.code = V_WILL_PROCESS;
        self.set_bpc(0);

        let mut ii = self.index - 1;
        while ii >= 0 {
            if ii < vsi {
                break;
            }
            self.set_bpc(self.bpc() + 1);
            let c = self.chr(ii);
            if c == KEY_A || c == KEY_U || c == KEY_O {
                if self.tw(ii) & TONEW_MASK != 0 {
                    // restore and disable temporary
                    if self.tw(ii) & STANDALONE_MASK != 0 {
                        self.hook.code = V_WILL_PROCESS;
                        let caps = self.tw(ii) & CAPS_MASK;
                        if c == KEY_U {
                            self.set_tw(ii, KEY_W as u32 | caps);
                        } else if c == KEY_O {
                            self.hook.code = V_RESTORE;
                            self.set_tw(ii, KEY_O as u32 | caps);
                            is_restored_w = true;
                        }
                        self.set_hdata(self.index - 1 - ii, self.tw(ii));
                    } else {
                        self.hook.code = V_RESTORE;
                        self.tw_update(ii, |w| w & !TONEW_MASK);
                        self.set_hdata(self.index - 1 - ii, self.tw(ii));
                        is_restored_w = true;
                    }

                    self.temp_disable_key = true;
                } else {
                    self.tw_update(ii, |w| (w | TONEW_MASK) & !TONE_MASK);
                    let v = self.get(self.tw(ii));
                    self.set_hdata(self.index - 1 - ii, v);
                }
                break;
            } else {
                let v = self.get(self.tw(ii));
                self.set_hdata(self.index - 1 - ii, v);
            }
            ii -= 1;
        }
        self.hook.new_char_count = self.hook.backspace_count;
        let _ = is_restored_w;
    }
}
