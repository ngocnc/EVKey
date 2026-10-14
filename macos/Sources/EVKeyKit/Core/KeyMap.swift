import CoreGraphics

/// Translates macOS keycodes and modifier flags into the Windows virtual-key codes the core ABI expects.
enum KeyMap {
    /// A modifier key: its Windows VK (side-specific) and the device-dependent flag bit that is set while it is held.
    struct Modifier {
        let vk: UInt16
        let deviceMask: UInt64
    }

    /// Keys the engine models. Keycodes not listed (Fn, Caps Lock, ...) end the current word.
    static let vkByKeycode: [UInt16: UInt16] = [
        // Letters: Windows VK is the ASCII uppercase code.
        0: 0x41, 1: 0x53, 2: 0x44, 3: 0x46, 4: 0x48, 5: 0x47, 6: 0x5A, 7: 0x58, 8: 0x43, 9: 0x56,
        11: 0x42, 12: 0x51, 13: 0x57, 14: 0x45, 15: 0x52, 16: 0x59, 17: 0x54, 31: 0x4F, 32: 0x55,
        34: 0x49, 35: 0x50, 37: 0x4C, 38: 0x4A, 40: 0x4B, 45: 0x4E, 46: 0x4D,
        // Digits.
        18: 0x31, 19: 0x32, 20: 0x33, 21: 0x34, 22: 0x36, 23: 0x35, 25: 0x39, 26: 0x37, 28: 0x38, 29: 0x30,
        // Punctuation (OEM codes).
        24: 0xBB, 27: 0xBD, 30: 0xDD, 33: 0xDB, 39: 0xDE, 41: 0xBA, 42: 0xDC, 43: 0xBC, 44: 0xBF,
        47: 0xBE, 50: 0xC0,
        // Editing and whitespace.
        36: 0x0D, 48: 0x09, 49: 0x20, 51: 0x08, 53: 0x1B, 117: 0x2E,
        // Navigation.
        115: 0x24, 116: 0x21, 119: 0x23, 121: 0x22, 123: 0x25, 124: 0x27, 125: 0x28, 126: 0x26,
        // Function keys F1-F12 (VK_F1..VK_F12), bound by the translate hotkey.
        122: 0x70, 120: 0x71, 99: 0x72, 118: 0x73, 96: 0x74, 97: 0x75, 98: 0x76, 100: 0x77, 101: 0x78, 109: 0x79, 103: 0x7A, 111: 0x7B,
    ]

    /// Windows VK -> macOS keycode, for resending a key that has no character.
    static let keycodeByVK: [UInt16: UInt16] = Dictionary(
        vkByKeycode.map { ($0.value, $0.key) }, uniquingKeysWith: { first, _ in first })

    /// Modifier keycodes (left and right). Their state arrives as `flagsChanged`.
    static let modifiers: [UInt16: Modifier] = [
        59: Modifier(vk: 0xA2, deviceMask: 0x0000_0001),  // left Control
        62: Modifier(vk: 0xA3, deviceMask: 0x0000_2000),  // right Control
        58: Modifier(vk: 0xA4, deviceMask: 0x0000_0020),  // left Option -> Alt
        61: Modifier(vk: 0xA5, deviceMask: 0x0000_0040),  // right Option -> Alt
        56: Modifier(vk: 0xA0, deviceMask: 0x0000_0002),  // left Shift
        60: Modifier(vk: 0xA1, deviceMask: 0x0000_0004),  // right Shift
        55: Modifier(vk: 0x5B, deviceMask: 0x0000_0008),  // left Command -> Win
        54: Modifier(vk: 0x5C, deviceMask: 0x0000_0010),  // right Command -> Win
    ]

    static func vk(forKeycode keycode: UInt16) -> UInt16? {
        vkByKeycode[keycode]
    }

    static func keycode(forVK vk: UInt16) -> UInt16? {
        keycodeByVK[vk]
    }

    static func modifier(forKeycode keycode: UInt16) -> Modifier? {
        modifiers[keycode]
    }

    /// Core modifier bits: 1 ctrl, 2 alt, 4 shift, 8 win (Command).
    static func modBits(_ flags: CGEventFlags) -> UInt8 {
        var bits: UInt8 = 0
        if flags.contains(.maskControl) { bits |= 1 }
        if flags.contains(.maskAlternate) { bits |= 2 }
        if flags.contains(.maskShift) { bits |= 4 }
        if flags.contains(.maskCommand) { bits |= 8 }
        return bits
    }
}
