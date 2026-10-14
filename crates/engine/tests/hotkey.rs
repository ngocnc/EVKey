use evkey_engine::hotkey::{parse_hotkey, Hotkey};

fn hk(ctrl: bool, alt: bool, shift: bool, win: bool, vk: Option<u16>) -> Option<Hotkey> {
    Some(Hotkey { ctrl, alt, shift, win, vk })
}

#[test]
fn parses_key_chords() {
    assert_eq!(parse_hotkey("alt+z"), Ok(hk(false, true, false, false, Some(b'Z' as u16))));
    assert_eq!(parse_hotkey("Ctrl+Shift+F12"), Ok(hk(true, false, true, false, Some(0x7B))));
    assert_eq!(parse_hotkey("win+space"), Ok(hk(false, false, false, true, Some(0x20))));
    assert_eq!(parse_hotkey("cmd+shift"), Ok(hk(false, false, true, true, None)));
    assert_eq!(parse_hotkey("ctrl+5"), Ok(hk(true, false, false, false, Some(b'5' as u16))));
    assert_eq!(parse_hotkey("f1"), Ok(hk(false, false, false, false, Some(0x70))));
}

#[test]
fn parses_modifier_only_chords_and_none() {
    assert_eq!(parse_hotkey("ctrl+shift"), Ok(hk(true, false, true, false, None)));
    assert_eq!(parse_hotkey("alt+shift"), Ok(hk(false, true, true, false, None)));
    assert_eq!(parse_hotkey("NONE"), Ok(None));
}

#[test]
fn rejects_garbage() {
    for bad in [
        "", "z+alt", "alt+alt+z", "alt+", "ctrl+ctrl", "f13", "f0", "alt+zz", "alt+ z", "+z", "tab",
        "alt+shift+x+y", "ctrl++z",
    ] {
        assert!(parse_hotkey(bad).is_err(), "{bad:?} should be rejected");
    }
}
