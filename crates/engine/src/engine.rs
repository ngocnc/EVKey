//! Port of OpenKey `Engine.cpp` (GPL-3.0). Behavior-compatible with the C++ original;
//! the C++ globals become fields of [`Engine`]. Hook-oriented output lives in [`HookState`].

use std::collections::VecDeque;

use crate::consts::*;
use crate::keys::*;
use crate::macros::MacroMap;
use crate::tables::{get_character_code, tables};

// HookState.code values (HoolCodeState in DataType.h).
pub const V_DO_NOTHING: u8 = 0;
pub const V_WILL_PROCESS: u8 = 1;
pub const V_BREAK_WORD: u8 = 2;
pub const V_RESTORE: u8 = 3;
pub const V_REPLACE_MACRO: u8 = 4;
pub const V_RESTORE_AND_START_NEW_SESSION: u8 = 5;

/// Input types (`vInputType`).
pub const V_TELEX: i32 = 0;
pub const V_VNI: i32 = 1;
pub const V_SIMPLE_TELEX1: i32 = 2;
pub const V_SIMPLE_TELEX2: i32 = 3;

/// Result of the last event (`vKeyHookState`).
#[derive(Clone, Debug, Default)]
pub struct HookState {
    pub code: u8,
    pub backspace_count: u8,
    pub new_char_count: u8,
    pub ext_code: u8,
    pub char_data: [u32; MAX_BUFF],
    pub macro_key: Vec<u32>,
    pub macro_data: Vec<u32>,
}

/// Engine settings (the C++ `v*` externs the engine core reads).
#[derive(Clone, Debug)]
pub struct Config {
    pub input_type: i32,
    pub free_mark: i32,
    pub code_table: i32,
    pub check_spelling: i32,
    pub use_modern_orthography: i32,
    pub quick_telex: i32,
    pub restore_if_wrong_spelling: i32,
    pub use_macro: i32,
    pub auto_caps_macro: i32,
    pub upper_case_first_char: i32,
    /// Which sentence ends make `upper_case_first_char` capitalize the next word. OpenKey does the period and Enter,
    /// so those default on and `?` / `!` (an EVKey addition) default off, which keeps the parity test exact.
    pub upper_case_after_period: i32,
    pub upper_case_after_question: i32,
    pub upper_case_after_exclaim: i32,
    pub upper_case_after_enter: i32,
    /// EVKey addition: after the host lost the caret context (focus change, click), the next letter starts a sentence.
    pub upper_case_on_reset: i32,
    pub allow_consonant_zfwj: i32,
    pub quick_start_consonant: i32,
    pub quick_end_consonant: i32,
    /// Letters allowed as a syllable start while `allow_consonant_zfwj` is on (key codes; only `f`, `w`, `z`, `j` need it).
    pub custom_consonants: Vec<u16>,
}

impl Default for Config {
    /// Zero-initialised like the C++ globals; the custom consonants start at `z`, `f`, `w`, `j`.
    fn default() -> Self {
        Config {
            input_type: 0,
            free_mark: 0,
            code_table: 0,
            check_spelling: 0,
            use_modern_orthography: 0,
            quick_telex: 0,
            restore_if_wrong_spelling: 0,
            use_macro: 0,
            auto_caps_macro: 0,
            upper_case_first_char: 0,
            upper_case_after_period: 1,
            upper_case_after_question: 0,
            upper_case_after_exclaim: 0,
            upper_case_after_enter: 1,
            upper_case_on_reset: 0,
            allow_consonant_zfwj: 0,
            quick_start_consonant: 0,
            quick_end_consonant: 0,
            custom_consonants: vec![KEY_Z, KEY_F, KEY_W, KEY_J],
        }
    }
}

const CHAR_KEY_CODE: [u16; 21] = [
    KEY_BACKQUOTE, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0, KEY_MINUS,
    KEY_EQUALS, KEY_LEFT_BRACKET, KEY_RIGHT_BRACKET, KEY_BACK_SLASH, KEY_SEMICOLON, KEY_QUOTE,
    KEY_COMMA, KEY_DOT, KEY_SLASH,
];

const BREAK_CODE: [u16; 31] = [
    KEY_ESC, KEY_TAB, KEY_ENTER, KEY_RETURN, KEY_LEFT, KEY_RIGHT, KEY_DOWN, KEY_UP, KEY_COMMA,
    KEY_DOT, KEY_SLASH, KEY_SEMICOLON, KEY_QUOTE, KEY_BACK_SLASH, KEY_MINUS, KEY_EQUALS,
    KEY_BACKQUOTE, KEY_TAB, VK_INSERT, VK_HOME, VK_END, VK_DELETE, VK_PRIOR, VK_NEXT, VK_SNAPSHOT,
    VK_PRINT, VK_SELECT, VK_HELP, VK_EXECUTE, VK_NUMLOCK, VK_SCROLL,
];

const MACRO_BREAK_CODE: [u16; 9] = [
    KEY_RETURN, KEY_COMMA, KEY_DOT, KEY_SLASH, KEY_SEMICOLON, KEY_QUOTE, KEY_BACK_SLASH, KEY_MINUS,
    KEY_EQUALS,
];

const PROCESSING_CHAR: [[u16; 11]; 4] = [
    [KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z],
    [KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0],
    [KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z],
    [KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z],
];

fn is_consonant(k: u16) -> bool {
    !(k == KEY_A || k == KEY_E || k == KEY_U || k == KEY_Y || k == KEY_I || k == KEY_O)
}

fn is_number_key(k: u16) -> bool {
    (KEY_0..=KEY_9).contains(&k)
}

fn is_bracket_key(k: u16) -> bool {
    k == KEY_LEFT_BRACKET || k == KEY_RIGHT_BRACKET
}

/// One typing session: the OpenKey engine state.
pub struct Engine {
    pub cfg: Config,
    pub hook: HookState,
    pub macros: MacroMap,

    typing_word: [u32; MAX_BUFF],
    index: i32,
    long_word_helper: Vec<u32>,
    typing_states: VecDeque<Vec<u32>>,
    key_states: [u32; MAX_BUFF],
    state_index: i32,

    temp_disable_key: bool,
    is_corect: bool,
    is_changed: bool,
    vowel_count: i32,
    vsi: i32,
    vei: i32,
    vwsm: i32,
    is_caps: bool,
    space_count: i32,
    has_handled_macro: bool,
    upper_case_status: u8,
    special_char: Vec<u32>,
    use_spell_checking_before: bool,
    has_handle_quick_consonant: bool,
    will_temp_off_engine: bool,
    /// Word length captured when a word-ending event zeroes `index` (quick-consonant / macro).
    ended_word_len: i32,

    spelling_ok: bool,
    spelling_vowel_ok: bool,
    spelling_end_index: i32,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            cfg: Config::default(),
            hook: HookState::default(),
            macros: MacroMap::new(),
            typing_word: [0; MAX_BUFF],
            index: 0,
            long_word_helper: Vec::new(),
            typing_states: VecDeque::new(),
            key_states: [0; MAX_BUFF],
            state_index: 0,
            temp_disable_key: false,
            is_corect: false,
            is_changed: false,
            vowel_count: 0,
            vsi: 0,
            vei: 0,
            vwsm: 0,
            is_caps: false,
            space_count: 0,
            has_handled_macro: false,
            upper_case_status: 0,
            special_char: Vec::new(),
            use_spell_checking_before: false,
            has_handle_quick_consonant: false,
            will_temp_off_engine: false,
            ended_word_len: -1,
            spelling_ok: false,
            spelling_vowel_ok: false,
            spelling_end_index: 0,
        }
    }

    /// `vKeyInit`.
    pub fn init(&mut self) {
        self.index = 0;
        self.state_index = 0;
        self.use_spell_checking_before = self.cfg.check_spelling != 0;
        self.typing_states.clear();
        self.long_word_helper.clear();
    }

    /// The word being typed, as raw engine words (key | masks).
    pub fn typing_word(&self) -> &[u32] {
        &self.typing_word[..self.index as usize]
    }

    /// The word a word-ending event leaves behind, as raw engine words. Valid right after
    /// `key_event` for space/break keys whose result is `V_RESTORE*` (it survives the index reset
    /// done for quick consonants).
    pub fn word_after_event(&self) -> &[u32] {
        let n = if self.ended_word_len >= 0 { self.ended_word_len } else { self.index };
        &self.typing_word[..n as usize]
    }

    /// Forget the current word and all word history. For IME hosts: once a word is committed to
    /// the application it can no longer be edited, so backspace must not resurrect it.
    pub fn finish_word(&mut self) {
        self.start_new_session();
        self.typing_states.clear();
        self.special_char.clear();
        self.space_count = 0;
    }

    // ---- small helpers -------------------------------------------------------------------

    fn tw(&self, i: i32) -> u32 {
        if (0..MAX_BUFF as i32).contains(&i) { self.typing_word[i as usize] } else { 0 }
    }

    fn chr(&self, i: i32) -> u16 {
        self.tw(i) as u16
    }

    fn tw_update(&mut self, i: i32, f: impl FnOnce(u32) -> u32) {
        if (0..MAX_BUFF as i32).contains(&i) {
            let v = &mut self.typing_word[i as usize];
            *v = f(*v);
        }
    }

    fn set_tw(&mut self, i: i32, v: u32) {
        self.tw_update(i, |_| v);
    }

    fn set_hdata(&mut self, i: i32, v: u32) {
        if (0..MAX_BUFF as i32).contains(&i) {
            self.hook.char_data[i as usize] = v;
        }
    }

    fn get(&self, data: u32) -> u32 {
        get_character_code(self.cfg.code_table as usize, data)
    }

    fn set_bpc(&mut self, v: i32) {
        self.hook.backspace_count = v as u8;
    }

    fn set_ncc(&mut self, v: i32) {
        self.hook.new_char_count = v as u8;
    }

    fn bpc(&self) -> i32 {
        self.hook.backspace_count as i32
    }

    fn input_type(&self) -> usize {
        self.cfg.input_type.clamp(0, 3) as usize
    }

    fn pc(&self, idx: usize) -> u16 {
        PROCESSING_CHAR[self.input_type()][idx]
    }

    fn is_vni(&self) -> bool {
        self.cfg.input_type == V_VNI
    }

    fn is_key_z(&self, k: u16) -> bool {
        self.pc(10) == k
    }

    fn is_key_d(&self, k: u16) -> bool {
        self.pc(9) == k
    }

    fn is_key_w(&self, k: u16) -> bool {
        if !self.is_vni() { self.pc(8) == k } else { self.pc(8) == k || self.pc(7) == k }
    }

    fn is_key_double(&self, k: u16) -> bool {
        if !self.is_vni() { self.pc(5) == k || self.pc(6) == k || self.pc(7) == k } else { self.pc(6) == k }
    }

    fn is_key_s(&self, k: u16) -> bool {
        self.pc(0) == k
    }

    fn is_key_f(&self, k: u16) -> bool {
        self.pc(1) == k
    }

    fn is_key_r(&self, k: u16) -> bool {
        self.pc(2) == k
    }

    fn is_key_x(&self, k: u16) -> bool {
        self.pc(3) == k
    }

    fn is_key_j(&self, k: u16) -> bool {
        self.pc(4) == k
    }

    fn is_mark_key(&self, k: u16) -> bool {
        if !self.is_vni() {
            k == KEY_S || k == KEY_F || k == KEY_R || k == KEY_J || k == KEY_X
        } else {
            k == KEY_1 || k == KEY_2 || k == KEY_3 || k == KEY_5 || k == KEY_4
        }
    }

    fn is_special_key(&self, k: u16) -> bool {
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

    fn is_quick_telex_key(&self, k: u16) -> bool {
        self.index > 0
            && (k == KEY_C || k == KEY_G || k == KEY_K || k == KEY_N || k == KEY_Q || k == KEY_P || k == KEY_T)
            && self.chr(self.index - 1) == k
    }

    fn is_word_break(&self, is_mouse: bool, data: u16) -> bool {
        is_mouse || BREAK_CODE.contains(&data)
    }

    fn is_macro_break_code(&self, data: u16) -> bool {
        MACRO_BREAK_CODE.contains(&data)
    }

    fn find_macro(&mut self) -> bool {
        self.macros.find(
            &mut self.hook.macro_key,
            &mut self.hook.macro_data,
            self.cfg.code_table as usize,
            self.cfg.auto_caps_macro != 0,
        )
    }

    // ---- spelling ---------------------------------------------------------------------------

    fn check_spelling(&mut self, force_check_vowel: bool) {
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

    fn check_grammar(&mut self, delta_back_space: i32) {
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

    // ---- buffer ---------------------------------------------------------------------------------

    fn set_key_data(&mut self, index: i32, key_code: u16, is_caps: bool) {
        if !(0..MAX_BUFF as i32).contains(&index) {
            return;
        }
        self.typing_word[index as usize] = key_code as u32 | if is_caps { CAPS_MASK } else { 0 };
    }

    fn insert_key(&mut self, key_code: u16, is_caps: bool, check_spelling: bool) {
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

    fn insert_state(&mut self, key_code: u16, is_caps: bool) {
        let v = key_code as u32 | if is_caps { CAPS_MASK } else { 0 };
        if self.state_index >= MAX_BUFF as i32 {
            self.key_states.copy_within(1..MAX_BUFF, 0);
            self.key_states[(self.state_index - 1) as usize] = v;
        } else {
            self.key_states[self.state_index as usize] = v;
            self.state_index += 1;
        }
    }

    fn save_word(&mut self) {
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

    fn save_word_repeat(&mut self, key_code: u32, count: i32) {
        self.typing_states.push_back(vec![key_code; count.max(0) as usize]);
    }

    fn save_special_char(&mut self) {
        let data = std::mem::take(&mut self.special_char);
        self.typing_states.push_back(data);
    }

    fn restore_last_typing_state(&mut self) {
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

    fn check_correct_vowel(&mut self, charset: &[Vec<u16>], i: usize, mark_key: u16) {
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

    fn find_and_calculate_vowel(&mut self, for_grammar: bool) {
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

    fn remove_mark(&mut self) {
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

    fn can_has_end_consonant(&self) -> bool {
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

    fn handle_modern_mark(&mut self) {
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

    fn handle_old_mark(&mut self) {
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

    fn insert_mark(&mut self, mark_mask: u32, can_modify_flag: bool) {
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

    fn insert_d(&mut self) {
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

    fn insert_aoe(&mut self, data: u16) {
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

    fn insert_w(&mut self) {
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

    fn reverse_last_standalone_char(&mut self, key_code: u16, is_caps: bool) {
        self.hook.code = V_WILL_PROCESS;
        self.set_bpc(0);
        self.set_ncc(1);
        self.hook.ext_code = 4;
        let w = key_code as u32 | TONEW_MASK | STANDALONE_MASK | if is_caps { CAPS_MASK } else { 0 };
        self.set_tw(self.index - 1, w);
        let v = self.get(w);
        self.set_hdata(0, v);
    }

    fn check_for_standalone_char(&mut self, data: u16, is_caps: bool, key_will_reverse: u16) {
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

    fn upper_case_first_character(&mut self) {
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

    fn handle_main_key(&mut self, data: u16, is_caps: bool) {
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

    fn handle_quick_telex(&mut self, data: u16, is_caps: bool) {
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

    fn check_restore_if_wrong_spelling(&mut self, handle_code: u8) -> bool {
        for ii in 0..self.index {
            if !is_consonant(self.chr(ii))
                && (self.tw(ii) & MARK_MASK != 0 || self.tw(ii) & TONE_MASK != 0 || self.tw(ii) & TONEW_MASK != 0)
            {
                self.restore_raw_word(handle_code);
                return true;
            }
        }
        false
    }

    /// Rebuilds the word from the raw typed keys (`key_states`) and tells the host to show them.
    fn restore_raw_word(&mut self, handle_code: u8) {
        self.hook.code = handle_code;
        self.set_bpc(self.index);
        self.set_ncc(self.state_index);
        for i in 0..self.state_index {
            let v = self.key_states[i as usize];
            self.set_tw(i, v);
            self.set_hdata(self.state_index - 1 - i, v);
        }
        self.index = self.state_index;
    }

    /// Hoàn tác từ (Esc): shows the raw typed keys of the word in progress and leaves the rest of it unconverted.
    /// Returns false when no word is in progress, so the key passes through.
    pub fn undo_word(&mut self) -> bool {
        if self.index == 0 {
            return false;
        }
        self.restore_raw_word(V_RESTORE);
        self.temp_off_engine(true);
        true
    }

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

    fn check_quick_consonant(&mut self) -> bool {
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

    // ---- entry points -------------------------------------------------------------------------

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
                self.hook.ext_code = 3;
                self.special_char.push(data as u32 | if is_caps { CAPS_MASK } else { 0 });
            }
        }
    }
}
