//! Key codes from `platforms/win32.h` (Windows VK values, plus the
//! engine's own extras), typed `u16` like the C++ engine.

// Windows VK_* values used by win32.h.
pub const VK_BACK: u16 = 0x08;
pub const VK_TAB: u16 = 0x09;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_SPACE: u16 = 0x20;
pub const VK_LEFT: u16 = 0x25;
pub const VK_UP: u16 = 0x26;
pub const VK_RIGHT: u16 = 0x27;
pub const VK_DOWN: u16 = 0x28;
pub const VK_OEM_PERIOD: u16 = 0xBE;
pub const VK_OEM_COMMA: u16 = 0xBC;

// Extra VK_* values referenced by `_breakCode` under `#if _WIN32` in Engine.cpp.
pub const VK_INSERT: u16 = 0x2D;
pub const VK_HOME: u16 = 0x24;
pub const VK_END: u16 = 0x23;
pub const VK_DELETE: u16 = 0x2E;
pub const VK_PRIOR: u16 = 0x21;
pub const VK_NEXT: u16 = 0x22;
pub const VK_SNAPSHOT: u16 = 0x2C;
pub const VK_PRINT: u16 = 0x2A;
pub const VK_SELECT: u16 = 0x29;
pub const VK_HELP: u16 = 0x2F;
pub const VK_EXECUTE: u16 = 0x2B;
pub const VK_NUMLOCK: u16 = 0x90;
pub const VK_SCROLL: u16 = 0x91;

pub const KEY_ESC: u16 = VK_ESCAPE;
pub const KEY_DELETE: u16 = VK_BACK;
pub const KEY_TAB: u16 = VK_TAB;
pub const KEY_ENTER: u16 = VK_RETURN;
pub const KEY_RETURN: u16 = VK_RETURN;
pub const KEY_SPACE: u16 = VK_SPACE;
pub const KEY_LEFT: u16 = VK_LEFT;
pub const KEY_RIGHT: u16 = VK_RIGHT;
pub const KEY_DOWN: u16 = VK_DOWN;
pub const KEY_UP: u16 = VK_UP;

pub const KEY_EMPTY: u16 = 256;

pub const KEY_A: u16 = 0x41;
pub const KEY_B: u16 = 0x42;
pub const KEY_C: u16 = 0x43;
pub const KEY_D: u16 = 0x44;
pub const KEY_E: u16 = 0x45;
pub const KEY_F: u16 = 0x46;
pub const KEY_G: u16 = 0x47;
pub const KEY_H: u16 = 0x48;
pub const KEY_I: u16 = 0x49;
pub const KEY_J: u16 = 0x4A;
pub const KEY_K: u16 = 0x4B;
pub const KEY_L: u16 = 0x4C;
pub const KEY_M: u16 = 0x4D;
pub const KEY_N: u16 = 0x4E;
pub const KEY_O: u16 = 0x4F;
pub const KEY_P: u16 = 0x50;
pub const KEY_Q: u16 = 0x51;
pub const KEY_R: u16 = 0x52;
pub const KEY_S: u16 = 0x53;
pub const KEY_T: u16 = 0x54;
pub const KEY_U: u16 = 0x55;
pub const KEY_V: u16 = 0x56;
pub const KEY_W: u16 = 0x57;
pub const KEY_X: u16 = 0x58;
pub const KEY_Y: u16 = 0x59;
pub const KEY_Z: u16 = 0x5A;

pub const KEY_1: u16 = 0x31;
pub const KEY_2: u16 = 0x32;
pub const KEY_3: u16 = 0x33;
pub const KEY_4: u16 = 0x34;
pub const KEY_5: u16 = 0x35;
pub const KEY_6: u16 = 0x36;
pub const KEY_7: u16 = 0x37;
pub const KEY_8: u16 = 0x38;
pub const KEY_9: u16 = 0x39;
pub const KEY_0: u16 = 0x30;

pub const KEY_LEFT_BRACKET: u16 = 0xDB;
pub const KEY_RIGHT_BRACKET: u16 = 0xDD;

pub const KEY_LEFT_SHIFT: u16 = 160;
pub const KEY_RIGHT_SHIFT: u16 = 161;
pub const KEY_DOT: u16 = VK_OEM_PERIOD;

pub const KEY_BACKQUOTE: u16 = 220;
pub const KEY_MINUS: u16 = 189;
pub const KEY_EQUALS: u16 = 187;
pub const KEY_BACK_SLASH: u16 = 222;
pub const KEY_SEMICOLON: u16 = 186;
pub const KEY_QUOTE: u16 = 192;
pub const KEY_COMMA: u16 = VK_OEM_COMMA;
pub const KEY_SLASH: u16 = 191;
