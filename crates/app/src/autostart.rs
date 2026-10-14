//! Autostart: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\EVKey` = `"<exe>" --hidden`.
use windows::core::w;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SAM_FLAGS, REG_SZ,
};

fn open_run_key(access: REG_SAM_FLAGS) -> Option<HKEY> {
    let mut key = HKEY::default();
    let rc = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            Some(0),
            access,
            &mut key,
        )
    };
    (rc == ERROR_SUCCESS).then_some(key)
}

pub fn is_enabled() -> bool {
    let Some(key) = open_run_key(KEY_QUERY_VALUE) else {
        return false;
    };
    let rc = unsafe { RegQueryValueExW(key, w!("EVKey"), None, None, None, None) };
    unsafe {
        let _ = RegCloseKey(key);
    }
    rc == ERROR_SUCCESS
}

pub fn set(on: bool) -> Result<(), String> {
    let key = open_run_key(KEY_SET_VALUE).ok_or_else(|| crate::i18n::tr("Không mở được khóa Run", "Could not open the Run registry key").to_string())?;
    let rc = if on {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let command: Vec<u16> = format!("\"{}\" --hidden", exe.display())
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: a u16 buffer of `len` items is exactly `len * 2` initialized bytes.
        let bytes = unsafe { std::slice::from_raw_parts(command.as_ptr().cast::<u8>(), command.len() * 2) };
        unsafe { RegSetValueExW(key, w!("EVKey"), None, REG_SZ, Some(bytes)) }
    } else {
        unsafe { RegDeleteValueW(key, w!("EVKey")) }
    };
    unsafe {
        let _ = RegCloseKey(key);
    }
    if !on && rc == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    rc.ok().map_err(|e| e.to_string())
}

/// Startup: when autostart is on, re-point the Run value at this exe (installs use versioned folders).
pub fn repoint_if_enabled() {
    if is_enabled() {
        let _ = set(true);
    }
}
