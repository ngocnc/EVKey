//! The Vietnamese/English toggle hotkey (`toggle_hotkey` in `config.toml`), parsed once and shared by TSF and the app.
//!
//! Grammar (case-insensitive, `+`-separated, no spaces): zero or more of `ctrl|alt|shift|win`, then either
//! one key (`a`-`z`, `0`-`9`, `space`, `f1`-`f12`) or nothing. `cmd` is an alias of `win` (the Command key on
//! macOS). A chord with no key (`ctrl+shift`) fires when
//! exactly those modifiers were pressed together and released with no other key in between. `none` disables it.

/// A parsed toggle chord. `vk` is a Windows virtual-key code; `None` means a modifier-only chord.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    pub vk: Option<u16>,
}

/// Parses `s` (see the module docs). `"none"` gives `Ok(None)`; anything malformed is an error.
pub fn parse_hotkey(s: &str) -> Result<Option<Hotkey>, String> {
    let lower = s.to_ascii_lowercase();
    if lower == "none" {
        return Ok(None);
    }
    let mut hk = Hotkey { ctrl: false, alt: false, shift: false, win: false, vk: None };
    let parts: Vec<&str> = lower.split('+').collect();
    for (i, part) in parts.iter().enumerate() {
        let flag = match *part {
            "ctrl" => &mut hk.ctrl,
            "alt" => &mut hk.alt,
            "shift" => &mut hk.shift,
            "win" | "cmd" => &mut hk.win,
            key => {
                if i + 1 != parts.len() {
                    return Err(format!("key {key:?} must be last in hotkey {s:?}"));
                }
                hk.vk = Some(key_vk(key).ok_or_else(|| format!("unknown key {key:?} in hotkey {s:?}"))?);
                continue;
            }
        };
        if *flag {
            return Err(format!("duplicate modifier {part:?} in hotkey {s:?}"));
        }
        *flag = true;
    }
    Ok(Some(hk))
}

/// Virtual-key code of a single key name; `None` for anything outside the grammar.
fn key_vk(k: &str) -> Option<u16> {
    let b = k.as_bytes();
    if k == "space" {
        Some(0x20)
    } else if b.len() == 1 && b[0].is_ascii_lowercase() {
        Some(b[0].to_ascii_uppercase() as u16)
    } else if b.len() == 1 && b[0].is_ascii_digit() {
        Some(b[0] as u16)
    } else if let Some(n) = k.strip_prefix('f').and_then(|d| d.parse::<u16>().ok()) {
        (k == format!("f{n}") && (1..=12).contains(&n)).then(|| 0x70 + n - 1)
    } else {
        None
    }
}
