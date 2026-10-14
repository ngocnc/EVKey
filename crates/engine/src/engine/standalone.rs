//! Standalone `w` to `ư` and reversal of the last standalone char (`checkForStandaloneChar`, `reverseLastStandaloneChar`).

use super::*;

impl Engine {
    pub(super) fn reverse_last_standalone_char(&mut self, key_code: u16, is_caps: bool) {
        self.hook.code = V_WILL_PROCESS;
        self.set_bpc(0);
        self.set_ncc(1);
        self.hook.ext_code = 4;
        let w = key_code as u32 | TONEW_MASK | STANDALONE_MASK | if is_caps { CAPS_MASK } else { 0 };
        self.set_tw(self.index - 1, w);
        let v = self.get(w);
        self.set_hdata(0, v);
    }

    pub(super) fn check_for_standalone_char(&mut self, data: u16, is_caps: bool, key_will_reverse: u16) {
        if self.chr(self.index - 1) == key_will_reverse && self.tw(self.index - 1) & TONEW_MASK != 0 {
            self.hook.code = V_WILL_PROCESS;
            self.set_bpc(1);
            self.set_ncc(1);
            let w = data as u32 | if is_caps { CAPS_MASK } else { 0 };
            self.set_tw(self.index - 1, w);
            let v = self.get(w);
            self.set_hdata(0, v);
            return;
        }

        // check standalone w -> ư
        if self.index > 0 && self.chr(self.index - 1) == KEY_U && key_will_reverse == KEY_O {
            self.insert_key(key_will_reverse, is_caps, true);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        }

        let t = tables();
        if self.index == 0 {
            // zero char
            self.insert_key(data, is_caps, false);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        } else if self.index == 1 {
            // 1 char
            for bad in t.standalone_w_bad.iter() {
                if self.chr(0) == *bad {
                    self.insert_key(data, is_caps, true);
                    return;
                }
            }
            self.insert_key(data, is_caps, false);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        } else if self.index == 2 {
            for allowed in t.double_w_allowed.iter() {
                if self.chr(0) == allowed[0] && self.chr(1) == allowed[1] {
                    self.insert_key(data, is_caps, false);
                    self.reverse_last_standalone_char(key_will_reverse, is_caps);
                    return;
                }
            }
            self.insert_key(data, is_caps, true);
            return;
        }

        self.insert_key(data, is_caps, true);
    }
}
