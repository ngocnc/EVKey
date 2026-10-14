//! Port of OpenKey `Engine.cpp` (GPL-3.0). Behavior-compatible with the C++ original;
//! the C++ globals become fields of [`Engine`]. Hook-oriented output lives in [`HookState`].
//! Method groups live in submodules; each file names the OpenKey functions it ports.

mod buffer;
mod capitalize;
mod helpers;
mod key_event;
mod main_key;
mod mark;
mod quick;
mod restore;
mod spelling;
mod standalone;
mod tone;

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
}
