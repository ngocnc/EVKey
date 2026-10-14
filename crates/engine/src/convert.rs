//! Chuyển mã: converts Vietnamese text between code tables.
//!
//! Every map is derived from `tables().code_table`, so there is no hand-typed glyph data.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::settings::CodeTable;
use crate::tables::tables;

/// Reverse lookup for one encoding: single units and two-unit runs to a precomposed char.
#[derive(Default)]
struct Reverse {
    single: HashMap<u32, char>,
    pair: HashMap<(u32, u32), char>,
}

#[derive(Default)]
struct Maps {
    /// Base char + combining mark -> precomposed (Unicode and UnicodeCompound sources).
    compound: Reverse,
    tcvn3: Reverse,
    vni: Reverse,
    /// Precomposed Vietnamese letter -> plain ASCII letter of the same case.
    base: HashMap<char, char>,
    /// Precomposed Vietnamese letter -> its TCVN3 / VNI-Windows glyphs (one or two chars).
    tcvn3_enc: HashMap<char, String>,
    vni_enc: HashMap<char, String>,
}

static MAPS: LazyLock<Maps> = LazyLock::new(build);

fn build() -> Maps {
    let t = tables();
    let mut m = Maps::default();
    // Lowercase (odd index) first, so bytes shared by caps and lowercase resolve to lowercase.
    for lower_pass in [true, false] {
        for (&key, uni_units) in &t.code_table[0] {
            let letter = (key & 0xFF) as u8 as char;
            for (i, &u) in uni_units.iter().enumerate() {
                if (i % 2 == 1) != lower_pass || u < 0x80 {
                    continue;
                }
                let Some(ch) = char::from_u32(u as u32) else { continue };
                m.base.insert(
                    ch,
                    if i % 2 == 1 { letter.to_ascii_lowercase() } else { letter.to_ascii_uppercase() },
                );

                let unit = |table: usize| {
                    t.code_table[table].get(&key).and_then(|v| v.get(i)).copied().unwrap_or(0) as u32
                };
                let comp = unit(3);
                let mark = comp >> 13;
                let comp_base = char::from_u32(comp & 0x1FFF).unwrap_or('\0');
                if mark != 0 {
                    let mark_ch = t.unicode_compound_mark[mark as usize - 1] as u32;
                    m.compound.pair.insert((comp_base as u32, mark_ch), ch);
                }

                // TCVN3 marks on plain vowels are base letter + mark byte (A + 0xB8 = Á),
                // so the base letter carries the case. Other TCVN3 forms are one byte.
                let tcvn = unit(1);
                if tcvn != 0 {
                    if mark != 0 && comp_base.is_ascii_alphabetic() {
                        m.tcvn3.pair.insert((comp_base as u32, tcvn), ch);
                        m.tcvn3_enc.insert(ch, [comp_base, glyph(tcvn)].into_iter().collect());
                    } else {
                        single(&mut m.tcvn3, tcvn, ch);
                        m.tcvn3_enc.insert(ch, glyph(tcvn).to_string());
                    }
                }

                // VNI units are (low, high) bytes; high <= 32 means a single byte.
                let vni = unit(2);
                let (lo, hi) = (vni & 0xFF, vni >> 8);
                if hi > 32 {
                    m.vni.pair.insert((lo, hi), ch);
                    m.vni_enc.insert(ch, [glyph(lo), glyph(hi)].into_iter().collect());
                } else if vni != 0 {
                    single(&mut m.vni, lo, ch);
                    m.vni_enc.insert(ch, glyph(lo).to_string());
                }
            }
        }
    }
    m
}

// ponytail: a byte shared by two forms keeps the first insert (lowercase). Toned TCVN3
// capitals may share bytes with lowercase; a real fix needs the TCVN3 capital glyph set.
fn single(r: &mut Reverse, byte: u32, ch: char) {
    if byte >= 0x80 {
        r.single.entry(byte).or_insert(ch);
    }
}

/// A legacy code-table byte as the char a host types for it (U+0000-U+00FF).
fn glyph(byte: u32) -> char {
    char::from_u32(byte & 0xFF).unwrap_or('?')
}

/// Greedy longest-match rewrite. Returns the text, the number of replacements,
/// and how many non-ASCII chars were left unconverted.
fn rewrite(text: &str, r: &Reverse) -> (String, usize, usize) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut hits, mut left) = (0, 0);
    let mut i = 0;
    while i < chars.len() {
        if let Some(&c) = chars.get(i + 1).and_then(|&b| r.pair.get(&(chars[i] as u32, b as u32))) {
            out.push(c);
            hits += 1;
            i += 2;
            continue;
        }
        if let Some(&c) = r.single.get(&(chars[i] as u32)) {
            out.push(c);
            hits += 1;
        } else {
            out.push(chars[i]);
            if !chars[i].is_ascii() {
                left += 1;
            }
        }
        i += 1;
    }
    (out, hits, left)
}

/// Guesses the code table of `text`. Plain ASCII and anything ambiguous are `Unicode`.
pub fn detect(text: &str) -> CodeTable {
    if text.chars().any(|c| ('\u{0300}'..='\u{0323}').contains(&c)) {
        return CodeTable::UnicodeCompound;
    }
    // Â â Ê ê Ô ô are real Unicode letters, and their bytes are also legacy glyphs; text made only of them is Unicode.
    if text.chars().all(|c| c.is_ascii() || "ÂâÊêÔô".contains(c)) || text.chars().any(|c| c as u32 > 0xFF) {
        return CodeTable::Unicode;
    }
    // Legacy only when every non-ASCII char converts and at least two conversions happen.
    // If both TCVN3 and VNI fit, the input is ambiguous and stays Unicode.
    let fits = |r: &Reverse| matches!(rewrite(text, r), (_, hits, 0) if hits >= 2);
    match (fits(&MAPS.tcvn3), fits(&MAPS.vni)) {
        (true, false) => CodeTable::Tcvn3,
        (false, true) => CodeTable::VniWindows,
        _ => CodeTable::Unicode,
    }
}

/// Converts `text` from `from` to precomposed Unicode (NFC for Vietnamese letters).
pub fn to_unicode(text: &str, from: CodeTable) -> String {
    let r = match from {
        CodeTable::Unicode | CodeTable::UnicodeCompound => &MAPS.compound,
        CodeTable::Tcvn3 => &MAPS.tcvn3,
        CodeTable::VniWindows => &MAPS.vni,
    };
    rewrite(text, r).0
}

/// Removes tone and vowel marks, keeps case, and maps đ/Đ to d/D.
pub fn strip_marks(text: &str) -> String {
    to_unicode(text, CodeTable::Unicode)
        .chars()
        .map(|c| MAPS.base.get(&c).copied().unwrap_or(c))
        .collect()
}

/// Converts `text` (any code table) to `to`. Unicode and Unicode compound targets give precomposed Unicode.
pub fn convert(text: &str, to: CodeTable) -> String {
    let unicode = to_unicode(text, detect(text));
    let enc = match to {
        CodeTable::Tcvn3 => &MAPS.tcvn3_enc,
        CodeTable::VniWindows => &MAPS.vni_enc,
        CodeTable::Unicode | CodeTable::UnicodeCompound => return unicode,
    };
    let mut out = String::with_capacity(unicode.len());
    for c in unicode.chars() {
        match enc.get(&c) {
            Some(s) => out.push_str(s),
            None => out.push(c),
        }
    }
    out
}

/// Text tool `id` of the action panel applied to `text`; `None` for an unknown id. Case tools expect Unicode text.
pub fn transform(id: &str, text: &str) -> Option<String> {
    Some(match id {
        "convert" => convert(text, CodeTable::Unicode),
        "convert_tcvn3" => convert(text, CodeTable::Tcvn3),
        "convert_vni" => convert(text, CodeTable::VniWindows),
        "strip_marks" => strip_marks(&to_unicode(text, detect(text))),
        "case_upper" => text.to_uppercase(),
        "case_lower" => text.to_lowercase(),
        "case_title" => {
            let mut start = true;
            text.chars()
                .flat_map(|c| {
                    let up = start;
                    start = c.is_whitespace();
                    let v: Vec<char> = if up { c.to_uppercase().collect() } else { c.to_lowercase().collect() };
                    v
                })
                .collect()
        }
        _ => return None,
    })
}
