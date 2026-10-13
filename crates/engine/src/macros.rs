//! Port of `engine/Macro.cpp`: macro table, text→key conversion, and
//! `findMacro` with auto-caps. Disk I/O (`initMacroMap`, `saveToFile`, ...)
//! is intentionally omitted. `vCodeTable` and `vAutoCapsMacro` are explicit
//! parameters.

use std::collections::BTreeMap;

use crate::consts::{CAPS_MASK, CHAR_CODE_MASK, PURE_CHARACTER_MASK};
use crate::tables::{get_character_code, key_code_to_character, tables};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MacroData {
    /// ex: "ms"
    pub macro_text: String,
    /// ex: "millisecond"
    pub macro_content: String,
    /// converted form of `macro_content`, computed with the code table active at the time
    pub macro_content_code: Vec<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct MacroMap {
    map: BTreeMap<Vec<u32>, MacroData>,
}

/// Port of `convert(const string&, vector<Uint32>&)`.
///
/// C++ `wstring` on Windows is UTF-16 (`codecvt_utf8_utf16`), so iterate UTF-16 units.
fn convert(s: &str, code_table: usize) -> Vec<u32> {
    let t = tables();
    let mut out = Vec::new();
    for unit in s.encode_utf16() {
        let ch = unit as u32;
        // find normal character first
        if let Some(&v) = t.character_map.get(&ch) {
            out.push(v);
            continue;
        }
        // find character which has tone/mark (searched in the Unicode table, index 0)
        let mut found = false;
        'search: for (code, list) in t.code_table[0].iter() {
            for (k, &v) in list.iter().enumerate() {
                if unit == v {
                    // C++ `_codeTable[vCodeTable][key][k]` (operator[] on a missing key is UB; 0 here)
                    let mapped = t.code_table[code_table]
                        .get(code)
                        .and_then(|l| l.get(k))
                        .copied()
                        .unwrap_or(0);
                    out.push(mapped as u32 | CHAR_CODE_MASK);
                    found = true;
                    break 'search;
                }
            }
        }
        if !found {
            // mark it as pure character
            out.push(ch | PURE_CHARACTER_MASK);
        }
    }
    out
}

/// Port of `modifyCaseUnicode`. Returns true when `code` changed.
fn modify_case_unicode(code: &mut u32, is_upper: bool, code_table: usize) -> bool {
    let before = *code;
    if before & CHAR_CODE_MASK == 0 {
        // normal char: C++ masks with CAPS_MASK for upper (quirk: clears all other bits)
        *code &= if is_upper { CAPS_MASK } else { !CAPS_MASK };
        return *code != before;
    }

    let table = &tables().code_table[code_table];
    for (key, list) in table.iter() {
        for (k, &entry) in list.iter().enumerate() {
            if before as u16 == entry {
                let mut idx = k;
                if idx % 2 == 0 && !is_upper {
                    idx += 1;
                } else if idx % 2 != 0 && is_upper {
                    idx -= 1;
                }
                // C++ indexes without a bounds check; out-of-range (odd-length list) is UB there.
                let v = table
                    .get(key)
                    .and_then(|l| l.get(idx))
                    .copied()
                    .unwrap_or(0);
                *code = v as u32 | CHAR_CODE_MASK;
                return *code != before;
            }
        }
    }
    false
}

impl MacroMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Port of `findMacro`. `key` is mutated (normalised via `getCharacterCode`) exactly as in C++.
    /// On success `out` receives the macro content code; on failure `out` is untouched.
    pub fn find(
        &self,
        key: &mut Vec<u32>,
        out: &mut Vec<u32>,
        code_table: usize,
        auto_caps_macro: bool,
    ) -> bool {
        for c in key.iter_mut() {
            *c = get_character_code(code_table, *c);
        }
        if let Some(data) = self.map.get(key) {
            out.clear();
            out.extend_from_slice(&data.macro_content_code);
            return true;
        }
        if auto_caps_macro {
            let mut macro_flag = false;
            if key.len() > 1 && modify_case_unicode(&mut key[1], false, code_table) {
                macro_flag = true;
                for c in key[2..].iter_mut() {
                    modify_case_unicode(c, false, code_table);
                }
            }

            if !key.is_empty() && modify_case_unicode(&mut key[0], false, code_table) {
                if let Some(data) = self.map.get(key) {
                    out.clear();
                    out.extend_from_slice(&data.macro_content_code);
                    let t = tables();
                    for c in 0..out.len() {
                        if c == 0 || macro_flag {
                            let k_char = key_code_to_character(out[c]);
                            if k_char != 0 {
                                let upper = ascii_toupper(k_char);
                                // C++ `_characterMap[_kChar]` (operator[] would insert 0 on a miss;
                                // unreachable: keyCodeToCharacter only yields keys of _characterMap)
                                out[c] = t.character_map.get(&(upper as u32)).copied().unwrap_or(0);
                                continue;
                            }
                            if out[c] & CHAR_CODE_MASK != 0 {
                                modify_case_unicode(&mut out[c], true, code_table);
                            }
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Port of `hasMacro`. `code_table` is needed because keys depend on the active code table.
    pub fn has(&self, macro_text: &str, code_table: usize) -> bool {
        self.map.contains_key(&convert(macro_text, code_table))
    }

    /// Port of `addMacro`. Editing an existing macro keeps its original `macro_text` (as C++ does).
    pub fn add(&mut self, macro_text: &str, macro_content: &str, code_table: usize) -> bool {
        let key = convert(macro_text, code_table);
        let content_code = convert(macro_content, code_table);
        match self.map.get_mut(&key) {
            None => {
                self.map.insert(
                    key,
                    MacroData {
                        macro_text: macro_text.to_owned(),
                        macro_content: macro_content.to_owned(),
                        macro_content_code: content_code,
                    },
                );
            }
            Some(data) => {
                data.macro_content = macro_content.to_owned();
                data.macro_content_code = content_code;
            }
        }
        true
    }

    /// Port of `deleteMacro`.
    pub fn delete(&mut self, macro_text: &str, code_table: usize) -> bool {
        self.map.remove(&convert(macro_text, code_table)).is_some()
    }

    /// Port of `onTableCodeChange`: recompute every `macro_content_code` for `code_table`.
    pub fn on_table_code_change(&mut self, code_table: usize) {
        for data in self.map.values_mut() {
            data.macro_content_code = convert(&data.macro_content, code_table);
        }
    }
}

/// C-locale `toupper` on a 16-bit key code: only ASCII a-z change.
fn ascii_toupper(c: u16) -> u16 {
    if (b'a' as u16..=b'z' as u16).contains(&c) {
        c - 32
    } else {
        c
    }
}
