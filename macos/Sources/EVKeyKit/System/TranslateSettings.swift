import Foundation

/// The translate hotkey as the Rust core reads it (`ctrl+alt+t`): modifiers in core order, then one key.
public enum TranslateHotkey {
    public enum Capture: Equatable {
        case chord(String)
        /// A letter or digit was pressed without Control, Option, Shift or Command.
        case needsModifier
        /// Only letters, digits, Space and F1-F12 can be bound.
        case badKey
    }

    /// Builds the chord for one key press. `command` is the core's `win` modifier.
    public static func capture(keycode: UInt16, control: Bool, option: Bool, shift: Bool, command: Bool) -> Capture {
        guard let vk = KeyMap.vk(forKeycode: keycode), let key = keyName(vk) else { return .badKey }
        var parts: [String] = []
        if control { parts.append("ctrl") }
        if option { parts.append("alt") }
        if shift { parts.append("shift") }
        if command { parts.append("win") }
        if parts.isEmpty && !key.isFunction { return .needsModifier }
        return .chord((parts + [key.name]).joined(separator: "+"))
    }

    /// Symbols for display, e.g. `ctrl+alt+t` becomes ["⌃", "⌥", "T"].
    public static func labels(_ chord: String) -> [String] {
        chord.split(separator: "+").map { (part: Substring) -> String in
            switch part {
            case "ctrl": return "⌃"
            case "alt": return "⌥"
            case "shift": return "⇧"
            case "win": return "⌘"
            case "space": return "Space"
            default: return part.uppercased()
            }
        }
    }

    /// The core's key name for a Windows VK; nil for keys it cannot bind.
    static func keyName(_ vk: UInt16) -> (name: String, isFunction: Bool)? {
        switch vk {
        case 0x41...0x5A: return (String(Character(Unicode.Scalar(UInt8(vk) + 0x20))), false)
        case 0x30...0x39: return (String(Character(Unicode.Scalar(UInt8(vk)))), false)
        case 0x20: return ("space", false)
        case 0x70...0x7B: return ("f\(vk - 0x6F)", true)
        default: return nil
        }
    }
}

/// The translate popup's display time, stored as milliseconds (`translate_popup_ms`); edited as a number plus a unit.
public enum TranslateDisplayTime {
    public static let minMs = 100
    public static let maxMs = 600_000

    public enum DisplayUnit: String, CaseIterable {
        case ms, s, never
    }

    /// The unit a stored value is shown in: 0 is never, whole seconds are s, anything else is ms.
    public static func unit(forMs ms: Int) -> DisplayUnit {
        if ms == 0 { return .never }
        return ms % 1000 == 0 ? .s : .ms
    }

    /// The number shown for a stored value in `unit`; seconds round and are at least 1.
    public static func shownValue(ms: Int, unit: DisplayUnit) -> Int {
        switch unit {
        case .never: return 0
        case .ms: return ms
        case .s: return max(1, Int((Double(ms) / 1000).rounded()))
        }
    }

    /// The stored milliseconds for a typed number in `unit`, clamped to 100 ms...600 000 ms; never stores 0.
    public static func storedMs(value: Int, unit: DisplayUnit) -> Int {
        switch unit {
        case .never: return 0
        case .ms: return min(max(value, minMs), maxMs)
        case .s: return min(max(value, 1), maxMs / 1000) * 1000
        }
    }
}
