import EVKeyKit
import SwiftUI

/// "Gõ tiếng Việt": thẻ trạng thái, cách gõ, viết hoa, và phần nâng cao (bỏ dấu, gõ nhanh).
struct SettingsTypingTab: View {
    @ObservedObject private var state = AppState.shared

    private static let inputHints: [InputKind: String] = [
        .telex: "hint.input_type.telex",
        .vni: "hint.input_type.vni",
        .simpleTelex1: "hint.input_type.simple_telex1",
        .simpleTelex2: "hint.input_type.simple_telex2",
    ]
    private static let toneHints: [TonePlacement: String] = [
        .new: "hint.tone_placement.new",
        .old: "hint.tone_placement.old",
    ]
    private static let tableHints: [CodeTable: String] = [
        .unicode: "hint.code_table.unicode",
        .tcvn3: "hint.code_table.tcvn3",
        .vniWindows: "hint.code_table.vni_windows",
        .unicodeCompound: "hint.code_table.unicode_compound",
    ]
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

    private var inputLabel: String {
        Self.inputChoices.first { $0.value == state.settings.inputType }?.label ?? ""
    }

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.typing")) {
            SettingsHero(inputLabel: inputLabel)

            SettingsHeading(UIStrings.tr("typing.group.input"))
            SettingsGroup {
                inputTypeRow
                SettingsCard {
                    SettingsToggleLine(
                        title: UIStrings.tr("smart.spellCheck"), subtitle: UIStrings.tr("smart.spellCheck.detail"),
                        isOn: state.binding(\.checkSpelling))
                    RevealGroup(isShown: state.settings.checkSpelling) {
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.restore"), subtitle: UIStrings.tr("smart.restore.detail"),
                            isOn: state.binding(\.restoreIfWrongSpelling))
                    }
                }
                SettingsCard {
                    SettingsToggleLine(
                        title: UIStrings.tr("smart.english"), subtitle: UIStrings.tr("smart.english.detail"), beta: true,
                        isOn: state.binding(\.restoreEnglish))
                    RevealGroup(isShown: state.settings.restoreEnglish) {
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.englishUndo"), subtitle: UIStrings.tr("smart.englishUndo.detail"),
                            isOn: state.binding(\.undoEnglishRestore))
                        SettingsDivider()
                        SettingsWordListLine(
                            title: UIStrings.tr("smart.englishExtra"), subtitle: UIStrings.tr("smart.englishExtra.detail"),
                            placeholder: "rust react", keyPath: \.englishWordsExtra)
                        SettingsDivider()
                        SettingsWordListLine(
                            title: UIStrings.tr("smart.englishExcluded"), subtitle: UIStrings.tr("smart.englishExcluded.detail"),
                            placeholder: "was", keyPath: \.englishWordsExcluded)
                    }
                }
                SettingsToggleRow(
                    title: UIStrings.tr("smart.perApp"), subtitle: UIStrings.tr("smart.perApp.detail"),
                    isOn: state.binding(\.rememberAppMode))
                SettingsToggleRow(
                    title: UIStrings.tr("smart.escUndo"), subtitle: UIStrings.tr("smart.escUndo.detail"),
                    isOn: state.binding(\.undoWord))
            }

            SettingsHeading(UIStrings.tr("smart.capitalize"))
            SettingsGroup {
                SettingsCard {
                    SettingsToggleLine(
                        title: UIStrings.tr("smart.autoCapitalize"), subtitle: UIStrings.tr("smart.caps_first.desc"),
                        isOn: state.binding(\.upperCaseFirstChar))
                    RevealGroup(isShown: state.settings.upperCaseFirstChar) {
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.capsPunct"), subtitle: UIStrings.tr("smart.capsPunct.detail"),
                            isOn: state.binding(\.upperCaseAfterPunct))
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.capsEnter"), subtitle: UIStrings.tr("smart.capsEnter.detail"),
                            isOn: state.binding(\.upperCaseAfterEnter))
                        SettingsDivider()
                        SettingsToggleLine(
                            title: UIStrings.tr("smart.capsFirst"), subtitle: UIStrings.tr("smart.capsFirst.detail"),
                            isOn: state.binding(\.upperCaseFirstLetter))
                    }
                }
            }

            SettingsAdvanced(pane: "typing") {
                SettingsHeading(UIStrings.tr("typing.toneHeading"))
                SettingsGroup {
                    SettingsChoiceRow(
                        title: UIStrings.tr("typing.tonePlacement"), subtitle: UIStrings.tr("typing.tonePlacement.detail"),
                        choices: Self.toneChoices, selection: state.binding(\.tonePlacement), hints: Self.toneHints)
                    SettingsMenuRow(
                        title: UIStrings.tr("typing.codeTable"), subtitle: UIStrings.tr("typing.codeTable.detail"),
                        choices: Self.tableChoices, selection: state.binding(\.codeTable))
                    SettingsToggleRow(
                        title: UIStrings.tr("typing.freeTone"), subtitle: UIStrings.tr("typing.freeTone.detail"),
                        isOn: state.binding(\.freeMark))
                }

                SettingsHeading(UIStrings.tr("smart.quickTyping"))
                SettingsGroup {
                    RevealGroup(isShown: state.settings.inputType != .vni) {
                        SettingsToggleRow(
                            title: UIStrings.tr("smart.quickTelex"), subtitle: "cc→ch, gg→gi, kk→kh, nn→ng, qq→qu, pp→ph, tt→th",
                            isOn: state.binding(\.quickTelex))
                    }
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
                        RevealGroup(isShown: state.settings.allowConsonantZfwj) {
                            SettingsDivider()
                            SettingsConsonantChips()
                        }
                    }
                }
            }
        }
    }

    /// Kiểu gõ: menu thả xuống (có nhiều lựa chọn) kèm gợi ý của lựa chọn đang dùng.
    private var inputTypeRow: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 12) {
                SettingsRowLabel(
                    title: UIStrings.tr("typing.inputMethod"), subtitle: UIStrings.tr("typing.inputMethod.detail"))
                Spacer(minLength: 12)
                SettingsDropdown(choices: Self.inputChoices, selection: state.binding(\.inputType))
            }
            if let key = Self.inputHints[state.settings.inputType] {
                Text(UIStrings.tr(key))
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }
}
