//! Vietnamese/English mode in one TSF global compartment, so the IME, the tray app and the popup share it.
use windows::core::{Result, GUID};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::TextServices::ITfThreadMgr;

/// Global compartment holding the mode as VT_I4.
pub const GUID_COMPARTMENT_EVKEY_MODE: GUID = GUID::from_u128(0xACEC5E5E_D1F2_48F0_9587_83E800B6B947);
pub const VIETNAMESE: i32 = 0;
pub const ENGLISH: i32 = 1;

/// Current mode. Anything unreadable (including a never-set compartment) counts as Vietnamese.
pub fn read(tm: &ITfThreadMgr) -> i32 {
    let value = unsafe { tm.GetGlobalCompartment() }
        .and_then(|mgr| unsafe { mgr.GetCompartment(&GUID_COMPARTMENT_EVKEY_MODE) })
        .and_then(|comp| unsafe { comp.GetValue() });
    value.ok().and_then(|v| i32::try_from(&v).ok()).unwrap_or(VIETNAMESE)
}

pub fn write(tm: &ITfThreadMgr, tid: u32, mode: i32) -> Result<()> {
    let mgr = unsafe { tm.GetGlobalCompartment()? };
    let comp = unsafe { mgr.GetCompartment(&GUID_COMPARTMENT_EVKEY_MODE)? };
    unsafe { comp.SetValue(tid, &VARIANT::from(mode)) }
}
