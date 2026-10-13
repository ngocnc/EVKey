//! Identifiers of the EVKey TIP. Generated once; changing them orphans existing registrations.
use windows::core::GUID;

/// COM class of the text service (also the TIP CLSID).
pub const CLSID_EVKEY: GUID = GUID::from_u128(0xB5F8D54F_6BAD_46D4_8DBB_FFE03BF4D2F4);
/// Input profile under the TIP.
pub const GUID_PROFILE: GUID = GUID::from_u128(0x91566F64_0E09_4724_93DA_6C798B765C4E);
/// Display attribute of the composition (underline).
pub const GUID_DISPLAY_ATTR: GUID = GUID::from_u128(0x11DB67BF_41AA_4E39_AC39_7B30D7A97734);

/// LANGID the profile is registered under (en-US): no Vietnamese entry needed in the language list.
pub const LANGID: u16 = 0x0409;
