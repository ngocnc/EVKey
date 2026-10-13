//! The Vietnamese/English toggle hotkey: recognizes the chord, flips the shared mode, commits any open word first.
//! A letter chord (`alt+z`) fires on its key-down. A modifier-only chord (`ctrl+shift`) fires on the release of a
//! chord modifier, provided no other key went down since the chord's modifiers were all held together.
use std::rc::Rc;

use evkey_engine::hotkey::Hotkey;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT};
use windows::Win32::UI::TextServices::ITfContext;

use crate::mode::{self, ENGLISH, VIETNAMESE};
use crate::service::Core;
use crate::{log, sinks};

/// Held chord modifiers in `Hotkey` field order: ctrl, alt, shift, win.
fn held() -> [bool; 4] {
    [
        sinks::held(VK_CONTROL),
        sinks::held(VK_MENU),
        sinks::held(VK_SHIFT),
        sinks::held(VK_LWIN) || sinks::held(VK_RWIN),
    ]
}

fn wanted(hk: &Hotkey) -> [bool; 4] {
    [hk.ctrl, hk.alt, hk.shift, hk.win]
}

/// Index into `wanted` of a modifier virtual key; `None` for every other key.
fn modifier_index(vk: u16) -> Option<usize> {
    match vk {
        0x11 | 0xA2 | 0xA3 => Some(0), // Ctrl
        0x12 | 0xA4 | 0xA5 => Some(1), // Alt
        0x10 | 0xA0 | 0xA1 => Some(2), // Shift
        0x5B | 0x5C => Some(3),        // Win
        _ => None,
    }
}

/// True when the shared mode is English. Read from the global compartment, so changes by other processes count.
pub fn english(core: &Core) -> bool {
    core.tm.borrow().as_ref().is_some_and(|tm| mode::read(tm) == ENGLISH)
}

/// Tracks the modifier-only chord: a modifier key-down arms it when exactly the chord's modifiers are held,
/// any other key disarms it. Called for every key-down (TSF may call only the test or only the real callback).
pub fn note_key_down(core: &Core, vk: u16) {
    let Some(hk) = core.hotkey.get() else { return };
    if hk.vk.is_some() {
        return;
    }
    match modifier_index(vk) {
        None => core.chord_armed.set(false),
        Some(_) => core.chord_armed.set(held() == wanted(&hk)),
    }
}

/// True when this key-down is a letter chord (`alt+z`) with exactly its modifiers held.
pub fn is_key_chord(core: &Core, vk: u16) -> bool {
    core.hotkey.get().is_some_and(|hk| hk.vk == Some(vk) && held() == wanted(&hk))
}

/// Pure check for `OnTestKeyUp`: would releasing `vk` complete an armed modifier-only chord?
pub fn chord_release_fires(core: &Core, vk: u16) -> bool {
    let Some(hk) = core.hotkey.get() else { return false };
    hk.vk.is_none() && core.chord_armed.get() && modifier_index(vk).is_some_and(|i| wanted(&hk)[i])
}

/// Modifier release that completes a modifier-only chord: toggles and disarms. The release itself is not eaten.
pub fn chord_release(core: &Rc<Core>, ctx: Option<&ITfContext>, vk: u16) {
    if chord_release_fires(core, vk) {
        core.chord_armed.set(false);
        toggle(core, ctx);
    }
}

/// Flips the shared mode. An open word is committed first, so the flip never lands mid-composition.
pub fn toggle(core: &Rc<Core>, ctx: Option<&ITfContext>) {
    let Some(tm) = core.tm.borrow().clone() else { return };
    let next = if mode::read(&tm) == ENGLISH { VIETNAMESE } else { ENGLISH };
    if let Some(ctx) = ctx {
        sinks::end_composition(core, ctx);
    }
    core.composer.borrow_mut().reset();
    match mode::write(&tm, core.tid.get(), next) {
        Ok(()) => log::line(&format!("mode -> {}", if next == ENGLISH { "English" } else { "Vietnamese" })),
        Err(e) => log::line(&format!("mode write failed: {e}")),
    }
}
