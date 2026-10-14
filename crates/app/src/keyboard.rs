//! The Vietnamese/English mode (owned by the keyboard hook), the toggle sound, and removal of the keyboard entry left
//! behind by the TSF text service of older versions (ADR 0006).
use windows::core::{s, w, PCWSTR};
use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONASTERISK, MB_OK};

/// Profile string of the TSF keyboard EVKey 2.x registered under English (United States).
const LEGACY_PROFILE: PCWSTR = w!("0409:{B5F8D54F-6BAD-46D4-8DBB-FFE03BF4D2F4}{91566F64-0E09-4724-93DA-6C798B765C4E}");
const ILOT_UNINSTALL: u32 = 0x1;

type InstallLayoutOrTipFn = unsafe extern "system" fn(PCWSTR, u32) -> i32;

/// Drops the old TSF keyboard from the current user's keyboard list, which would otherwise process every key a second
/// time. Reads one registry value when there is nothing to do. The per-user list lives in this key, not in
/// `IsEnabledLanguageProfile` (that one reports the machine-wide registration).
pub fn remove_legacy_keyboard() {
    let mut data = 0u32;
    let mut size = size_of::<u32>() as u32;
    let listed = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Control Panel\\International\\User Profile\\en-US"),
            LEGACY_PROFILE,
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut _),
            Some(&mut size),
        )
    }
    .is_ok()
        && data != 0;
    if !listed {
        return;
    }
    unsafe {
        // InstallLayoutOrTip is not in the windows crate; input.dll exports it (undocumented but stable since Windows 7).
        let Ok(dll) = LoadLibraryW(w!("input.dll")) else { return };
        let Some(proc) = GetProcAddress(dll, s!("InstallLayoutOrTip")) else { return };
        let install: InstallLayoutOrTipFn = std::mem::transmute(proc);
        install(LEGACY_PROFILE, ILOT_UNINSTALL);
    }
}

/// Vietnamese/English mode. It lives in the hook thread's session; these read and set the shared flag.
pub mod mode {
    use serde::Serialize;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Mode {
        Vietnamese,
        English,
    }

    pub fn get() -> Mode {
        if crate::hook::english() { Mode::English } else { Mode::Vietnamese }
    }

    pub fn set(mode: Mode) {
        crate::hook::set_english(mode == Mode::English);
    }

    /// Flips the mode and returns the new one.
    pub fn toggle() -> Mode {
        let next = match get() {
            Mode::Vietnamese => Mode::English,
            Mode::English => Mode::Vietnamese,
        };
        set(next);
        next
    }
}

/// Short cue for a user-initiated V/E toggle (hotkey, popup switch). One system sound per mode; `MessageBeep` returns at once, so it is safe in the hook callback.
pub fn play_toggle_sound(english: bool) {
    let style = if english { MB_OK } else { MB_ICONASTERISK };
    let _ = unsafe { MessageBeep(style) };
}
