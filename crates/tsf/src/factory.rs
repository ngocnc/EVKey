//! Class factory for the text service.
use std::ffi::c_void;

use windows::core::{implement, ComObject, Interface, IUnknown, Ref, Result, BOOL, GUID};
use windows::Win32::Foundation::CLASS_E_NOAGGREGATION;
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};

use crate::service::TextService;
use crate::Lock;

#[implement(IClassFactory)]
pub struct Factory {
    _lock: Lock,
}

impl Factory {
    pub fn new() -> Self {
        Self { _lock: Lock::new() }
    }
}

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(&self, outer: Ref<IUnknown>, riid: *const GUID, obj: *mut *mut c_void) -> Result<()> {
        if outer.as_ref().is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let ts: IUnknown = ComObject::new(TextService::new()).to_interface();
        unsafe { ts.query(riid, obj).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}
