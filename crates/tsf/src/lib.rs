//! EVKey Windows text service (TSF TIP): COM entry points. Logic lives in the modules.
mod display;
mod edit;
mod factory;
mod guids;
mod log;
mod mode;
mod register;
mod secure;
mod service;
mod sinks;
mod toggle;

use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU32, Ordering};

use guids::CLSID_EVKEY;
use windows::core::{ComObject, Interface, GUID, HRESULT};
use windows::Win32::Foundation::{CLASS_E_CLASSNOTAVAILABLE, S_FALSE, S_OK};
use windows::Win32::System::Com::IClassFactory;

static LOCKS: AtomicU32 = AtomicU32::new(0);

/// One per live COM object; keeps `DllCanUnloadNow` honest.
pub struct Lock;

impl Lock {
    pub fn new() -> Self {
        LOCKS.fetch_add(1, Ordering::SeqCst);
        Lock
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        LOCKS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// A panic must never unwind into the host through COM: that aborts every app using the TIP.
pub fn guard<T>(default: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_| {
        log::line("panic caught at FFI boundary");
        default
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: *const GUID, obj: *mut *mut c_void) -> HRESULT {
    guard(CLASS_E_CLASSNOTAVAILABLE, || unsafe {
        if *rclsid != CLSID_EVKEY {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let f: IClassFactory = ComObject::new(factory::Factory::new()).to_interface();
        f.query(riid, obj)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if LOCKS.load(Ordering::SeqCst) == 0 { S_OK } else { S_FALSE }
}

#[unsafe(no_mangle)]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    guard(windows::Win32::Foundation::E_FAIL, || match register::register() {
        Ok(()) => {
            log::line("registered");
            S_OK
        }
        Err(e) => {
            log::line(&format!("register failed: {e}"));
            e.code()
        }
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    guard(windows::Win32::Foundation::E_FAIL, || match register::unregister() {
        Ok(()) => S_OK,
        Err(e) => e.code(),
    })
}
