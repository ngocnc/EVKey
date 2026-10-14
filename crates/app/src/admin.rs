//! Chạy với quyền quản trị: a Task Scheduler task starts EVKey elevated at sign-in, so the hook also types into
//! windows that run as administrator. The task replaces the `Run` autostart value while it exists. A non-elevated
//! launch (Start menu, self-update relaunch) hands over to the task, which starts EVKey elevated without a UAC prompt.

use std::os::windows::process::CommandExt;
use std::process::Command;
use std::time::Duration;

use windows::core::{w, HSTRING};
use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

use crate::i18n::tr;
use crate::{autostart, ipc};

const TASK: &str = "EVKey";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn is_elevated() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}

fn schtasks(args: &[&str]) -> Result<(), String> {
    let out = Command::new("schtasks").args(args).creation_flags(CREATE_NO_WINDOW).output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

pub fn task_exists() -> bool {
    schtasks(&["/Query", "/TN", TASK]).is_ok()
}

/// Startup of a non-elevated instance: true when it should start the elevated task and exit instead.
pub fn should_hand_over() -> bool {
    !is_elevated() && task_exists()
}

/// Starts the task (EVKey elevated). The caller must have released the single-instance mutex first.
pub fn run_task() -> Result<(), String> {
    schtasks(&["/Run", "/TN", TASK])
}

/// Settings switch: runs `evkey --admin on|off` elevated, which shows the UAC prompt. Turning it on ends this
/// instance: the elevated process stops it and takes over.
pub fn request(on: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let args = HSTRING::from(if on { "--admin on" } else { "--admin off" });
    let rc = unsafe { ShellExecuteW(None, w!("runas"), &HSTRING::from(exe.as_os_str()), &args, None, SW_HIDE) };
    // ShellExecute returns a value above 32 on success; a cancelled UAC prompt gives SE_ERR_ACCESSDENIED (5).
    if rc.0 as usize > 32 {
        Ok(())
    } else {
        Err(tr("Chưa được cấp quyền quản trị", "Administrator permission was not granted").into())
    }
}

/// `evkey --admin on|off`, run elevated. `on`: creates the task, stops the running EVKey and waits until it is gone;
/// the caller then runs the app in this (elevated) process. `off`: deletes the task and restores the `Run` value.
pub fn apply(on: bool) -> Result<(), String> {
    if !is_elevated() {
        return Err(tr("Cần chạy với quyền quản trị", "Must run as administrator").into());
    }
    if !on {
        let _ = schtasks(&["/Delete", "/TN", TASK, "/F"]);
        return autostart::set(true);
    }
    create_task()?;
    // The Run value would start a second, non-elevated copy at sign-in.
    autostart::set(false)?;
    let _ = ipc::send_to_running_instance("quit");
    for _ in 0..50 {
        if ipc::acquire_single_instance() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(tr("EVKey đang chạy không chịu thoát", "The running EVKey did not quit").into())
}

/// `evkey --admin repoint`, run by the (elevated) installer: installs go to versioned folders, so an existing task is
/// pointed at the new exe.
pub fn repoint() -> Result<(), String> {
    if task_exists() { create_task() } else { Ok(()) }
}

/// Registers the task from XML: plain `schtasks /Create` would add a 72-hour run limit, battery conditions and a
/// below-normal priority, all wrong for an input method.
fn create_task() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let user = format!("{}\\{}", std::env::var("USERDOMAIN").unwrap_or_default(), std::env::var("USERNAME").unwrap_or_default());
    let xml = task_xml(&exe.display().to_string(), &user);
    let path = std::env::temp_dir().join("evkey-admin-task.xml");
    // schtasks reads the XML as UTF-16 LE with a BOM.
    let bytes: Vec<u8> = std::iter::once(0xFEFF).chain(xml.encode_utf16()).flat_map(u16::to_le_bytes).collect();
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let result = schtasks(&["/Create", "/TN", TASK, "/XML", &path.display().to_string(), "/F"]);
    let _ = std::fs::remove_file(&path);
    result
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn task_xml(exe: &str, user: &str) -> String {
    let (exe, user) = (xml_escape(exe), xml_escape(user));
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{user}</UserId></LogonTrigger></Triggers>
  <Principals><Principal id="Author"><UserId>{user}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>HighestAvailable</RunLevel></Principal></Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
  </Settings>
  <Actions Context="Author"><Exec><Command>{exe}</Command><Arguments>--hidden</Arguments></Exec></Actions>
</Task>"#
    )
}
