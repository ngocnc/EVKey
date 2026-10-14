import AppKit
import ApplicationServices

/// Accessibility trust: the keyboard tap cannot see or change keystrokes of other apps without it.
enum Accessibility {
    static var isTrusted: Bool {
        AXIsProcessTrusted()
    }

    /// Shows the system prompt, which asks the user to enable EVKey in Privacy > Accessibility.
    @discardableResult
    static func prompt() -> Bool {
        let options = ["AXTrustedCheckOptionPrompt": true] as CFDictionary
        return AXIsProcessTrustedWithOptions(options)
    }

    static func openSettings() {
        guard let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") else {
            return
        }
        NSWorkspace.shared.open(url)
    }
}
