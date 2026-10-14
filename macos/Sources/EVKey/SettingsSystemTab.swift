import AppKit
import EVKeyKit
import SwiftUI

/// "Hệ thống": UI language, launch at login, config folder and the about section.
struct SettingsSystemTab: View {
    @ObservedObject private var state = AppState.shared

    private static let languageChoices: [ChoiceOption<UILanguage>] = [
        ChoiceOption(value: .vi, label: "Tiếng Việt"),
        ChoiceOption(value: .en, label: "English"),
    ]

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.system")) {
            SettingsGroup {
                SettingsChoiceRow(
                    title: UIStrings.tr("system.uiLanguage"),
                    choices: Self.languageChoices,
                    selection: state.binding(\.uiLanguage))
                SettingsToggleRow(
                    title: UIStrings.tr("system.launchAtLogin"),
                    subtitle: UIStrings.tr("system.launchAtLogin.detail"),
                    isOn: Binding(
                        get: { state.autostart },
                        set: { value in state.perform { try state.setAutostart(value) } }))
                SettingsToggleRow(
                    title: UIStrings.tr("system.speakOnSwitch"),
                    subtitle: UIStrings.tr("system.speakOnSwitch.detail"),
                    isOn: state.binding(\.toggleSound))
            }

            SettingsHeading(UIStrings.tr("system.configFiles"))
            SettingsCard {
                VStack(alignment: .leading, spacing: 10) {
                    Text(state.configDir)
                        .font(.system(size: 12, design: .monospaced))
                        .textSelection(.enabled)
                        .foregroundStyle(.secondary)
                    HStack {
                        Button(UIStrings.tr("system.openFolder"), action: state.openConfigDir)
                        Spacer()
                        Button(UIStrings.tr("system.reset"), role: .destructive) {
                            if confirmResetConfig() { state.perform { try state.resetConfig() } }
                        }
                    }
                }
                .padding(14)
            }
        }
    }
}

/// Asks before the config is replaced by defaults. Returns true when the user confirms.
@MainActor
func confirmResetConfig() -> Bool {
    let alert = NSAlert()
    alert.messageText = UIStrings.tr("system.reset.title")
    alert.informativeText = UIStrings.tr("system.reset.message")
    alert.alertStyle = .warning
    alert.addButton(withTitle: UIStrings.tr("system.reset.confirm"))
    alert.addButton(withTitle: UIStrings.tr("common.cancel"))
    return alert.runModal() == .alertFirstButtonReturn
}
