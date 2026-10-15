//! Buffer accessors, hook output setters and key classifiers for the active input method.

use super::*;

impl Engine {
    // ---- small helpers -------------------------------------------------------------------

    #[inline]
    pub(super) fn tw(&self, i: i32) -> u32 {
        if (0..MAX_BUFF as i32).contains(&i) { self.typing_word[i as usize] } else { 0 }
    }

    #[inline]
    pub(super) fn chr(&self, i: i32) -> u16 {
        self.tw(i) as u16
    }

    #[inline]
    pub(super) fn tw_update(&mut self, i: i32, f: impl FnOnce(u32) -> u32) {
        if (0..MAX_BUFF as i32).contains(&i) {
            let v = &mut self.typing_word[i as usize];
            *v = f(*v);
        }
    }

    #[inline]
    pub(super) fn set_tw(&mut self, i: i32, v: u32) {
        self.tw_update(i, |_| v);
    }

    #[inline]
    pub(super) fn set_hdata(&mut self, i: i32, v: u32) {
        if (0..MAX_BUFF as i32).contains(&i) {
            self.hook.char_data[i as usize] = v;
        }
    }

    #[inline]
    pub(super) fn get(&self, data: u32) -> u32 {
        get_character_code(self.cfg.code_table as usize, data)
    }

    #[inline]
    pub(super) fn set_bpc(&mut self, v: i32) {
        self.hook.backspace_count = v as u8;
    }

    #[inline]
    pub(super) fn set_ncc(&mut self, v: i32) {
        self.hook.new_char_count = v as u8;
    }

    #[inline]
    pub(super) fn bpc(&self) -> i32 {
        self.hook.backspace_count as i32
    }

    #[inline]
    pub(super) fn input_type(&self) -> usize {
        self.cfg.input_type.clamp(0, 3) as usize
    }

    #[inline]
    pub(super) fn pc(&self, idx: usize) -> u16 {
        PROCESSING_CHAR[self.input_type()][idx]
    }

    #[inline]
    pub(super) fn is_vni(&self) -> bool {
        self.cfg.input_type == V_VNI
    }

    #[inline]
    pub(super) fn is_key_z(&self, k: u16) -> bool {
        self.pc(10) == k
    }

    #[inline]
    pub(super) fn is_key_d(&self, k: u16) -> bool {
        self.pc(9) == k
    }

    #[inline]
    pub(super) fn is_key_w(&self, k: u16) -> bool {
        if !self.is_vni() { self.pc(8) == k } else { self.pc(8) == k || self.pc(7) == k }
    }

    #[inline]
    pub(super) fn is_key_double(&self, k: u16) -> bool {
        if !self.is_vni() { self.pc(5) == k || self.pc(6) == k || self.pc(7) == k } else { self.pc(6) == k }
    }

    #[inline]
    pub(super) fn is_key_s(&self, k: u16) -> bool {
        self.pc(0) == k
    }

    #[inline]
    pub(super) fn is_key_f(&self, k: u16) -> bool {
        self.pc(1) == k
    }

    #[inline]
    pub(super) fn is_key_r(&self, k: u16) -> bool {
        self.pc(2) == k
    }

    #[inline]
    pub(super) fn is_key_x(&self, k: u16) -> bool {
        self.pc(3) == k
    }

    #[inline]
    pub(super) fn is_key_j(&self, k: u16) -> bool {
        self.pc(4) == k
    }

    #[inline]
    pub(super) fn is_mark_key(&self, k: u16) -> bool {
        if !self.is_vni() {
            k == KEY_S || k == KEY_F || k == KEY_R || k == KEY_J || k == KEY_X
        } else {
            k == KEY_1 || k == KEY_2 || k == KEY_3 || k == KEY_5 || k == KEY_4
        }
    }

    #[inline]
    pub(super) fn is_special_key(&self, k: u16) -> bool {
        match self.cfg.input_type {
            V_TELEX => {
                k == KEY_W || k == KEY_E || k == KEY_R || k == KEY_O || k == KEY_LEFT_BRACKET
                    || k == KEY_RIGHT_BRACKET || k == KEY_A || k == KEY_S || k == KEY_D || k == KEY_F
                    || k == KEY_J || k == KEY_Z || k == KEY_X
            }
            V_VNI => is_number_key(k),
            V_SIMPLE_TELEX1 | V_SIMPLE_TELEX2 => {
                k == KEY_W || k == KEY_E || k == KEY_R || k == KEY_O || k == KEY_A || k == KEY_S
                    || k == KEY_D || k == KEY_F || k == KEY_J || k == KEY_Z || k == KEY_X
            }
            _ => false,
        }
    }

    #[inline]
    pub(super) fn is_quick_telex_key(&self, k: u16) -> bool {
        self.index > 0
            && (k == KEY_C || k == KEY_G || k == KEY_K || k == KEY_N || k == KEY_Q || k == KEY_P || k == KEY_T)
            && self.chr(self.index - 1) == k
    }

    #[inline]
    pub(super) fn is_word_break(&self, is_mouse: bool, data: u16) -> bool {
        is_mouse || BREAK_CODE.contains(&data)
    }

    #[inline]
    pub(super) fn is_macro_break_code(&self, data: u16) -> bool {
        MACRO_BREAK_CODE.contains(&data)
    }

    pub(super) fn find_macro(&mut self) -> bool {
        let found = self.macros.find(
            &mut self.hook.macro_key,
            &mut self.hook.macro_data,
            self.cfg.code_table as usize,
            self.cfg.auto_caps_macro != 0 || self.macro_first_auto_capped,
        );
        self.macro_first_auto_capped = false;
        found
    }
}
