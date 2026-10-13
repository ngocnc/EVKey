//! Port of the internal engine masks in `DataType.h`.

pub const MAX_BUFF: usize = 32;

pub const CAPS_MASK: u32 = 0x10000;
pub const TONE_MASK: u32 = 0x20000;
pub const TONEW_MASK: u32 = 0x40000;

pub const MARK1_MASK: u32 = 0x80000;
pub const MARK2_MASK: u32 = 0x100000;
pub const MARK3_MASK: u32 = 0x200000;
pub const MARK4_MASK: u32 = 0x400000;
pub const MARK5_MASK: u32 = 0x800000;
pub const MARK_MASK: u32 = 0xF80000;

pub const CHAR_MASK: u32 = 0xFFFF;
pub const STANDALONE_MASK: u32 = 0x1000000;
pub const CHAR_CODE_MASK: u32 = 0x2000000;
pub const PURE_CHARACTER_MASK: u32 = 0x80000000;

pub const END_CONSONANT_MASK: u32 = 0x4000;
pub const CONSONANT_ALLOW_MASK: u32 = 0x8000;

/// u16 variants: the tables store these flags OR-ed into u16 entries.
pub const END_CONSONANT_MASK16: u16 = 0x4000;
pub const CONSONANT_ALLOW_MASK16: u16 = 0x8000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputType {
    Telex = 0,
    Vni = 1,
    SimpleTelex1 = 2,
    SimpleTelex2 = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HookCode {
    DoNothing = 0,
    WillProcess = 1,
    BreakWord = 2,
    Restore = 3,
    ReplaceMacro = 4,
    RestoreAndStartNewSession = 5,
}
