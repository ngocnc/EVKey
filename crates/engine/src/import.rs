//! Import of user data: UniKey / OpenKey macro files and the combined EVKey config file.
//!
//! Macro file sources (read from the source code):
//! - UniKey, `unikey/mactab.cpp` (`readHeader`, `loadFromFile`, `addItem`) and `unikey/charset.cpp`
//!   (`VIQRCharset::nextInput`), https://github.com/fcitx/fcitx5-unikey (GPL-2.0). Header
//!   `;[DO NOT DELETE THIS LINE]***version=N`. `UKMACRO_VERSION_UTF8` is 1: items are UTF-8.
//!   Other versions mean legacy VIQR items. Deviation: UniKey also reads a headerless file as VIQR; we read it as
//!   UTF-8, because VIQR would turn ordinary text like `add` into `ađ`. Legacy headerless VIQR needs `version=0`.
//!   Each line is `key:text`, split at the first `:`.
//! - OpenKey, `Sources/OpenKey/engine/Macro.cpp` (`saveToFile`, `readFromFile`),
//!   https://github.com/tuyenvm/OpenKey. Writes `;Compatible OpenKey Macro Data file for UniKey*** version=1 ***`
//!   then UTF-8 `key:text` lines. Its reader skips line 1 unconditionally.
//! UTF-16 LE with BOM is not in either source; it is accepted because Notepad's "Unicode" save produces it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::hotkey::parse_hotkey;
use crate::settings::{Loaded, Settings};

/// `UKMACRO_VERSION_UTF8` in UniKey's `mactab.cpp`: a header with this version marks UTF-8 items.
const UNIKEY_UTF8_VERSION: u32 = 1;

/// Parses a macro file, detecting encoding and format. Returns `(key, text)` pairs in file order.
/// Blank lines and `;` comment lines are skipped. Any other line without a usable key is an error naming its line.
pub fn parse_macros(bytes: &[u8]) -> Result<Vec<(String, String)>, String> {
    let text = decode_text(bytes)?;
    let mut lines = text.lines().enumerate().peekable();
    // A header line sets the item encoding. Without a header the file is plain UTF-8.
    let mut viqr = false;
    if let Some((_, first)) = lines.peek() {
        if let Some(version) = header_version(first) {
            viqr = version != UNIKEY_UTF8_VERSION;
            lines.next();
        }
    }
    let mut out = Vec::new();
    for (index, line) in lines {
        let line_no = index + 1;
        if line.trim().is_empty() || line.starts_with(';') {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("line {line_no}: no ':' between macro key and text"))?;
        let (key, value) = if viqr {
            (viqr_decode(key), viqr_decode(value))
        } else {
            (key.to_owned(), value.to_owned())
        };
        let key = key.trim().to_owned();
        if key.is_empty() {
            return Err(format!("line {line_no}: empty macro key"));
        }
        out.push((key, value));
    }
    Ok(out)
}

/// Strips a UTF-16 LE or UTF-8 BOM and returns the text. Rejects anything that is not valid text.
fn decode_text(bytes: &[u8]) -> Result<String, String> {
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        if rest.len() % 2 != 0 {
            return Err("UTF-16 LE file has an odd number of bytes".into());
        }
        let units: Vec<u16> = rest.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        return String::from_utf16(&units).map_err(|e| format!("invalid UTF-16 LE text: {e}"));
    }
    let body = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    // Without this, UTF-16 without a BOM would parse as text with NUL characters in every key.
    if body.contains(&0) {
        return Err("file contains NUL bytes; save it as UTF-8 or UTF-16 LE with BOM".into());
    }
    std::str::from_utf8(body).map(str::to_owned).map_err(|e| {
        let line = body[..e.valid_up_to()].iter().filter(|&&b| b == b'\n').count() + 1;
        format!("line {line}: not valid UTF-8; save the file as UTF-8 or UTF-16 LE with BOM")
    })
}

/// The version in a `...***version=N...` header, read like UniKey's `readHeader`.
fn header_version(line: &str) -> Option<u32> {
    let (_, after) = line.split_once("***")?;
    let digits = after.trim_start().strip_prefix("version=")?;
    let end = digits.find(|c: char| !c.is_ascii_digit()).unwrap_or(digits.len());
    digits[..end].parse().ok()
}

/// Vowel + mark → precomposed vowel (lowercase). Marks: `^` circumflex, `(` breve, `+` or `*` horn.
const VIQR_MODS: [(char, char, char); 6] = [
    ('a', '^', 'â'),
    ('a', '(', 'ă'),
    ('e', '^', 'ê'),
    ('o', '^', 'ô'),
    ('o', '+', 'ơ'),
    ('u', '+', 'ư'),
];

/// Vowel → its five toned forms in VIQR tone order: `` ` `` `'` `?` `~` `.`.
const VIQR_TONES: [(char, [char; 5]); 12] = [
    ('a', ['à', 'á', 'ả', 'ã', 'ạ']),
    ('ă', ['ằ', 'ắ', 'ẳ', 'ẵ', 'ặ']),
    ('â', ['ầ', 'ấ', 'ẩ', 'ẫ', 'ậ']),
    ('e', ['è', 'é', 'ẻ', 'ẽ', 'ẹ']),
    ('ê', ['ề', 'ế', 'ể', 'ễ', 'ệ']),
    ('i', ['ì', 'í', 'ỉ', 'ĩ', 'ị']),
    ('o', ['ò', 'ó', 'ỏ', 'õ', 'ọ']),
    ('ô', ['ồ', 'ố', 'ổ', 'ỗ', 'ộ']),
    ('ơ', ['ờ', 'ớ', 'ở', 'ỡ', 'ợ']),
    ('u', ['ù', 'ú', 'ủ', 'ũ', 'ụ']),
    ('ư', ['ừ', 'ứ', 'ử', 'ữ', 'ự']),
    ('y', ['ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ']),
];

/// Decodes UniKey's legacy VIQR text, e.g. `Tie^'ng` → `Tiếng`, `dd` → `đ`, `\'` → `'`.
/// ponytail: simplified. Marks apply to the previous character only when it is a vowel. UniKey's word-state
/// heuristics (smartViqr, suspicious-vowel mode) and escapes such as `://` are not implemented.
fn viqr_decode(s: &str) -> String {
    let mut out: Vec<char> = Vec::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            out.extend(it.next());
            continue;
        }
        if matches!(c, 'd' | 'D') && matches!(it.peek(), Some('d' | 'D')) {
            it.next();
            out.push(if c == 'd' { 'đ' } else { 'Đ' });
            continue;
        }
        // A tone mark sits between the vowel and its coda (`Tie^'ng`); a trailing `.` or `?` is punctuation.
        let followed_by_letter = it.peek().is_some_and(|n| n.is_alphabetic());
        let mapped = out.last().and_then(|&base| match c {
            '^' | '(' | '+' | '*' => viqr_modify(base, c),
            '`' | '\'' | '?' | '~' | '.' if followed_by_letter => viqr_tone(base, c),
            _ => None,
        });
        match (mapped, out.last_mut()) {
            (Some(m), Some(slot)) => *slot = m,
            _ => out.push(c),
        }
    }
    out.into_iter().collect()
}

fn viqr_modify(base: char, mark: char) -> Option<char> {
    let mark = if mark == '*' { '+' } else { mark };
    let lower = base.to_ascii_lowercase();
    let &(_, _, result) = VIQR_MODS.iter().find(|(b, m, _)| *b == lower && *m == mark)?;
    matched_case(base, result)
}

fn viqr_tone(base: char, tone: char) -> Option<char> {
    let index = "`'?~.".chars().position(|t| t == tone)?;
    let lower = base.to_lowercase().next()?;
    let (_, forms) = VIQR_TONES.iter().find(|(b, _)| *b == lower)?;
    matched_case(base, forms[index])
}

/// Returns `result` uppercased when `base` is uppercase.
fn matched_case(base: char, result: char) -> Option<char> {
    if base.is_uppercase() {
        result.to_uppercase().next()
    } else {
        Some(result)
    }
}

/// Layout of the combined config file: `format_version`, then `[settings]` and `[macros]` tables.
/// Holds no secrets: the Gemini key is not stored in these files.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    format_version: u32,
    settings: Settings,
    macros: BTreeMap<String, String>,
}

pub const CONFIG_FORMAT_VERSION: u32 = 1;

/// Serialises `config.toml` and `macros.toml` into one TOML document.
pub fn encode_config(loaded: &Loaded) -> Result<String, String> {
    let file = ConfigFile {
        format_version: CONFIG_FORMAT_VERSION,
        settings: loaded.settings.clone(),
        macros: loaded.macros.clone(),
    };
    toml::to_string_pretty(&file).map_err(|e| e.to_string())
}

/// Parses and validates a document from `encode_config`, with the same checks `settings::load` applies
/// to each file. Nothing is written here, so the caller writes only a result that passed.
pub fn decode_config(text: &str) -> Result<Loaded, String> {
    let file: ConfigFile = toml::from_str(text).map_err(|e| e.to_string())?;
    if file.format_version != CONFIG_FORMAT_VERSION {
        return Err(format!(
            "unsupported config format_version {} (expected {CONFIG_FORMAT_VERSION})",
            file.format_version
        ));
    }
    parse_hotkey(&file.settings.toggle_hotkey).map_err(|e| format!("toggle_hotkey: {e}"))?;
    file.settings.validate()?;
    if file.macros.keys().any(|k| k.trim().is_empty()) {
        return Err("macros: an abbreviation cannot be empty".into());
    }
    Ok(Loaded {
        settings: file.settings,
        macros: file.macros,
    })
}
