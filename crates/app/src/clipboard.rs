//! Plain-text clipboard access (CF_UNICODETEXT), used by the translate feature to read the selection and paste a result.

use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

const CF_UNICODETEXT: u32 = 13;

/// An open clipboard; closes on drop. Another app may hold it briefly, so opening retries.
struct Open;

impl Open {
    fn new() -> Option<Open> {
        for _ in 0..10 {
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Some(Open);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

/// Changes whenever any app writes the clipboard.
pub fn sequence() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

pub fn get_text() -> Option<String> {
    let _open = Open::new()?;
    unsafe {
        let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
        let global = HGLOBAL(handle.0);
        let ptr = GlobalLock(global) as *const u16;
        if ptr.is_null() {
            return None;
        }
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
        let _ = GlobalUnlock(global);
        Some(text)
    }
}

pub fn set_text(text: &str) -> bool {
    let units: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let Some(_open) = Open::new() else { return false };
    unsafe {
        let _ = EmptyClipboard();
        let Ok(global) = GlobalAlloc(GMEM_MOVEABLE, units.len() * 2) else { return false };
        let ptr = GlobalLock(global) as *mut u16;
        if ptr.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(units.as_ptr(), ptr, units.len());
        let _ = GlobalUnlock(global);
        SetClipboardData(CF_UNICODETEXT, Some(HANDLE(global.0))).is_ok()
    }
}
