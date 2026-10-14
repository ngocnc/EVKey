//! Key-event types the host builds and reads, and the modifier-group helpers the session shares.
use evkey_engine::hotkey::Hotkey;

/// Modifier groups in `Hotkey` field order: ctrl, alt, shift, win.
pub(super) type Groups = [bool; 4];

/// Physical modifier state at event time, supplied by the host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

impl Mods {
    pub(super) fn groups(self) -> Groups {
        [self.ctrl, self.alt, self.shift, self.win]
    }
}

/// One key event. `vk` is a Windows virtual-key code (hosts map their native codes).
#[derive(Clone, Copy, Debug)]
pub struct KeyInput {
    pub vk: u16,
    pub down: bool,
    pub mods: Mods,
    pub caps_lock: bool,
}

/// What the host must type when a key was converted: an optional [`EMPTY_CHAR`], `backspaces` backspaces, `text`
/// as UTF-16 units, then the optional `resend`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub prefix_empty_char: bool,
    pub backspaces: usize,
    pub text: String,
    pub resend: Option<Resend>,
}

/// The trigger key typed again after the edit: a Unicode character, or the raw virtual key when it has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resend {
    Char(u16),
    Vk(u16),
}

/// The host's instructions for one event.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reaction {
    /// Do not pass the key to the focused app.
    pub swallow: bool,
    pub edit: Option<Output>,
    /// The Vietnamese/English mode flipped on this event.
    pub hotkey_toggled: bool,
    /// Alt-hotkey: inject a throwaway key (VK 0xFF down/up) so releasing Alt does not open the window menu.
    pub dummy_key: bool,
    /// The translate hotkey was pressed; the host translates the selected text.
    pub translate_requested: bool,
}

/// Group index of a modifier virtual key (the hook reports the left/right variants); `None` for other keys.
pub(super) fn group(vk: u16) -> Option<usize> {
    match vk {
        0xA2 | 0xA3 | 0x11 => Some(0),
        0xA4 | 0xA5 | 0x12 => Some(1),
        0xA0 | 0xA1 | 0x10 => Some(2),
        0x5B | 0x5C => Some(3),
        _ => None,
    }
}

/// True for the modifier virtual keys (either side). Hosts skip foreground and Secure Input checks for these.
pub fn is_modifier_vk(vk: u16) -> bool {
    group(vk).is_some()
}

pub(super) fn wanted(hk: &Hotkey) -> Groups {
    [hk.ctrl, hk.alt, hk.shift, hk.win]
}
