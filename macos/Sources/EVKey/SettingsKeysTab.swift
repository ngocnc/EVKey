import EVKeyKit
import SwiftUI

/// "Phím tắt": Vietnamese/English toggle chord and the temporary-off key.
struct SettingsKeysTab: View {
    @ObservedObject private var state = AppState.shared

    private static let presetHotkeys = ["alt+z", "ctrl+shift", "alt+shift", "ctrl+alt+z", "cmd+shift", "cmd+alt", "cmd+ctrl"]

    private static let tempOffChoices: [ChoiceOption<TempOffKey>] = [
        ChoiceOption(value: .none, label: "Không dùng"),
        ChoiceOption(value: .ctrl, label: "⌃"),
        ChoiceOption(value: .alt, label: "⌥"),
    ]

    /// The presets plus the current value when it was written by hand in config.toml, so saving never drops it.
    private var hotkeyChoices: [ChoiceOption<String>] {
        var values = Self.presetHotkeys
        let current = state.settings.toggleHotkey
        if !values.contains(current) { values.append(current) }
        return values.map { ChoiceOption(value: $0, label: Self.hotkeyLabel($0)) }
    }

    var body: some View {
        SettingsPage(title: "Phím tắt") {
            SettingsGroup {
                SettingsMenuRow(
                    title: "Chuyển Việt / Anh", subtitle: "Tổ hợp phím bật tắt bộ gõ trong mọi ứng dụng.",
                    choices: hotkeyChoices, selection: state.binding(\.toggleHotkey))
                SettingsChoiceRow(
                    title: "Phím tạm tắt giữa từ",
                    subtitle: "Nhấn phím này một lần giữa từ để phần còn lại của từ không bị biến đổi.",
                    choices: Self.tempOffChoices, selection: state.binding(\.tempOffKey))
            }
        }
    }

    /// Symbols for a chord such as `ctrl+alt+z`; `none` reads as "Tắt".
    private static func hotkeyLabel(_ chord: String) -> String {
        if chord == "none" { return "Tắt" }
        return chord.split(separator: "+").map { part -> String in
            switch part {
            case "ctrl": "⌃"
            case "alt": "⌥"
            case "shift": "⇧"
            case "cmd", "win": "⌘"
            default: part.uppercased()
            }
        }.joined(separator: " ")
    }
}
