//! One object implementing the key, composition and thread-manager sinks; shares `Core` with `TextService`.
use std::rc::Rc;

use evkey_engine::composer::{is_modifier_key, Action};
use evkey_engine::settings::TempOffKey;
use windows::core::{implement, Ref, Result, BOOL, GUID};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
 GetKeyState, GetKeyboardState, MapVirtualKeyW, ToUnicode, VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT, MAPVK_VK_TO_VSC,
};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfDocumentMgr,
    ITfKeyEventSink, ITfKeyEventSink_Impl, ITfThreadMgrEventSink, ITfThreadMgrEventSink_Impl,
};

use crate::service::Core;
use crate::{edit, guard, log, secure, toggle, Lock};
use crate::guids::GUID_PRESERVED_TOGGLE;

/// lparam bit 30: the key was already down, so this key-down is an auto-repeat.
const REPEAT_BIT: isize = 1 << 30;

#[implement(ITfKeyEventSink, ITfCompositionSink, ITfThreadMgrEventSink)]
pub struct Sinks {
    core: Rc<Core>,
    _lock: Lock,
}

impl Sinks {
    pub fn new(core: Rc<Core>) -> Self {
        Self { core, _lock: Lock::new() }
    }
}

impl ITfKeyEventSink_Impl for Sinks_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        self.core.lose();
        Ok(())
    }

    fn OnTestKeyDown(&self, pic: Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(guard(false, || pic.as_ref().is_some() && wants(&self.core, wparam.0 as u16)).into())
    }

    fn OnTestKeyUp(&self, _pic: Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(guard(false, || toggle::chord_release_fires(&self.core, wparam.0 as u16)).into())
    }

    fn OnKeyDown(&self, pic: Ref<ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        let repeat = lparam.0 & REPEAT_BIT != 0;
        Ok(guard(false, || key_down(&self.core, pic.as_ref(), wparam.0 as u16, repeat)).into())
    }

    fn OnKeyUp(&self, pic: Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        guard((), || toggle::chord_release(&self.core, pic.as_ref(), wparam.0 as u16));
        Ok(BOOL(0))
    }

    fn OnPreservedKey(&self, pic: Ref<ITfContext>, rguid: *const GUID) -> Result<BOOL> {
        Ok(guard(BOOL(0), || {
            if unsafe { *rguid } != GUID_PRESERVED_TOGGLE {
                return BOOL(0);
            }
            let ctx = pic.as_ref().cloned().or_else(|| self.core.focus_context());
            toggle::toggle(&self.core, ctx.as_ref());
            BOOL(1)
        }))
    }
}

impl ITfCompositionSink_Impl for Sinks_Impl {
    fn OnCompositionTerminated(&self, _ecwrite: u32, _pcomposition: Ref<ITfComposition>) -> Result<()> {
        self.core.lose();
        Ok(())
    }
}

impl ITfThreadMgrEventSink_Impl for Sinks_Impl {
    fn OnInitDocumentMgr(&self, _pdim: Ref<ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnUninitDocumentMgr(&self, _pdim: Ref<ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnSetFocus(&self, _pdimfocus: Ref<ITfDocumentMgr>, _pdimprevfocus: Ref<ITfDocumentMgr>) -> Result<()> {
        self.core.lose();
        Ok(())
    }

    fn OnPushContext(&self, _pic: Ref<ITfContext>) -> Result<()> {
        Ok(())
    }

    fn OnPopContext(&self, _pic: Ref<ITfContext>) -> Result<()> {
        Ok(())
    }
}

/// Per-key housekeeping shared by `OnTestKeyDown` and `OnKeyDown` (TSF may call only the latter):
/// pick up edited settings, honour the exclusion list, handle the temporary-off key.
/// Returns false when the key must be ignored.
fn prepare(core: &Core, vk: u16) -> bool {
    core.refresh();
    if core.excluded.get() {
        return false;
    }
    let tap = match core.temp_off_key.get() {
        TempOffKey::Ctrl => matches!(vk, 0x11 | 0xA2 | 0xA3),
        TempOffKey::Alt => matches!(vk, 0x12 | 0xA4 | 0xA5),
        TempOffKey::None => false,
    };
    if tap {
        core.composer.borrow_mut().temp_off();
    }
    true
}

/// Ctrl/Alt/Win shortcuts are eaten only mid-composition, so Ctrl+A/Ctrl+C still reach the app otherwise.
fn wants(core: &Core, vk: u16) -> bool {
    core.refresh();
    toggle::note_key_down(core, vk);
    if toggle::is_key_chord(core, vk) {
        return true;
    }
    if toggle::english(core) {
        // Only true when a mode change made elsewhere left a composition open; key_down closes it.
        return core.comp.borrow().is_some();
    }
    if !prepare(core, vk) {
        return false;
    }
    let c = core.composer.borrow();
    c.wants_key(vk) && (c.is_composing() || !ctrl_alt_held())
}

pub fn held(k: VIRTUAL_KEY) -> bool {
    unsafe { GetKeyState(k.0 as i32) < 0 }
}

fn ctrl_alt_held() -> bool {
    held(VK_CONTROL) || held(VK_MENU) || held(VK_LWIN) || held(VK_RWIN)
}

/// Closes an open composition, keeping its text, so the host owns the keys.
pub fn end_composition(core: &Rc<Core>, ctx: &ITfContext) {
    if core.comp.borrow().is_some() {
        core.composer.borrow_mut().reset();
        if let Err(e) = edit::request(core, ctx, Action::Pass) {
            log::line(&format!("end composition failed: {e}"));
        }
    }
}

/// The printable character `vk` types under the current layout, Shift and CapsLock; `None` otherwise.
fn typed_char(vk: u16) -> Option<char> {
    let mut state = [0u8; 256];
    if unsafe { GetKeyboardState(&mut state) }.is_err() {
        return None;
    }
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) };
    let mut buf = [0u16; 4];
    let n = unsafe { ToUnicode(vk as u32, scan, Some(&state), &mut buf, 0) };
    if n != 1 {
        return None;
    }
    char::from_u32(buf[0] as u32).filter(|c| *c >= ' ')
}

/// Feeds one key to the engine and applies the resulting `Action`. Returns whether the key is eaten.
fn key_down(core: &Rc<Core>, ctx: Option<&ITfContext>, vk: u16, repeat: bool) -> bool {
    core.refresh();
    toggle::note_key_down(core, vk);
    if toggle::is_key_chord(core, vk) {
        if !repeat {
            toggle::toggle(core, ctx);
        }
        return true;
    }
    if toggle::english(core) {
        if let Some(ctx) = ctx {
            end_composition(core, ctx);
        }
        return false;
    }
    if !prepare(core, vk) {
        return false;
    }
    // Shift/Ctrl/Alt alone never end a composition (e.g. Shift before a capital mid-word).
    if is_modifier_key(vk) {
        return false;
    }
    let Some(ctx) = ctx else { return false };
    if secure::is_secure(core, ctx) {
        log::line("keydown in secure field: passed to host");
        end_composition(core, ctx);
        return false;
    }
    let caps = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
    let action = core.composer.borrow_mut().key_down(vk, held(VK_SHIFT), caps, ctrl_alt_held());
    // A printable key that ends a word is committed with the word and eaten. Hosts drop such a key
    // right after the composition ends (WinForms TextBox lost the space), so it must not reach the host.
    let action = match action {
        Action::Commit { text, eaten: false } if !ctrl_alt_held() => match typed_char(vk) {
            Some(c) => Action::Commit { text: text + &c.to_string(), eaten: true },
            None => Action::Commit { text, eaten: false },
        },
        other => other,
    };
    let eaten = match &action {
        Action::Pass => {
            // Host key (e.g. arrows) while composing: close the composition first, keeping its text.
            end_composition(core, ctx);
            return false;
        }
        Action::Compose(_) => true,
        Action::Commit { eaten, .. } => *eaten,
    };
    log::line(&format!("keydown vk={vk:#04x} eaten={eaten}"));
    if let Err(e) = edit::request(core, ctx, action) {
        log::line(&format!("edit session failed: {e}"));
    }
    eaten
}
