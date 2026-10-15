import EVKeyKit
import SwiftUI

/// "Phím tắt": phím chuyển Việt/Anh (ghi trực tiếp, nhận cả tổ hợp chỉ gồm phím bổ trợ) và phím tạm tắt.
struct SettingsKeysTab: View {
    @ObservedObject private var state = AppState.shared
    /// Id của hàng đang ghi phím, nếu có.
    @State private var recorder: String?

    private static var tempOffChoices: [ChoiceOption<TempOffKey>] {
        [
            ChoiceOption(value: .none, label: UIStrings.tr("keys.tempOff.none")),
            ChoiceOption(value: .ctrl, label: "⌃"),
            ChoiceOption(value: .alt, label: "⌥"),
        ]
    }

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.keys"), note: UIStrings.tr("keys.recordNote")) {
            SettingsGroup {
                SettingsHotkeyRow(
                    id: "toggle", title: UIStrings.tr("keys.toggle"), subtitle: UIStrings.tr("keys.toggle.detail"),
                    chord: state.binding(\.toggleHotkey), recorder: $recorder,
                    others: [state.settings.translateHotkey, state.settings.actionHotkey],
                    conflictNames: [
                        state.settings.translateHotkey: UIStrings.tr("translate.hotkey"),
                        state.settings.actionHotkey: UIStrings.tr("actions.hotkey"),
                    ],
                    allowsModifierOnly: true)
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

    /// Ký hiệu của một tổ hợp như `ctrl+alt+z`; `none` hiện là "Tắt".
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
