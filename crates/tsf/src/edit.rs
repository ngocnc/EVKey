//! Edit session: applies one engine `Action` to the composition inside a synchronous TSF edit cookie.
use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::Rc;

use evkey_engine::composer::Action;
use windows::core::{implement, ComObject, Interface, Result};
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::TextServices::{
 GUID_PROP_ATTRIBUTE, ITfComposition, ITfContext, ITfContextComposition, ITfEditSession, ITfEditSession_Impl,
 ITfRange, TF_ANCHOR_END, TF_AE_NONE, TF_DEFAULT_SELECTION, TF_ES_READWRITE, TF_ES_SYNC, TF_SELECTION,
 TF_SELECTIONSTYLE,
};

use crate::service::Core;

#[implement(ITfEditSession)]
pub struct Edit {
    core: Rc<Core>,
    ctx: ITfContext,
    action: RefCell<Option<Action>>,
}

impl ITfEditSession_Impl for Edit_Impl {
 fn DoEditSession(&self, ec: u32) -> Result<()> {
 match self.action.take() {
 Some(Action::Compose(t)) => apply(&self.core, &self.ctx, ec, &t, false),
 Some(Action::Commit { text, .. }) => apply(&self.core, &self.ctx, ec, &text, true),
 // Pass while composing: close the composition and keep its text, so a host caret move can't land inside it.
 _ => end_keep(&self.core, ec),
 }
 }
}

/// Runs `action` inside a synchronous read/write edit session on `ctx`.
pub fn request(core: &Rc<Core>, ctx: &ITfContext, action: Action) -> Result<()> {
    let es: ITfEditSession = ComObject::new(Edit {
        core: core.clone(),
        ctx: ctx.clone(),
        action: RefCell::new(Some(action)),
    })
    .to_interface();
    unsafe { ctx.RequestEditSession(core.tid.get(), &es, TF_ES_SYNC | TF_ES_READWRITE)?.ok() }
}

fn apply(core: &Core, ctx: &ITfContext, ec: u32, text: &str, end: bool) -> Result<()> {
    let cur = core.comp.borrow().clone();
    // A composition the host already dropped fails GetRange; start a fresh one instead.
    let (comp, range) = match cur.and_then(|c| unsafe { c.GetRange() }.ok().map(|r| (c, r))) {
        Some(x) => x,
        None if text.is_empty() => return Ok(()),
        None => start(core, ctx, ec)?,
    };

    let buf: Vec<u16> = text.encode_utf16().collect();
    unsafe { range.SetText(ec, 0, &buf)? };
    if !end && !text.is_empty() {
        unsafe {
            let prop = ctx.GetProperty(&GUID_PROP_ATTRIBUTE)?;
            prop.SetValue(ec, &range, &VARIANT::from(core.da_atom.get() as i32))?;
        }
    }

    unsafe { range.Collapse(ec, TF_ANCHOR_END)? };
    let mut sel = TF_SELECTION {
        range: ManuallyDrop::new(Some(range.clone())),
        style: TF_SELECTIONSTYLE { ase: TF_AE_NONE, fInterimChar: false.into() },
    };
    let r = unsafe { ctx.SetSelection(ec, std::slice::from_ref(&sel)) };
    unsafe { ManuallyDrop::drop(&mut sel.range) };
    r?;

    if end {
        let r = unsafe { comp.EndComposition(ec) };
        core.comp.borrow_mut().take();
        r
    } else {
        *core.comp.borrow_mut() = Some(comp);
        Ok(())
    }
}

/// Closes the active composition without changing its text.
fn end_keep(core: &Core, ec: u32) -> Result<()> {
 let comp = core.comp.borrow_mut().take(); // release the borrow: EndComposition can re-enter the sink
 match comp {
 Some(comp) => unsafe { comp.EndComposition(ec) },
 None => Ok(()),
 }
}

/// Starts a composition over the current selection.
fn start(core: &Core, ctx: &ITfContext, ec: u32) -> Result<(ITfComposition, ITfRange)> {
    let mut sel = [TF_SELECTION::default()];
    let mut fetched = 0u32;
    let range = unsafe {
        ctx.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?;
        ManuallyDrop::take(&mut sel[0].range)
    }
    .ok_or(E_FAIL)?;
    let sink = core.sink.borrow().clone().ok_or(E_FAIL)?;
    let cc: ITfContextComposition = ctx.cast()?;
    let comp = unsafe { cc.StartComposition(ec, &range, &sink)? };
    let range = unsafe { comp.GetRange()? };
    Ok((comp, range))
}
