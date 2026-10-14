//! The user's Gemini API key, kept in Windows Credential Manager (a per-user generic credential).
//! The key is never written to config files or logs, and never sent to the web UI.
use windows::core::{PCWSTR, PWSTR, HRESULT};
use windows::Win32::Foundation::ERROR_NOT_FOUND;
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
};

use crate::i18n::tr;

const GEMINI_TARGET: &str = "EVKey/gemini";

pub fn set_gemini_key(key: &str) -> Result<(), String> {
    store(GEMINI_TARGET, key)
}

pub fn gemini_key() -> Option<String> {
    load(GEMINI_TARGET)
}

pub fn delete_gemini_key() -> Result<(), String> {
    remove(GEMINI_TARGET)
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn store(target: &str, key: &str) -> Result<(), String> {
    let mut name = wide(target);
    let blob = key.as_bytes();
    let cred = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(name.as_mut_ptr()),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_ptr().cast_mut(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        ..Default::default()
    };
    // Safety: `name` and `blob` outlive the call; CredWriteW copies the credential.
    unsafe { CredWriteW(&cred, 0) }.map_err(|e| format!("{}: {e}", tr("Không lưu được API key vào Credential Manager", "Could not save the API key to Credential Manager")))
}

fn load(target: &str) -> Option<String> {
    let name = wide(target);
    let mut cred: *mut CREDENTIALW = std::ptr::null_mut();
    // Safety: on success CredReadW allocates `cred`, which is freed with CredFree below.
    unsafe { CredReadW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None, &mut cred) }.ok()?;
    // Safety: CredReadW succeeded, so `cred` points to a valid CREDENTIALW until CredFree.
    let key = unsafe {
        let c = &*cred;
        let blob: &[u8] = if c.CredentialBlobSize == 0 { &[] } else { std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize) };
        String::from_utf8(blob.to_vec()).ok()
    };
    unsafe { CredFree(cred.cast_const().cast()) };
    key.filter(|k| !k.is_empty())
}

/// Deleting a missing credential succeeds, so callers can clear the key without checking it first.
fn remove(target: &str) -> Result<(), String> {
    let name = wide(target);
    // Safety: `name` outlives the call.
    match unsafe { CredDeleteW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None) } {
        Ok(()) => Ok(()),
        Err(e) if e.code() == HRESULT::from_win32(ERROR_NOT_FOUND.0) => Ok(()),
        Err(e) => Err(format!("{}: {e}", tr("Không xóa được API key khỏi Credential Manager", "Could not remove the API key from Credential Manager"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A separate target so the test never touches the user's real key.
    const TEST_TARGET: &str = "EVKey/gemini-test";

    #[test]
    fn round_trip_and_delete() {
        let _ = remove(TEST_TARGET);
        assert_eq!(load(TEST_TARGET), None);
        store(TEST_TARGET, "AIza-test-key").unwrap();
        assert_eq!(load(TEST_TARGET).as_deref(), Some("AIza-test-key"));
        remove(TEST_TARGET).unwrap();
        assert_eq!(load(TEST_TARGET), None);
        remove(TEST_TARGET).unwrap();
    }
}
