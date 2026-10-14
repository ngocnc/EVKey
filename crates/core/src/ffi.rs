//! C ABI for non-Rust hosts (Swift on macOS). Header: `crates/core/include/evkey_core.h`, kept in sync by a test.
//! Every entry point is wrapped in `catch_unwind`: a Rust panic becomes a `false`/`NULL` return, never an unwind
//! across the C boundary. Strings returned as `char*` are owned by the caller and released with `evkey_string_free`.
use std::collections::BTreeMap;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

use evkey_engine::convert;
use evkey_engine::hotkey::parse_hotkey;
use evkey_engine::settings::{self, Loaded, Settings};
use serde::{Deserialize, Serialize};

use crate::session::{KeyInput, Mods, Resend, Session};

const RESEND_NONE: u8 = 0;
const RESEND_CHAR: u8 = 1;
const RESEND_VK: u8 = 2;

/// Flattened [`crate::session::Reaction`]. `text_ptr`/`text_len` are UTF-16 units owned by the session and valid
/// until the next call on that session.
#[repr(C)]
pub struct EvReaction {
    pub swallow: bool,
    pub hotkey_toggled: bool,
    pub dummy_key: bool,
    pub translate_requested: bool,
    pub actions_requested: bool,
    pub has_edit: bool,
    pub prefix_empty_char: bool,
    pub backspaces: u32,
    /// `EV_RESEND_NONE`, `EV_RESEND_CHAR` (`resend_value` is a UTF-16 unit) or `EV_RESEND_VK` (a virtual-key code).
    pub resend_kind: u8,
    pub resend_value: u16,
    pub text_ptr: *const u16,
    pub text_len: usize,
}

/// `config.toml` + `macros.toml` as JSON (`settings_load_json` / `settings_save_json`).
#[derive(Serialize, Deserialize)]
struct SettingsDoc {
    settings: Settings,
    macros: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct LoadDoc {
    settings: Settings,
    macros: BTreeMap<String, String>,
    error: Option<String>,
}

/// Runs `f`; a panic yields `fallback` instead of unwinding into C.
fn guard<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(fallback)
}

/// `NULL` means the default settings directory.
unsafe fn dir_arg(p: *const c_char) -> Option<PathBuf> {
    if p.is_null() {
        return settings::default_dir();
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok().map(PathBuf::from)
}

unsafe fn str_arg<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

fn into_c_string(s: String) -> *mut c_char {
    CString::new(s).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}

/// Bit 0 ctrl, bit 1 alt, bit 2 shift, bit 3 win.
fn mods_from_bits(bits: u8) -> Mods {
    Mods { ctrl: bits & 1 != 0, alt: bits & 2 != 0, shift: bits & 4 != 0, win: bits & 8 != 0 }
}

/// Creates a session. `dir_utf8` is the settings directory (`NULL` = default). Free with `evkey_session_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_new(dir_utf8: *const c_char) -> *mut Session {
    guard(std::ptr::null_mut(), || {
        let dir = unsafe { dir_arg(dir_utf8) };
        Box::into_raw(Box::new(Session::new(dir)))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_free(session: *mut Session) {
    if !session.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(unsafe { Box::from_raw(session) })));
    }
}

/// Handles one key event (Windows virtual-key code). Returns false if `session` or `out` is `NULL`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_on_key(
    session: *mut Session,
    vk: u16,
    down: bool,
    mods: u8,
    caps_lock: bool,
    out: *mut EvReaction,
) -> bool {
    guard(false, || {
        let (Some(s), false) = (unsafe { session.as_mut() }, out.is_null()) else { return false };
        let r = s.on_key(KeyInput { vk, down, mods: mods_from_bits(mods), caps_lock });
        s.text16.clear();
        let (has_edit, prefix_empty_char, backspaces, resend) = match &r.edit {
            Some(o) => {
                s.text16 = o.text.encode_utf16().collect();
                (true, o.prefix_empty_char, o.backspaces as u32, o.resend)
            }
            None => (false, false, 0, None),
        };
        let (resend_kind, resend_value) = match resend {
            None => (RESEND_NONE, 0),
            Some(Resend::Char(c)) => (RESEND_CHAR, c),
            Some(Resend::Vk(v)) => (RESEND_VK, v),
        };
        let ev = EvReaction {
            swallow: r.swallow,
            hotkey_toggled: r.hotkey_toggled,
            dummy_key: r.dummy_key,
            translate_requested: r.translate_requested,
            actions_requested: r.actions_requested,
            has_edit,
            prefix_empty_char,
            backspaces,
            resend_kind,
            resend_value,
            text_ptr: s.text16.as_ptr(),
            text_len: s.text16.len(),
        };
        unsafe { out.write(ev) };
        true
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_mouse_click(session: *mut Session) {
    guard((), || {
        if let Some(s) = unsafe { session.as_mut() } {
            s.on_mouse_click();
        }
    })
}

/// `app_id_utf8` is the exe name (Windows) or bundle ID (macOS); `NULL` when unknown.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_set_foreground(session: *mut Session, app_id_utf8: *const c_char) {
    guard((), || {
        if let Some(s) = unsafe { session.as_mut() } {
            s.set_foreground(unsafe { str_arg(app_id_utf8) });
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_set_secure(session: *mut Session, on: bool) {
    guard((), || {
        if let Some(s) = unsafe { session.as_mut() } {
            s.set_secure(on);
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_is_english(session: *mut Session) -> bool {
    guard(false, || unsafe { session.as_ref() }.is_some_and(|s| s.is_english()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_set_english(session: *mut Session, on: bool) {
    guard((), || {
        if let Some(s) = unsafe { session.as_mut() } {
            s.set_english(on);
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_session_toggle(session: *mut Session) {
    guard((), || {
        if let Some(s) = unsafe { session.as_mut() } {
            s.toggle();
        }
    })
}

/// `%APPDATA%\EVKey` or `~/Library/Application Support/EVKey`; `NULL` if unknown. Free with `evkey_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_default_dir() -> *mut c_char {
    guard(std::ptr::null_mut(), || {
        settings::default_dir().and_then(|p| p.into_os_string().into_string().ok()).map_or(std::ptr::null_mut(), into_c_string)
    })
}

/// Loads settings as `{"settings":{...},"macros":{...},"error":null|"message"}`. On error the settings are defaults
/// and must not be saved back. Free with `evkey_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_settings_load_json(dir_utf8: *const c_char) -> *mut c_char {
    guard(std::ptr::null_mut(), || {
        let doc = match unsafe { dir_arg(dir_utf8) }.map(|d| settings::load(&d)) {
            Some(Ok(loaded)) => {
                LoadDoc { settings: loaded.settings, macros: loaded.macros, error: None }
            }
            Some(Err(e)) => LoadDoc { settings: Settings::default(), macros: BTreeMap::new(), error: Some(e) },
            None => LoadDoc {
                settings: Settings::default(),
                macros: BTreeMap::new(),
                error: Some("no settings directory".into()),
            },
        };
        into_c_string(serde_json::to_string(&doc).unwrap_or_else(|_| "{}".into()))
    })
}

/// Saves `{"settings":{...},"macros":{...}}` JSON. Validates like the app: hotkey must parse, banner values must be
/// valid, macro keys are trimmed and must be non-empty and unique. Returns `NULL` on success, else the error message
/// (free with `evkey_string_free`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_settings_save_json(dir_utf8: *const c_char, json_utf8: *const c_char) -> *mut c_char {
    guard(std::ptr::null_mut(), || {
        let err = |msg: String| into_c_string(msg);
        let Some(dir) = (unsafe { dir_arg(dir_utf8) }) else { return err("no settings directory".into()) };
        let Some(json) = (unsafe { str_arg(json_utf8) }) else { return err("invalid JSON argument".into()) };
        let doc: SettingsDoc = match serde_json::from_str(json) {
            Ok(d) => d,
            Err(e) => return err(format!("settings JSON: {e}")),
        };
        if let Err(e) = parse_hotkey(&doc.settings.toggle_hotkey) {
            return err(format!("Phím tắt không hợp lệ: {e}"));
        }
        let mut macros = BTreeMap::new();
        for (key, value) in doc.macros {
            let key = key.trim().to_string();
            if key.is_empty() {
                return err("Từ viết tắt không được để trống.".into());
            }
            if macros.insert(key.clone(), value).is_some() {
                return err(format!("Từ viết tắt bị trùng: {key}"));
            }
        }
        match settings::save(&dir, &Loaded { settings: doc.settings, macros }) {
            Ok(()) => std::ptr::null_mut(),
            Err(e) => err(e),
        }
    })
}

/// Converts `text_utf8` to precomposed Unicode, guessing its code table (the action panel's Convert). `NULL` on bad
/// input. Free with `evkey_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_convert_to_unicode(text_utf8: *const c_char) -> *mut c_char {
    guard(std::ptr::null_mut(), || {
        let Some(text) = (unsafe { str_arg(text_utf8) }) else { return std::ptr::null_mut() };
        into_c_string(convert::to_unicode(text, convert::detect(text)))
    })
}

/// Releases a string returned by this library. `NULL` is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn evkey_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(unsafe { CString::from_raw(s) })));
    }
}

/// Library version as a static NUL-terminated string. Do not free.
#[unsafe(no_mangle)]
pub extern "C" fn evkey_version() -> *const c_char {
    const VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr().cast()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Feeds one key through the C entry point and applies the reply to `field` the way a host would: type the
    /// empty-char prefix, delete `backspaces`, type the UTF-16 text, then the resend key. Without an edit the key
    /// passes through and is typed as-is.
    unsafe fn feed(s: *mut Session, field: &mut String, ch: char) {
        let mut ev = std::mem::MaybeUninit::<EvReaction>::uninit();
        assert!(unsafe { evkey_session_on_key(s, ch.to_ascii_uppercase() as u16, true, 0, false, ev.as_mut_ptr()) });
        let ev = unsafe { ev.assume_init() };
        if !ev.has_edit {
            if !ev.swallow {
                field.push(ch);
            }
            return;
        }
        if ev.prefix_empty_char {
            field.push('\u{202F}');
        }
        for _ in 0..ev.backspaces {
            field.pop();
        }
        field.push_str(&String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ev.text_ptr, ev.text_len) }));
        match ev.resend_kind {
            RESEND_CHAR => field.push(char::from_u32(ev.resend_value as u32).unwrap_or('?')),
            RESEND_VK => field.push(ch),
            _ => {}
        }
    }

    #[test]
    fn round_trip_types_vietnamese_through_the_c_abi() {
        let dir = std::env::temp_dir().join(format!("evkey-ffi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let c = CString::new(dir.to_str().unwrap()).unwrap();
        unsafe {
            let s = evkey_session_new(c.as_ptr());
            assert!(!s.is_null());
            let mut field = String::new();
            for ch in "vieejt".chars() {
                feed(s, &mut field, ch);
            }
            assert_eq!(field, "việt");
            evkey_session_free(s);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn header_declares_every_exported_symbol() {
        let header = include_str!("../include/evkey_core.h");
        let source = include_str!("ffi.rs");
        let mut exported = 0;
        for line in source.lines() {
            let Some(rest) = line.strip_prefix("pub unsafe extern \"C\" fn ").or(line.strip_prefix("pub extern \"C\" fn "))
            else {
                continue;
            };
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            assert!(header.contains(&format!("{name}(")), "header is missing {name}");
            exported += 1;
        }
        assert_eq!(exported, 15, "update the header test when adding or removing exports");
    }
}
