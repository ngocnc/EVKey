//! Drives `Composer` the way a TSF host would and checks the resulting text.

use evkey_engine::composer::{Action, Composer};
use evkey_engine::engine::Config;
use evkey_engine::keys::*;

struct Host {
    c: Composer,
    committed: String,
    comp: String,
}

impl Host {
    fn new(cfg: Config) -> Self {
        Host { c: Composer::new(cfg), committed: String::new(), comp: String::new() }
    }

    fn press(&mut self, vk: u16, native: Option<char>, shift: bool) {
        match self.c.key_down(vk, shift, false, false) {
            Action::Pass => self.committed.extend(native),
            Action::Compose(t) => self.comp = t,
            Action::Commit { text, eaten } => {
                self.committed.push_str(&text);
                self.comp.clear();
                if !eaten {
                    self.committed.extend(native);
                }
            }
        }
    }

    fn type_str(&mut self, s: &str) {
        for ch in s.chars() {
            let (vk, shift) = match ch {
                'a'..='z' => (ch.to_ascii_uppercase() as u16, false),
                'A'..='Z' => (ch as u16, true),
                ' ' => (KEY_SPACE, false),
                ',' => (KEY_COMMA, false),
                '.' => (KEY_DOT, false),
                '1'..='9' | '0' => (ch as u16, false),
                '[' => (KEY_LEFT_BRACKET, false),
                _ => panic!("unmapped {ch}"),
            };
            self.press(vk, Some(ch), shift);
        }
    }

    fn text(&self) -> String {
        format!("{}{}", self.committed, self.comp)
    }
}

fn telex() -> Config {
    Config { check_spelling: 1, ..Config::default() }
}

fn typed(cfg: Config, s: &str) -> String {
    let mut h = Host::new(cfg);
    h.type_str(s);
    h.text()
}

#[test]
fn telex_words_and_punctuation() {
    assert_eq!(typed(telex(), "Vieejt Nam, hoaf dduowcj."), "Việt Nam, hòa được.");
    assert_eq!(typed(telex(), "tooi yeeu ban"), "tôi yêu ban");
    assert_eq!(typed(telex(), "w "), "ư ");
}

#[test]
fn digits_start_no_composition_but_extend_a_word() {
    assert_eq!(typed(telex(), "123 a1b"), "123 a1b");
}

#[test]
fn backspace_edits_the_composition_and_never_resurrects_committed_words() {
    let mut h = Host::new(telex());
    h.type_str("hoaf");
    assert_eq!(h.text(), "hòa");
    h.press(KEY_DELETE, None, false);
    assert_eq!(h.text(), "hò");
    h.press(KEY_DELETE, None, false);
    h.press(KEY_DELETE, None, false);
    assert_eq!(h.text(), "");
    assert!(!h.c.is_composing());
    // backspace with no composition is the application's business
    assert_eq!(h.c.key_down(KEY_DELETE, false, false, false), Action::Pass);

    // type a word, commit it with space, then start a new word: backspacing it away must not
    // bring the first word back into the engine
    h.type_str("ba ca");
    h.press(KEY_DELETE, None, false);
    h.press(KEY_DELETE, None, false);
    assert!(!h.c.is_composing());
    h.type_str("x");
    assert_eq!(h.text(), "ba x");
}

#[test]
fn macro_replaces_word_at_space() {
    let cfg = Config { use_macro: 1, ..telex() };
    let mut h = Host::new(cfg);
    h.c.engine.macros.add("btw", "by the way", 0);
    h.type_str("btw ");
    assert_eq!(h.text(), "by the way ");
}

#[test]
fn restore_wrong_spelling_at_space() {
    let cfg = Config { restore_if_wrong_spelling: 1, ..telex() };
    assert_eq!(typed(cfg, "windows "), "windows ");
}

#[test]
fn quick_consonants_expand_on_commit() {
    let cfg = Config { quick_start_consonant: 1, quick_end_consonant: 1, ..telex() };
    assert_eq!(typed(cfg, "fanh ,"), "phanh ,");
}

#[test]
fn custom_consonants_choose_which_letters_may_start_a_syllable() {
    let cfg = |letters: Vec<u16>| Config { check_spelling: 1, allow_consonant_zfwj: 1, custom_consonants: letters, ..telex() };
    assert_eq!(typed(cfg(vec![evkey_engine::keys::KEY_F]), "fas "), "fá ");
    assert_eq!(typed(cfg(vec![evkey_engine::keys::KEY_Z]), "fas "), "fas ");
}

#[test]
fn legacy_tables_render() {
    // TCVN3: 'ệ' is a single legacy code, not Unicode
    let cfg = Config { code_table: 1, ..telex() };
    let out = typed(cfg, "ee");
    assert_eq!(out.chars().count(), 1);
    assert_ne!(out, "ê");
}

#[test]
fn temp_off_leaves_the_rest_of_the_word_raw() {
    let mut h = Host::new(telex());
    h.type_str("hoaf");
    h.c.temp_off();
    h.type_str("ee");
    assert_eq!(h.text(), "hòaee");
    h.press(KEY_DELETE, None, false);
    assert_eq!(h.text(), "hòae");
    h.type_str(" aa");
    assert_eq!(h.text(), "hòae â", "the next word converts again");
}

#[test]
fn apply_replaces_config_and_macros() {
    use evkey_engine::settings::{load, Loaded};
    let mut h = Host::new(telex());
    let mut l = Loaded::default();
    l.settings.tone_placement = evkey_engine::settings::TonePlacement::Old;
    l.macros.insert("vn".into(), "Việt Nam".into());
    h.c.apply(&l);
    h.type_str("hoaf vn ");
    assert_eq!(h.text(), "hoà Việt Nam ");
    let _ = load; // loading from disk is covered in settings.rs
}
