use evkey_engine::import::{decode_config, encode_config, parse_macros};
use evkey_engine::settings::{Loaded, Settings};

fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
    v.iter().map(|(k, t)| (k.to_string(), t.to_string())).collect()
}

#[test]
fn unikey_utf8_header_and_comments_are_skipped() {
    let text = ";[DO NOT DELETE THIS LINE]***version=1\n; a comment\n\nbtw:by the way\nvn:Việt Nam\n";
    assert_eq!(parse_macros(text.as_bytes()).unwrap(), pairs(&[("btw", "by the way"), ("vn", "Việt Nam")]));
}

#[test]
fn openkey_file_with_bom_and_crlf() {
    let text = "\u{feff};Compatible OpenKey Macro Data file for UniKey*** version=1 ***\r\nhcm:Thành phố Hồ Chí Minh\r\n";
    assert_eq!(parse_macros(text.as_bytes()).unwrap(), pairs(&[("hcm", "Thành phố Hồ Chí Minh")]));
}

#[test]
fn utf16_le_with_bom_is_decoded() {
    let text = "ko:không\nsr:sorry\n";
    let mut bytes = vec![0xFF, 0xFE];
    for u in text.encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    assert_eq!(parse_macros(&bytes).unwrap(), pairs(&[("ko", "không"), ("sr", "sorry")]));
}

#[test]
fn legacy_viqr_items_are_decoded_when_header_version_is_not_1() {
    // Keys go through the same charset conversion as the text (UniKey `addItem`), so `dd` becomes `đ`.
    let text = ";[DO NOT DELETE THIS LINE]***version=0\ntks:Cam o+n\nvn:Vie^.t Nam\ndd:dda`n\n";
    assert_eq!(
        parse_macros(text.as_bytes()).unwrap(),
        pairs(&[("tks", "Cam ơn"), ("vn", "Việt Nam"), ("đ", "đàn")])
    );
}

#[test]
fn headerless_file_is_plain_utf8_so_ordinary_text_is_kept() {
    assert_eq!(parse_macros(b"add:add dd\n").unwrap(), pairs(&[("add", "add dd")]));
}

#[test]
fn viqr_trailing_punctuation_is_not_a_tone() {
    assert_eq!(parse_macros(b"hi:Hello.\n").unwrap(), pairs(&[("hi", "Hello.")]));
}

#[test]
fn line_without_colon_is_an_error_naming_the_line() {
    let err = parse_macros(b"btw:by the way\nbroken line\n").unwrap_err();
    assert!(err.starts_with("line 2:"), "{err}");
}

#[test]
fn empty_key_is_an_error() {
    let err = parse_macros(b"ok:fine\n:no key\n").unwrap_err();
    assert!(err.starts_with("line 2:"), "{err}");
}

#[test]
fn invalid_utf8_is_an_error_naming_the_line() {
    let err = parse_macros(b"ok:fine\nbad:\xff\n").unwrap_err();
    assert!(err.starts_with("line 2:"), "{err}");
}

#[test]
fn config_round_trips_through_one_document() {
    let mut macros = std::collections::BTreeMap::new();
    macros.insert("btw".to_string(), "by the way".to_string());
    let loaded = Loaded {
        settings: Settings {
            toggle_hotkey: "ctrl+shift".into(),
            ..Settings::default()
        },
        macros,
    };
    let text = encode_config(&loaded).unwrap();
    assert!(text.contains("format_version = 1"), "{text}");
    assert_eq!(decode_config(&text).unwrap(), loaded);
}

#[test]
fn config_rejects_bad_hotkey_before_anything_is_written() {
    let text = "format_version = 1\n[settings]\ntoggle_hotkey = \"alt+\"\n[macros]\n";
    assert!(decode_config(text).is_err());
}

#[test]
fn config_rejects_unknown_format_version_and_unknown_keys() {
    assert!(decode_config("format_version = 2\n[settings]\n[macros]\n").is_err());
    assert!(decode_config("format_version = 1\nsecret = \"x\"\n[settings]\n[macros]\n").is_err());
}
