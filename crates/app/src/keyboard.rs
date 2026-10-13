//! V/E state = whether the EVKey TSF profile is in the current user's keyboard list.
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
