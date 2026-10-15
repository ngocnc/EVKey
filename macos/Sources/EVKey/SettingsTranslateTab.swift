import EVKeyKit
import SwiftUI

/// "Dịch thuật": the translate and action panel hotkeys, what happens after a result, how long the popup stays,
/// the translation service, and the Gemini key.
struct SettingsTranslateTab: View {
    @ObservedObject private var state = AppState.shared
    /// Id of the hotkey row that is recording a chord, if any.
    @State private var recorder: String?
    @State private var timeText = ""
    @State private var timeUnit = TranslateDisplayTime.DisplayUnit.s
    @FocusState private var timeFocused: Bool

    private var actionChoices: [ChoiceOption<TranslateAction>] {
        [
            ChoiceOption(value: .show, label: UIStrings.tr("translate.action.show")),
            ChoiceOption(value: .copy, label: UIStrings.tr("translate.action.copy")),
            ChoiceOption(value: .replace, label: UIStrings.tr("translate.action.replace")),
        ]
    }

    private let actionHints: [TranslateAction: String] = [
        .show: "hint.translate_action.show",
        .copy: "hint.translate_action.copy",
        .replace: "hint.translate_action.replace",
    ]

    private let providerChoices: [ChoiceOption<TranslateProvider>] = [
        ChoiceOption(value: .google, label: "Google Translate"),
        ChoiceOption(value: .gemini, label: "Gemini"),
    ]

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.translate")) {
            SettingsGroup {
                SettingsHotkeyRow(
                    id: "translate", title: UIStrings.tr("translate.hotkey"), subtitle: UIStrings.tr("translate.hotkey.detail"),
                    chord: state.binding(\.translateHotkey), recorder: $recorder,
                    others: [state.settings.actionHotkey, state.settings.toggleHotkey],
                    conflictNames: [
                        state.settings.actionHotkey: UIStrings.tr("actions.hotkey"),
                        state.settings.toggleHotkey: UIStrings.tr("keys.toggle"),
                    ])
                SettingsHotkeyRow(
                    id: "actions", title: UIStrings.tr("actions.hotkey"), subtitle: UIStrings.tr("actions.hotkey.detail"),
                    chord: state.binding(\.actionHotkey), recorder: $recorder,
                    others: [state.settings.translateHotkey, state.settings.toggleHotkey],
                    conflictNames: [
                        state.settings.translateHotkey: UIStrings.tr("translate.hotkey"),
                        state.settings.toggleHotkey: UIStrings.tr("keys.toggle"),
                    ])
                SettingsChoiceRow(
                    title: UIStrings.tr("translate.action"), subtitle: UIStrings.tr("translate.action.detail"),
                    choices: actionChoices, selection: state.binding(\.translateAction), hints: actionHints)
                SettingsMenuRow(
                    title: UIStrings.tr("translate.service"), subtitle: UIStrings.tr("translate.service.detail"),
                    choices: providerChoices, selection: state.binding(\.translateProvider))
            }

            SettingsAdvanced(pane: "translate") {
                SettingsGroup { displayTimeRow }
            }

            SettingsHeading(UIStrings.tr("gemini.heading"))
            SettingsGeminiKeyRow()
        }
        .onAppear(perform: loadDisplayTime)
    }

    // MARK: Display time

    private var displayTimeRow: some View {
        HStack(spacing: 12) {
            SettingsRowLabel(title: UIStrings.tr("translate.displayTime"), subtitle: UIStrings.tr("translate.displayTime.detail"))
            Spacer(minLength: 12)
            TextField("", text: $timeText)
                .textFieldStyle(.roundedBorder)
                .multilineTextAlignment(.trailing)
                .frame(width: 70)
                .focused($timeFocused)
                .disabled(timeUnit == .never)
                .onChange(of: timeText) { text in
                    let digits = text.filter { $0.isASCII && $0.isNumber }
                    if digits != text { timeText = digits }
                }
                .onSubmit { commitDisplayTime() }
                .onChange(of: timeFocused) { focused in
                    if !focused { commitDisplayTime() }
                }
            Picker("", selection: unitBinding) {
                ForEach(TranslateDisplayTime.DisplayUnit.allCases, id: \.self) { unit in
                    Text(unitLabel(unit)).tag(unit)
                }
            }
            .pickerStyle(.menu)
            .labelsHidden()
            .fixedSize()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }

    private var unitBinding: Binding<TranslateDisplayTime.DisplayUnit> {
        Binding(get: { timeUnit }, set: { changeDisplayUnit(to: $0) })
    }

    private func unitLabel(_ unit: TranslateDisplayTime.DisplayUnit) -> String {
        switch unit {
        case .ms: return UIStrings.tr("translate.unit.ms")
        case .s: return UIStrings.tr("translate.unit.s")
        case .never: return UIStrings.tr("translate.unit.never")
        }
    }

    private func loadDisplayTime() {
        let ms = state.settings.translatePopupMs
        timeUnit = TranslateDisplayTime.unit(forMs: ms)
        timeText = timeUnit == .never ? "" : String(TranslateDisplayTime.shownValue(ms: ms, unit: timeUnit))
    }

    /// The same time in the new unit: the number is converted from the stored value, then saved.
    private func changeDisplayUnit(to unit: TranslateDisplayTime.DisplayUnit) {
        timeUnit = unit
        let ms = state.settings.translatePopupMs
        timeText = unit == .never ? "" : String(TranslateDisplayTime.shownValue(ms: ms, unit: unit))
        commitDisplayTime()
    }

    /// Saves the typed number in the chosen unit (clamped), then shows the value as stored.
    private func commitDisplayTime() {
        let ms = TranslateDisplayTime.storedMs(value: Int(timeText) ?? 0, unit: timeUnit)
        state.updateSettings { $0.translatePopupMs = ms }
        timeText = timeUnit == .never ? "" : String(TranslateDisplayTime.shownValue(ms: ms, unit: timeUnit))
    }
}
