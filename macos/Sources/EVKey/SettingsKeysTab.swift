import EVKeyKit
import SwiftUI

/// "Phím tắt": Vietnamese/English toggle chord and the temporary-off key.
struct SettingsKeysTab: View {
    @ObservedObject private var state = AppState.shared

    private static let presetHotkeys = ["alt+z", "ctrl+shift", "alt+shift", "ctrl+alt+z", "cmd+shift", "cmd+alt", "cmd+ctrl"]

    private static var tempOffChoices: [ChoiceOption<TempOffKey>] {
        [
            ChoiceOption(value: .none, label: UIStrings.tr("keys.tempOff.none")),
            ChoiceOption(value: .ctrl, label: "⌃"),
            ChoiceOption(value: .alt, label: "⌥"),
        ]
    }

    /// The presets plus the current value when it was written by hand in config.toml, so saving never drops it.
    private var hotkeyChoices: [ChoiceOption<String>] {
        var values = Self.presetHotkeys
        let current = state.settings.toggleHotkey
        if !values.contains(current) { values.append(current) }
        return values.map { ChoiceOption(value: $0, label: Self.hotkeyLabel($0)) }
    }

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.keys")) {
            SettingsGroup {
                SettingsMenuRow(
                    title: UIStrings.tr("keys.toggle"), subtitle: UIStrings.tr("keys.toggle.detail"),
                    choices: hotkeyChoices, selection: state.binding(\.toggleHotkey))
            }
            SettingsAdvanced(pane: "keys") {
                SettingsGroup {
                    SettingsChoiceRow(
                        title: UIStrings.tr("keys.tempOff"),
                        subtitle: UIStrings.tr("keys.tempOff.detail"),
                        choices: Self.tempOffChoices, selection: state.binding(\.tempOffKey))
                }
            }
        }
    }

    /// Symbols for a chord such as `ctrl+alt+z`; `none` reads as "Tắt".
    static func hotkeyLabel(_ chord: String) -> String {
        if chord == "none" { return UIStrings.tr("keys.hotkey.none") }
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
