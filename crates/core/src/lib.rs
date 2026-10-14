//! Platform-independent EVKey core: the typing session shared by the Windows hook and the macOS event tap,
//! plus settings I/O. Hosts map their native key codes to Windows virtual-key codes and inject what [`Reaction`]
//! describes. [`ffi`] is the C ABI used by non-Rust hosts.
pub mod ffi;
pub mod session;
pub mod typing;

pub use session::{is_modifier_vk, KeyInput, Mods, Output, Reaction, Resend, Session};
