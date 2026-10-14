//! Quick Telex (`handleQuickTelex`) and quick start/end consonants (`checkQuickConsonant`).

use super::*;

impl Engine {
    pub(super) fn handle_quick_telex(&mut self, data: u16, is_caps: bool) {
        let t = tables();
        let caps = if is_caps { CAPS_MASK } else { 0 };
        self.hook.code = V_WILL_PROCESS;
        self.set_bpc(1);
        self.set_ncc(2);
        let pair = t.quick_telex.get(&(data as u32)).cloned().unwrap_or_default();
        let a = pair.first().copied().unwrap_or(0) as u32;
        let b = pair.get(1).copied().unwrap_or(0) as u32;
        self.set_hdata(1, a | caps);
        self.set_hdata(0, b | caps);
        self.insert_key(b as u16, is_caps, false);
    }

    pub(super) fn check_quick_consonant(&mut self) -> bool {
        if self.index <= 1 {
            return false;
        }
        let mut l = 0;
        let caps = |w: u32| if w & CAPS_MASK != 0 { CAPS_MASK } else { 0 };
        let t = tables();
        if self.index > 0 {
            if self.cfg.quick_start_consonant != 0 && t.quick_start_consonant.contains_key(&self.chr(0)) {
                let q = t.quick_start_consonant[&self.chr(0)].clone();
                self.hook.code = V_RESTORE;
                self.set_bpc(self.index);
                self.set_ncc(self.index + 1);
                if self.index < MAX_BUFF as i32 - 1 {
                    self.index += 1;
                }
                // right shift
                let mut i = self.index - 1;
                while i >= 2 {
                    let v = self.tw(i - 1);
                    self.set_tw(i, v);
                    i -= 1;
                }
                let both = if (self.tw(0) & CAPS_MASK) != 0 && (self.tw(2) & CAPS_MASK) != 0 { CAPS_MASK } else { 0 };
                self.set_tw(1, q[1] as u32 | both);
                let c0 = caps(self.tw(0));
                self.set_tw(0, q[0] as u32 | c0);
                l = 1;
            }
            if self.cfg.quick_end_consonant != 0
                && (self.index - 2 >= 0 && !is_consonant(self.chr(self.index - 2)))
                && t.quick_end_consonant.contains_key(&self.chr(self.index - 1))
            {
                self.hook.code = V_RESTORE;
                if l == 1 {
                    self.hook.new_char_count = self.hook.new_char_count.wrapping_add(1);
                } else {
                    self.set_bpc(1);
                    self.set_ncc(2);
                }
                if self.index < MAX_BUFF as i32 - 1 {
                    self.index += 1;
                }
                let q = t.quick_end_consonant[&self.chr(self.index - 2)].clone();
                let c = caps(self.tw(self.index - 2));
                self.set_tw(self.index - 1, q[1] as u32 | c);
                self.set_tw(self.index - 2, q[0] as u32 | c);

                l = 1;
            }
            if l == 1 {
                self.has_handle_quick_consonant = true;
                let mut i = self.index - 1;
                while i >= 0 {
                    let v = self.get(self.tw(i));
                    self.set_hdata(self.index - 1 - i, v);
                    i -= 1;
                }
                return true;
            }
        }
        false
    }
}
