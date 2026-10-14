import EVKeyKit
import SwiftUI

/// "Chính tả & gõ nhanh": spelling check, quick consonants and capitalisation.
struct SettingsSmartTab: View {
    @ObservedObject private var state = AppState.shared

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.smart")) {
            SettingsGroup {
                SettingsCard {
                    SettingsToggleLine(
                        title: UIStrings.tr("smart.spellCheck"), subtitle: UIStrings.tr("smart.spellCheck.detail"),
                        isOn: state.binding(\.checkSpelling))
                    SettingsDivider()
                    SettingsToggleLine(
                        title: UIStrings.tr("smart.restore"), subtitle: UIStrings.tr("smart.restore.detail"),
                        isOn: state.binding(\.restoreIfWrongSpelling))
                }
            }

            SettingsHeading(UIStrings.tr("smart.quickTyping"))
            SettingsGroup {
                SettingsToggleRow(
                    title: UIStrings.tr("smart.quickTelex"), subtitle: "cc→ch, gg→gi, kk→kh, nn→ng, qq→qu, pp→ph, tt→th",
                    isOn: state.binding(\.quickTelex))
                SettingsToggleRow(
                    title: UIStrings.tr("smart.quickStart"), subtitle: "f→ph, j→gi, w→qu",
                    isOn: state.binding(\.quickStartConsonant))
                SettingsToggleRow(
                    title: UIStrings.tr("smart.quickEnd"), subtitle: "g→ng, h→nh, k→ch",
                    isOn: state.binding(\.quickEndConsonant))
                SettingsCard {
                    SettingsToggleLine(
                        title: UIStrings.tr("smart.customConsonants"),
                        isOn: state.binding(\.allowConsonantZfwj))
                    if state.settings.allowConsonantZfwj {
                        SettingsDivider()
                        SettingsConsonantChips()
                    }
                }
            }

            SettingsGroup {
                SettingsToggleRow(
                    title: UIStrings.tr("smart.perApp"),
                    subtitle: UIStrings.tr("smart.perApp.detail"),
                    isOn: state.binding(\.rememberAppMode))
                SettingsToggleRow(
                    title: UIStrings.tr("smart.escUndo"),
                    subtitle: UIStrings.tr("smart.escUndo.detail"),
                    isOn: state.binding(\.undoWord))
            }

            SettingsHeading(UIStrings.tr("smart.capitalize"))
            SettingsGroup {
                SettingsCard {
                    SettingsToggleLine(
                        title: UIStrings.tr("smart.autoCapitalize"), subtitle: UIStrings.tr("smart.caps_first.desc"),
                        isOn: state.binding(\.upperCaseFirstChar))
                    if state.settings.upperCaseFirstChar {
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.capsPunct"), subtitle: UIStrings.tr("smart.capsPunct.detail"),
                            isOn: state.binding(\.upperCaseAfterPunct))
                            .padding(.leading, 14)
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.capsEnter"), subtitle: UIStrings.tr("smart.capsEnter.detail"),
                            isOn: state.binding(\.upperCaseAfterEnter))
                            .padding(.leading, 14)
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.capsFirst"), subtitle: UIStrings.tr("smart.capsFirst.detail"),
                            isOn: state.binding(\.upperCaseFirstLetter))
                            .padding(.leading, 14)
                    }
                }
                SettingsToggleRow(
                    title: UIStrings.tr("smart.macroCase"), subtitle: UIStrings.tr("smart.macroCase.detail"),
                    isOn: state.binding(\.autoCapsMacro))
            }
        }
    }
}
