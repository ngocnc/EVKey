//! Display attribute: plain text, no underline or colours. Enumerator yields that attribute once.
use std::cell::Cell;

use windows::core::{implement, BSTR, ComObject, Result, GUID};
use windows::Win32::UI::TextServices::{
    IEnumTfDisplayAttributeInfo, IEnumTfDisplayAttributeInfo_Impl, ITfDisplayAttributeInfo,
    ITfDisplayAttributeInfo_Impl, TF_ATTR_INPUT, TF_DISPLAYATTRIBUTE, TF_LS_NONE,
};

use crate::guids::GUID_DISPLAY_ATTR;
use crate::Lock;

pub fn attr() -> TF_DISPLAYATTRIBUTE {
    TF_DISPLAYATTRIBUTE { lsStyle: TF_LS_NONE, bAttr: TF_ATTR_INPUT, ..Default::default() }
}

#[implement(ITfDisplayAttributeInfo)]
pub struct DaInfo {
    _lock: Lock,
}

impl ITfDisplayAttributeInfo_Impl for DaInfo_Impl {
    fn GetGUID(&self) -> Result<GUID> {
        Ok(GUID_DISPLAY_ATTR)
    }

    fn GetDescription(&self) -> Result<BSTR> {
        Ok(BSTR::from("EVKey composition"))
    }

    fn GetAttributeInfo(&self, pda: *mut TF_DISPLAYATTRIBUTE) -> Result<()> {
        unsafe { *pda = attr() };
        Ok(())
    }

    fn SetAttributeInfo(&self, _pda: *const TF_DISPLAYATTRIBUTE) -> Result<()> {
        Ok(())
    }

    fn Reset(&self) -> Result<()> {
        Ok(())
    }
}

#[implement(IEnumTfDisplayAttributeInfo)]
pub struct DaEnum {
    done: Cell<bool>,
    _lock: Lock,
}

impl IEnumTfDisplayAttributeInfo_Impl for DaEnum_Impl {
    fn Clone(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        Ok(ComObject::new(DaEnum { done: Cell::new(self.done.get()), _lock: Lock::new() }).to_interface())
    }

    fn Next(&self, ulcount: u32, rginfo: *mut Option<ITfDisplayAttributeInfo>, pcfetched: *mut u32) -> Result<()> {
        let mut n = 0;
        if ulcount >= 1 && !self.done.get() {
            unsafe { rginfo.write(Some(new_info())) };
            self.done.set(true);
            n = 1;
        }
        if !pcfetched.is_null() {
            unsafe { *pcfetched = n };
        }
        Ok(())
    }

    fn Reset(&self) -> Result<()> {
        self.done.set(false);
        Ok(())
    }

    fn Skip(&self, ulcount: u32) -> Result<()> {
        if ulcount >= 1 {
            self.done.set(true);
        }
        Ok(())
    }
}

pub fn new_info() -> ITfDisplayAttributeInfo {
    ComObject::new(DaInfo { _lock: Lock::new() }).to_interface()
}

pub fn new_enum() -> IEnumTfDisplayAttributeInfo {
    ComObject::new(DaEnum { done: Cell::new(false), _lock: Lock::new() }).to_interface()
}
