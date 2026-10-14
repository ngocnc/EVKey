//! Turns one engine answer into the edit a host types. Pure: no OS calls, so it is unit-tested.
use evkey_engine::composer::render;
use evkey_engine::engine::{
    HookState, V_BREAK_WORD, V_DO_NOTHING, V_REPLACE_MACRO, V_RESTORE, V_RESTORE_AND_START_NEW_SESSION,
};

/// Delete `backspaces` characters, type `text`, then type the trigger key again if `resend_key`.
#[derive(Debug, PartialEq, Eq)]
pub struct Edit {
    pub backspaces: usize,
    pub text: String,
    pub resend_key: bool,
}

/// `None` means the key passes through untouched.
pub fn edit_for(hook: &HookState, code_table: i32) -> Option<Edit> {
    match hook.code {
        V_DO_NOTHING | V_BREAK_WORD => None,
        V_REPLACE_MACRO => Some(Edit {
            backspaces: hook.backspace_count as usize,
            text: render(hook.macro_data.iter().copied(), code_table),
            resend_key: true,
        }),
        _ => {
            let n = (hook.new_char_count as usize).min(hook.char_data.len());
            Some(Edit {
                backspaces: hook.backspace_count as usize,
                // OpenKey sends the new characters from the last one back to the first.
                text: render(hook.char_data[..n].iter().rev().copied(), code_table),
                resend_key: matches!(hook.code, V_RESTORE | V_RESTORE_AND_START_NEW_SESSION),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evkey_engine::composer::Composer;
    use evkey_engine::settings::{Loaded, Settings};
    use std::collections::BTreeMap;

    /// Types `keys` into a fake text field, applying each edit the way a host injects it.
    fn type_keys(loaded: &Loaded, keys: &str) -> String {
        let mut c = Composer::new(Settings::default().to_config(false));
        c.apply(loaded);
        let mut field = String::new();
        for ch in keys.chars() {
            c.engine.key_event(false, ch.to_ascii_uppercase() as u16, 0, false);
            match edit_for(&c.engine.hook, c.engine.cfg.code_table) {
                None => field.push(ch),
                Some(edit) => {
                    for _ in 0..edit.backspaces {
                        field.pop();
                    }
                    field.push_str(&edit.text);
                    if edit.resend_key {
                        field.push(ch);
                    }
                }
            }
        }
        field
    }

    fn loaded(macros: BTreeMap<String, String>) -> Loaded {
        Loaded { settings: Settings::default(), macros }
    }

    #[test]
    fn telex_edits_rebuild_the_word() {
        assert_eq!(type_keys(&loaded(BTreeMap::new()), "vieejt"), "việt");
    }

    #[test]
    fn a_sentence_with_marks_and_punctuation() {
        assert_eq!(type_keys(&loaded(BTreeMap::new()), "Vieejt Nam, hoaf dduowcj."), "Việt Nam, hòa được.");
    }

    #[test]
    fn macro_replaces_abbreviation_and_retypes_trigger() {
        let macros = BTreeMap::from([("bbc".to_string(), "BBC News".to_string())]);
        assert_eq!(type_keys(&loaded(macros), "bbc "), "BBC News ");
    }

    /// Types (vk, shift, char) triples with "capitalize sentence starts" on.
    fn type_vks(settings: Settings, keys: &[(u16, bool, char)]) -> String {
        let loaded = Loaded { settings, macros: BTreeMap::new() };
        let mut c = Composer::new(Settings::default().to_config(false));
        c.apply(&loaded);
        let mut field = String::new();
        for &(vk, shift, ch) in keys {
            c.engine.key_event(false, vk, shift as u8, false);
            match edit_for(&c.engine.hook, c.engine.cfg.code_table) {
                None => field.push(ch),
                Some(edit) => {
                    for _ in 0..edit.backspaces {
                        field.pop();
                    }
                    field.push_str(&edit.text);
                    if edit.resend_key {
                        field.push(ch);
                    }
                }
            }
        }
        field
    }

    #[test]
    fn each_capitalization_case_can_be_switched_off_on_its_own() {
        let word = |s: &str| s.chars().map(|c| (c.to_ascii_uppercase() as u16, false, c)).collect::<Vec<_>>();
        let after = |settings: Settings, end: (u16, bool, char), sep: Option<(u16, bool, char)>| {
            let mut keys = word("xin");
            keys.push(end);
            keys.extend(sep);
            keys.extend(word("chao"));
            type_vks(settings, &keys)
        };
        let on = || Settings { upper_case_first_char: true, ..Settings::default() };
        let (dot, question, exclaim, enter) = ((0xBE, false, '.'), (0xBF, true, '?'), (0x31, true, '!'), (0x0D, false, '\n'));
        let space = Some((0x20, false, ' '));
        assert_eq!(after(on(), dot, space), "xin. Chao");
        assert_eq!(after(on(), question, space), "xin? Chao");
        assert_eq!(after(on(), exclaim, space), "xin! Chao");
        assert_eq!(after(on(), enter, None), "xin\nChao");
        // A plain slash does not end a sentence.
        assert_eq!(after(on(), (0xBF, false, '/'), space), "xin/ chao");
        // One case off leaves the others working.
        assert_eq!(after(Settings { upper_case_after_punct: false, ..on() }, dot, space), "xin. chao");
        assert_eq!(after(Settings { upper_case_after_punct: false, ..on() }, question, space), "xin? chao");
        assert_eq!(after(Settings { upper_case_after_punct: false, ..on() }, exclaim, space), "xin! chao");
        assert_eq!(after(Settings { upper_case_after_punct: false, ..on() }, enter, None), "xin\nChao");
        assert_eq!(after(Settings { upper_case_after_enter: false, ..on() }, enter, None), "xin\nchao");
        // The master switch wins.
        assert_eq!(after(Settings::default(), dot, space), "xin. chao");
    }
}
