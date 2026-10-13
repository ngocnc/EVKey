//! COM and TSF registration (run by regsvr32 as admin, writes HKLM).
use windows::core::{Interface, IUnknown, Result, GUID, HSTRING, PCWSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_WRITE,
    REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::TextServices::{
    ITfCategoryMgr, ITfInputProcessorProfileMgr, CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles,
    GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER, GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT, GUID_TFCAT_TIPCAP_UIELEMENTENABLED, GUID_TFCAT_TIP_KEYBOARD,
};

use crate::guids::{CLSID_EVKEY, GUID_PROFILE, LANGID};

const CLSID_ROOT: &str = r"SOFTWARE\Classes\CLSID";

const CATEGORIES: [GUID; 5] = [
    GUID_TFCAT_TIP_KEYBOARD,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
    GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
];

/// `SOFTWARE\Classes\CLSID\{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` for the TIP.
fn clsid_key() -> String {
    let g = CLSID_EVKEY;
    let tail: String = g.data4.iter().map(|b| format!("{b:02X}")).collect();
    format!(r"{CLSID_ROOT}\{{{:08X}-{:04X}-{:04X}-{}-{}}}", g.data1, g.data2, g.data3, &tail[..4], &tail[4..])
}

fn dll_path() -> Result<String> {
    let mut module = HMODULE::default();
    let addr = dll_path as *const () as *const u16;
    unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(addr),
            &mut module,
        )?;
    }
    let mut buf = [0u16; 1024];
    let n = unsafe { GetModuleFileNameW(Some(module), &mut buf) };
    Ok(String::from_utf16_lossy(&buf[..n as usize]))
}

fn set_str(subkey: &str, name: &str, data: &str) -> Result<()> {
    let wide: Vec<u8> = data.encode_utf16().chain([0]).flat_map(u16::to_le_bytes).collect();
    unsafe {
        let mut key = HKEY::default();
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            &HSTRING::from(subkey),
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        )
        .ok()?;
        let r = RegSetValueExW(key, &HSTRING::from(name), None, REG_SZ, Some(&wide)).ok();
        let _ = RegCloseKey(key);
        r
    }
}

fn create<T: Interface>(clsid: &GUID) -> Result<T> {
    unsafe { CoCreateInstance(clsid, None::<&IUnknown>, CLSCTX_INPROC_SERVER) }
}

pub fn register() -> Result<()> {
    let key = clsid_key();
    let inproc = format!(r"{key}\InprocServer32");
    set_str(&inproc, "", &dll_path()?)?;
    set_str(&inproc, "ThreadingModel", "Apartment")?;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let profiles: ITfInputProcessorProfileMgr = create(&CLSID_TF_InputProcessorProfiles)?;
    let desc: Vec<u16> = "EVKey".encode_utf16().collect();
    unsafe {
        profiles.RegisterProfile(&CLSID_EVKEY, LANGID, &GUID_PROFILE, &desc, &[], 0, HKL::default(), 0, true, 0)?;
    }
    let cats: ITfCategoryMgr = create(&CLSID_TF_CategoryMgr)?;
    for cat in &CATEGORIES {
        unsafe { cats.RegisterCategory(&CLSID_EVKEY, cat, &CLSID_EVKEY)? };
    }
    Ok(())
}

/// Best effort: each step is attempted even if an earlier one failed.
pub fn unregister() -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    if let Ok(profiles) = create::<ITfInputProcessorProfileMgr>(&CLSID_TF_InputProcessorProfiles) {
        let _ = unsafe { profiles.UnregisterProfile(&CLSID_EVKEY, LANGID, &GUID_PROFILE, 0) };
    }
    if let Ok(cats) = create::<ITfCategoryMgr>(&CLSID_TF_CategoryMgr) {
        for cat in &CATEGORIES {
            let _ = unsafe { cats.UnregisterCategory(&CLSID_EVKEY, cat, &CLSID_EVKEY) };
        }
    }
    unsafe {
        let _ = RegDeleteTreeW(HKEY_LOCAL_MACHINE, &HSTRING::from(clsid_key()));
    }
    Ok(())
}
