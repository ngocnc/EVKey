import CoreGraphics

/// Types a `TextEdit` with CGEvents. Every posted event carries `marker`, so the tap ignores it.
public struct KeyInjector {
    /// "EVKY" in `eventSourceUserData`, marking events EVKey posted.
    static let marker: Int64 = 0x4556_4B59

    private static let backspace: CGKeyCode = 0x33
    private static let leftArrow: CGKeyCode = 0x7B
    /// EMPTY_CHAR of the core session: typed and deleted first so browser autocomplete cannot swallow the edit.
    private static let emptyChar: UInt16 = 0x202F
    private static let unicodeChunk = 20

    func apply(_ edit: TextEdit) {
        if edit.prefixEmptyChar { typeUnits([Self.emptyChar]) }
        for _ in 0..<edit.backspaces { tapKey(Self.backspace) }
        let (units, back) = MacroVars.splitCursor(MacroVars.expand(edit.text))
        typeUnits(units)
        var resent = 1
        switch edit.resend {
        case .char(let unit)?:
            typeUnits([unit])
        case .vk(let vk)?:
            if let code = KeyMap.keycode(forVK: vk) { tapKey(code) }
        case nil:
            resent = 0
        }
        // `{cursor}`: step back over the text after it and the re-sent trigger key.
        if let back {
            for _ in 0..<(back + resent) { tapKey(Self.leftArrow) }
        }
    }

    private func typeUnits(_ units: [UInt16]) {
        var start = 0
        while start < units.count {
            let chunk = Array(units[start..<min(start + Self.unicodeChunk, units.count)])
            start += chunk.count
            for down in [true, false] {
                guard let event = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: down) else { continue }
                chunk.withUnsafeBufferPointer {
                    event.keyboardSetUnicodeString(stringLength: $0.count, unicodeString: $0.baseAddress)
                }
                post(event)
            }
        }
    }

    /// Presses Command plus the key at `keycode` (for Cmd+C and Cmd+V). The events carry `marker`, so the tap passes them through.
    public static func commandKey(_ keycode: CGKeyCode) {
        KeyInjector().tapKey(keycode, flags: .maskCommand)
    }

    /// Presses and releases `code`. `flags` replaces the modifier state of the events when given.
    func tapKey(_ code: CGKeyCode, flags: CGEventFlags? = nil) {
        for down in [true, false] {
            if let event = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: down) {
                if let flags { event.flags = flags }
                post(event)
            }
        }
    }

    private func post(_ event: CGEvent) {
        event.setIntegerValueField(.eventSourceUserData, value: Self.marker)
        event.post(tap: .cgSessionEventTap)
    }
}
