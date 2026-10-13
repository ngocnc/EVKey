//! Secure-field detection: TSF reports each field's InputScope; password scopes mean EVKey stays out.
use std::cell::Cell;
use std::rc::Rc;

use windows::core::{implement, IUnknown, Interface, Result};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Variant::{VARIANT, VT_UNKNOWN};
use windows::Win32::UI::TextServices::{
    ITfContext, ITfEditSession, ITfEditSession_Impl, ITfInputScope, ITfReadOnlyProperty, GUID_PROP_INPUTSCOPE,
    TF_ES_READ, TF_ES_SYNC,
};

use crate::service::Core;

/// IS_PASSWORD, IS_NUMERIC_PASSWORD, and IS_PRIVATE (what Edge/Chromium report for `type=password`).
// ponytail: IS_PRIVATE can also mark non-password private fields; those are left to the host too.
const SECURE_SCOPES: [i32; 3] = [31, 63, 61];

/// True when the focused field of `ctx` is a password-type field. Reads InputScope synchronously.
pub fn is_secure(core: &Core, ctx: &ITfContext) -> bool {
    let Ok(prop) = (unsafe { ctx.GetAppProperty(&GUID_PROP_INPUTSCOPE) }) else { return false };
    let secure = Rc::new(Cell::new(false));
    let es: ITfEditSession = Check { ctx: ctx.clone(), prop, secure: secure.clone() }.into();
    let hr = unsafe { ctx.RequestEditSession(core.tid.get(), &es, TF_ES_SYNC | TF_ES_READ) };
    hr.is_ok_and(|hr| hr.is_ok()) && secure.get()
}

#[implement(ITfEditSession)]
struct Check {
    ctx: ITfContext,
    prop: ITfReadOnlyProperty,
    secure: Rc<Cell<bool>>,
}

impl ITfEditSession_Impl for Check_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let value = unsafe { self.ctx.GetStart(ec).and_then(|r| self.prop.GetValue(ec, &r)) };
        self.secure.set(value.is_ok_and(|v| scopes(&v).iter().any(|s| SECURE_SCOPES.contains(s))));
        Ok(())
    }
}

/// InputScope values carried by `v`; empty when it is not an `ITfInputScope`.
fn scopes(v: &VARIANT) -> Vec<i32> {
    if unsafe { v.Anonymous.Anonymous.vt } != VT_UNKNOWN {
        return Vec::new();
    }
    let Ok(unk) = IUnknown::try_from(v) else { return Vec::new() };
    let Ok(scope) = unk.cast::<ITfInputScope>() else { return Vec::new() };
    let mut ptr = std::ptr::null_mut();
    let mut n = 0u32;
    if unsafe { scope.GetInputScopes(&mut ptr, &mut n) }.is_err() {
        return Vec::new();
    }
    let vals = (0..n as usize).map(|i| unsafe { (*ptr.add(i)).0 }).collect();
    unsafe { CoTaskMemFree(Some(ptr as *const _)) };
    vals
}
