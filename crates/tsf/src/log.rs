//! Debug log: active only while `%LOCALAPPDATA%\EVKey\debug` exists.
use std::io::Write;
use std::path::Path;

use windows::Win32::System::SystemInformation::GetLocalTime;

pub fn line(msg: &str) {
    let Some(base) = std::env::var_os("LOCALAPPDATA") else { return };
    let dir = Path::new(&base).join("EVKey");
    if !dir.join("debug").exists() {
        return;
    }
    let t = unsafe { GetLocalTime() };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("tsf.log")) {
        let _ = writeln!(f, "{:02}:{:02}:{:02}.{:03} {msg}", t.wHour, t.wMinute, t.wSecond, t.wMilliseconds);
    }
}
