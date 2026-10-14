import EVKeyKit
import SwiftUI

/// "Gõ tiếng Việt": input method, tone placement, code table and free tone marks.
struct SettingsTypingTab: View {
    @ObservedObject private var state = AppState.shared

    private static let inputChoices: [ChoiceOption<InputKind>] = [
        ChoiceOption(value: .telex, label: "Telex"),
        ChoiceOption(value: .vni, label: "VNI"),
        ChoiceOption(value: .simpleTelex1, label: "Simple Telex 1"),
        ChoiceOption(value: .simpleTelex2, label: "Simple Telex 2"),
    ]

    private static var toneChoices: [ChoiceOption<TonePlacement>] {
        [
            ChoiceOption(value: .new, label: UIStrings.tr("typing.tone.modern")),
            ChoiceOption(value: .old, label: UIStrings.tr("typing.tone.classic")),
        ]
    }

    private static var tableChoices: [ChoiceOption<CodeTable>] {
        [
            ChoiceOption(value: .unicode, label: "Unicode"),
            ChoiceOption(value: .tcvn3, label: "TCVN3"),
            ChoiceOption(value: .vniWindows, label: "VNI Windows"),
            ChoiceOption(value: .unicodeCompound, label: UIStrings.tr("codeTable.unicodeCompound")),
        ]
    }

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.typing")) {
            SettingsGroup {
                HStack(spacing: 12) {
                    Image(systemName: state.accessibilityTrusted ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                        .foregroundStyle(state.accessibilityTrusted ? Color.green : Color.orange)
                        .font(.system(size: 18))
                    SettingsRowLabel(
                        title: state.accessibilityTrusted
                            ? UIStrings.tr("typing.access.granted") : UIStrings.tr("typing.access.missing"),
                        subtitle: state.accessibilityTrusted
                            ? UIStrings.tr("typing.access.grantedDetail") : UIStrings.tr("typing.access.missingDetail"))
                    Spacer(minLength: 12)
                    if !state.accessibilityTrusted {
                        Button(UIStrings.tr("common.openSettings"), action: state.openAccessibilitySettings)
                    }
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .settingsBlock()
            }
            SettingsGroup {
                SettingsChoiceRow(
                    title: UIStrings.tr("typing.inputMethod"), subtitle: UIStrings.tr("typing.inputMethod.detail"),
                    choices: Self.inputChoices, selection: state.binding(\.inputType))
                SettingsChoiceRow(
                    title: UIStrings.tr("typing.tonePlacement"), subtitle: UIStrings.tr("typing.tonePlacement.detail"),
                    choices: Self.toneChoices, selection: state.binding(\.tonePlacement))
                SettingsChoiceRow(
                    title: UIStrings.tr("typing.codeTable"), subtitle: UIStrings.tr("typing.codeTable.detail"),
                    choices: Self.tableChoices, selection: state.binding(\.codeTable))
                SettingsToggleRow(
                    title: UIStrings.tr("typing.freeTone"), subtitle: UIStrings.tr("typing.freeTone.detail"),
                    isOn: state.binding(\.freeMark))
            }
        }
    }
}
