//! Two independent pieces of state:
//! - the keyboard list (`is_enabled`/`set_enabled`): whether the EVKey TSF profile is in the
//!   current user's keyboard list. Settings shows it; it does not decide V/E.
//! - the mode (`mode`): Vietnamese/English in the TSF global compartment that the IME reads.
//!   This is the real V/E state, shared system-wide.
//! Measured limit: from another process the active profile cannot be read or switched
//! reliably (GetActiveProfile reports EVKey for every thread, and the HKL is 04090409 for
//! both EVKey and en-US), so the app does not claim to show the active profile.
use windows::core::{s, w, PCWSTR};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

const ILOT_UNINSTALL: u32 = 0x1;

type InstallLayoutOrTipFn = unsafe extern "system" fn(PCWSTR, u32) -> i32;

/// True when the profile is in the current user's keyboard list. `IsEnabledLanguageProfile` is
/// not usable here: it reports the machine-wide registration, so it stays true after the user
/// removed the keyboard. `Get-WinUserLanguageList` reads this registry key instead.
pub fn is_enabled() -> Result<bool, String> {
    let mut data = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Control Panel\\International\\User Profile\\en-US"),
            w!("0409:{B5F8D54F-6BAD-46D4-8DBB-FFE03BF4D2F4}{91566F64-0E09-4724-93DA-6C798B765C4E}"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    Ok(r.is_ok() && data != 0)
}

/// Adds (`on`) or removes the EVKey profile in the current user's keyboard list. Calling it when
/// the state already matches is harmless.
pub fn set_enabled(on: bool) -> Result<(), String> {
    unsafe {
        // InstallLayoutOrTip is not in the windows crate; input.dll exports it (undocumented but stable since Windows 7).
        let dll = LoadLibraryW(w!("input.dll")).map_err(|e| e.to_string())?;
        let proc = GetProcAddress(dll, s!("InstallLayoutOrTip")).ok_or("input.dll thiếu InstallLayoutOrTip")?;
        let install: InstallLayoutOrTipFn = std::mem::transmute(proc);
        let flags = if on { 0 } else { ILOT_UNINSTALL };
        let profile = w!("0409:{B5F8D54F-6BAD-46D4-8DBB-FFE03BF4D2F4}{91566F64-0E09-4724-93DA-6C798B765C4E}");
        if install(profile, flags) == 0 {
            return Err("InstallLayoutOrTip thất bại".into());
        }
    }
    Ok(())
}

/// Vietnamese/English mode in the TSF global compartment, shared with the IME (`crates/tsf`).
/// Each thread that touches it gets one COM STA `ITfThreadMgr`, created on first use and kept.
pub mod mode {
    use std::cell::RefCell;

    use serde::Serialize;
    use windows::core::GUID;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::UI::TextServices::{CLSID_TF_ThreadMgr, ITfCompartment, ITfThreadMgr};

    /// Same GUID and values as `crates/tsf/src/mode.rs`: VT_I4, 0 = Vietnamese, 1 = English.
    const GUID_COMPARTMENT_EVKEY_MODE: GUID = GUID::from_u128(0xACEC5E5E_D1F2_48F0_9587_83E800B6B947);
    const VIETNAMESE: i32 = 0;
    const ENGLISH: i32 = 1;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Mode {
        Vietnamese,
        English,
    }

    impl Mode {
        pub fn as_str(self) -> &'static str {
            match self {
                Mode::Vietnamese => "vietnamese",
                Mode::English => "english",
            }
        }
    }

    thread_local! {
        static THREAD_MGR: RefCell<Option<ITfThreadMgr>> = const { RefCell::new(None) };
    }

    fn create_thread_mgr() -> Result<ITfThreadMgr, String> {
        // S_FALSE (already STA on this thread) is success; an MTA thread fails here.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(|e| format!("CoInitializeEx: {e}"))?;
        unsafe { CoCreateInstance(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| format!("TF_ThreadMgr: {e}"))
    }

    fn with_thread_mgr<T>(f: impl FnOnce(&ITfThreadMgr) -> Result<T, String>) -> Result<T, String> {
        THREAD_MGR.with(|slot| {
            if slot.borrow().is_none() {
                *slot.borrow_mut() = Some(create_thread_mgr()?);
            }
            let slot = slot.borrow();
            f(slot.as_ref().expect("created above"))
        })
    }

    fn compartment(tm: &ITfThreadMgr) -> Result<ITfCompartment, String> {
        unsafe {
            let mgr = tm.GetGlobalCompartment().map_err(|e| format!("GetGlobalCompartment: {e}"))?;
            mgr.GetCompartment(&GUID_COMPARTMENT_EVKEY_MODE)
                .map_err(|e| format!("GetCompartment: {e}"))
        }
    }

    /// Current mode. An unset or unreadable compartment reads as Vietnamese, as in `crates/tsf`.
    pub fn get() -> Result<Mode, String> {
        if crate::hook::active() {
            return Ok(if crate::hook::english() { Mode::English } else { Mode::Vietnamese });
        }
        with_thread_mgr(|tm| {
            let value = unsafe { compartment(tm)?.GetValue() }.ok();
            let raw = value.as_ref().and_then(|v| i32::try_from(v).ok()).unwrap_or(VIETNAMESE);
            Ok(if raw == ENGLISH { Mode::English } else { Mode::Vietnamese })
        })
    }

    pub fn set(mode: Mode) -> Result<(), String> {
        if crate::hook::active() {
            crate::hook::set_english(mode == Mode::English);
            return Ok(());
        }
        let raw = match mode {
            Mode::Vietnamese => VIETNAMESE,
            Mode::English => ENGLISH,
        };
        with_thread_mgr(|tm| {
            let comp = compartment(tm)?;
            unsafe { comp.SetValue(0, &VARIANT::from(raw)) }.map_err(|e| format!("SetValue: {e}"))
        })
    }

    /// Flips the mode and returns the new one.
    pub fn toggle() -> Result<Mode, String> {
        let next = match get()? {
            Mode::Vietnamese => Mode::English,
            Mode::English => Mode::Vietnamese,
        };
        set(next)?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::mode::{self, Mode};

    #[test]
    fn mode_round_trip() {
        // The test writes the real global compartment; put back whatever the user had.
        let before = mode::get().unwrap();
        mode::set(Mode::English).unwrap();
        assert_eq!(mode::get().unwrap(), Mode::English);
        mode::set(Mode::Vietnamese).unwrap();
        assert_eq!(mode::get().unwrap(), Mode::Vietnamese);
        mode::set(before).unwrap();
    }

    /// In-process latency of `mode::get`. Run with `-- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn mode_get_latency() {
        const N: u32 = 2000;
        mode::get().unwrap();
        let start = std::time::Instant::now();
        for _ in 0..N {
            mode::get().unwrap();
        }
        println!("mode::get: {:?} per call over {N} calls", start.elapsed() / N);
    }
}
