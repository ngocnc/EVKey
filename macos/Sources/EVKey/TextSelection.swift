import AppKit
import EVKeyKit

/// Moves text in and out of the app the user is in, through the pasteboard and Cmd+C / Cmd+V. The translate popup and the
/// action panel never take focus, so the keys reach that app.
@MainActor
enum TextSelection {
    private static let copyKey: CGKeyCode = 0x08  // kVK_ANSI_C
    private static let pasteKey: CGKeyCode = 0x09  // kVK_ANSI_V
    private static let modifiers: CGEventFlags = [.maskCommand, .maskAlternate, .maskControl, .maskShift]

    /// Copies the current selection with Cmd+C and restores the pasteboard. Returns "" when nothing new was copied.
    // ponytail: only text is restored; a non-text pasteboard (image, files) is lost. Per-type backup if that matters.
    static func capture() async -> String {
        await waitForModifiersReleased()
        let pasteboard = NSPasteboard.general
        let saved = pasteboard.string(forType: .string)
        let before = pasteboard.changeCount
        KeyInjector.commandKey(copyKey)
        let deadline = Date().addingTimeInterval(0.6)
        while pasteboard.changeCount == before && Date() < deadline {
            try? await Task.sleep(nanoseconds: 15_000_000)
        }
        guard pasteboard.changeCount != before else { return "" }
        let copied = pasteboard.string(forType: .string) ?? ""
        write(saved)
        return copied.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    /// The step chosen in settings once a result is on screen: nothing, copy it, or paste it over the selection.
    static func apply(_ action: TranslateAction, result: String) async {
        switch action {
        case .show:
            return
        case .copy:
            write(result)
        case .replace:
            await waitForModifiersReleased()
            let saved = NSPasteboard.general.string(forType: .string)
            write(result)
            KeyInjector.commandKey(pasteKey)
            try? await Task.sleep(nanoseconds: 300_000_000)
            write(saved)
        }
    }

    /// Replaces the pasteboard string; nil clears it.
    static func write(_ string: String?) {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        if let string { pasteboard.setString(string, forType: .string) }
    }

    /// The hotkey's own modifiers are still down; Cmd+C sent while they are held would be a different shortcut.
    private static func waitForModifiersReleased() async {
        let deadline = Date().addingTimeInterval(1)
        while !CGEventSource.flagsState(.combinedSessionState).intersection(modifiers).isEmpty && Date() < deadline {
            try? await Task.sleep(nanoseconds: 20_000_000)
        }
    }

    /// Where a popup of `size` goes: below-right of the pointer, flipped above when it would fall off the bottom,
    /// kept inside the visible area of the screen the pointer is on.
    static func popupFrame(anchor: NSPoint, size: NSSize) -> NSRect {
        let screen = NSScreen.screens.first { $0.frame.contains(anchor) } ?? NSScreen.main
        let area = screen?.visibleFrame ?? .zero
        let margin: CGFloat = 6
        let x = min(max(anchor.x + 12, area.minX + margin), area.maxX - size.width - margin)
        var y = anchor.y - 16 - size.height
        if y < area.minY + margin { y = anchor.y + 16 }
        y = min(max(y, area.minY + margin), area.maxY - size.height - margin)
        return NSRect(origin: NSPoint(x: x, y: y), size: size)
    }
}
