import AppKit
import Combine
import EVKeyKit
import SwiftUI

/// "Dịch thuật" (Beta): the translate hotkey, what happens after a translation, how long the popup stays, and the service.
struct SettingsTranslateTab: View {
    @ObservedObject private var state = AppState.shared
    @State private var recording = false
    @State private var message: String?
    /// Key-down monitor while recording; removed as soon as recording ends.
    @State private var monitor: Any?
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

    var body: some View {
        SettingsPage(title: UIStrings.tr("tab.translate")) {
            SettingsGroup {
                hotkeyRow
                SettingsChoiceRow(
                    title: UIStrings.tr("translate.action"), subtitle: UIStrings.tr("translate.action.detail"),
                    choices: actionChoices, selection: state.binding(\.translateAction))
                displayTimeRow
                serviceRow
            }
        }
        .onAppear(perform: loadDisplayTime)
        .onDisappear(perform: stopRecording)
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didResignKeyNotification)) { _ in
            stopRecording()
        }
    }

    // MARK: Hotkey

    private var hotkeyRow: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 12) {
                SettingsRowLabel(title: UIStrings.tr("translate.hotkey"), subtitle: UIStrings.tr("translate.hotkey.detail"))
                Spacer(minLength: 12)
                Button(action: toggleRecording) { hotkeyCaps }
                    .buttonStyle(.plain)
                Button(UIStrings.tr("keys.hotkey.none"), action: clearHotkey)
                    .buttonStyle(.plain)
                    .foregroundStyle(Color.primary.opacity(0.7))
                    .disabled(state.settings.translateHotkey == "none")
            }
            if let message {
                Text(message)
                    .font(.system(size: 11))
                    .foregroundStyle(.red)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }

    /// Key caps for the current chord, or the recording prompt, or the unset text.
    private var hotkeyCaps: some View {
        Group {
            if recording {
                Text(UIStrings.tr("translate.hotkey.recording"))
            } else if state.settings.translateHotkey == "none" {
                Text(UIStrings.tr("translate.hotkey.unset"))
            } else {
                HStack(spacing: 4) {
                    ForEach(TranslateHotkey.labels(state.settings.translateHotkey), id: \.self) { label in
                        Text(label)
                            .font(.system(size: 12, weight: .medium))
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.primary.opacity(0.1), in: RoundedRectangle(cornerRadius: 5, style: .continuous))
                    }
                }
            }
        }
        .font(.system(size: 12, weight: .medium))
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
        .background(
            recording ? Color.accentColor.opacity(0.18) : Color.primary.opacity(0.07),
            in: RoundedRectangle(cornerRadius: 7, style: .continuous))
    }

    private func toggleRecording() {
        if recording { stopRecording() } else { startRecording() }
    }

    /// Captures the next key combination. Esc cancels; other keys are swallowed while recording.
    private func startRecording() {
        stopRecording()
        message = nil
        recording = true
        monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            if event.keyCode == 53 {  // Esc
                self.message = nil
                self.stopRecording()
                return nil
            }
            let flags = event.modifierFlags
            switch TranslateHotkey.capture(
                keycode: event.keyCode,
                control: flags.contains(.control), option: flags.contains(.option),
                shift: flags.contains(.shift), command: flags.contains(.command)) {
            case .chord(let chord):
                self.state.updateSettings { $0.translateHotkey = chord }
                self.stopRecording()
            case .needsModifier:
                self.message = UIStrings.tr("translate.hotkey.needModifier")
            case .badKey:
                self.message = UIStrings.tr("translate.hotkey.badKey")
            }
            return nil
        }
    }

    private func stopRecording() {
        if let monitor { NSEvent.removeMonitor(monitor) }
        monitor = nil
        recording = false
    }

    private func clearHotkey() {
        stopRecording()
        message = nil
        state.updateSettings { $0.translateHotkey = "none" }
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

    // MARK: Service

    private var serviceRow: some View {
        HStack(spacing: 12) {
            SettingsRowLabel(title: UIStrings.tr("translate.service"), subtitle: UIStrings.tr("translate.service.detail"))
            Spacer(minLength: 12)
            Text("Google Translate")
                .font(.system(size: 12, weight: .medium))
                .padding(.horizontal, 9)
                .padding(.vertical, 4)
                .background(Color.primary.opacity(0.09), in: Capsule())
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }
}
