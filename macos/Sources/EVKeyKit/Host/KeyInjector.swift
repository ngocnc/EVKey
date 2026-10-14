import CoreGraphics

/// Types a `TextEdit` with CGEvents. Every posted event carries `marker`, so the tap ignores it.
struct KeyInjector {
    /// "EVKY" in `eventSourceUserData`, marking events EVKey posted.
    static let marker: Int64 = 0x4556_4B59

    private static let backspace: CGKeyCode = 0x33
    /// EMPTY_CHAR of the core session: typed and deleted first so browser autocomplete cannot swallow the edit.
    private static let emptyChar: UInt16 = 0x202F
    private static let unicodeChunk = 20

    func apply(_ edit: TextEdit) {
        if edit.prefixEmptyChar { typeUnits([Self.emptyChar]) }
        for _ in 0..<edit.backspaces { tapKey(Self.backspace) }
        typeUnits(edit.text)
        switch edit.resend {
        case .char(let unit)?:
            typeUnits([unit])
        case .vk(let vk)?:
            if let code = KeyMap.keycode(forVK: vk) { tapKey(code) }
        case nil:
            break
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

    private func tapKey(_ code: CGKeyCode) {
        for down in [true, false] {
            if let event = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: down) {
                post(event)
            }
        }
    }

    private func post(_ event: CGEvent) {
        event.setIntegerValueField(.eventSourceUserData, value: Self.marker)
        event.post(tap: .cgSessionEventTap)
    }
}
