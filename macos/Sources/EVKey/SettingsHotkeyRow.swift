import AppKit
import Combine
import EVKeyKit
import SwiftUI

/// A row that records a key chord (`ctrl+alt+t`) into `chord`, with an "Off" button. Click the key caps, then press the
/// new combination; Esc cancels. Only one row records at a time: `recorder` holds the id of the row that does.
struct SettingsHotkeyRow: View {
    let id: String
    let title: String
    let subtitle: String
    @Binding var chord: String
    @Binding var recorder: String?
    @State private var message: String?
    /// Key-down monitor while recording; removed as soon as recording ends.
    @State private var monitor: Any?

    private var recording: Bool { recorder == id }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 12) {
                SettingsRowLabel(title: title, subtitle: subtitle)
                Spacer(minLength: 12)
                Button(action: toggleRecording) { caps }
                    .buttonStyle(.plain)
                Button(UIStrings.tr("keys.hotkey.none"), action: clear)
                    .buttonStyle(.plain)
                    .foregroundStyle(Color.primary.opacity(0.7))
                    .disabled(chord == "none")
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
        .onChange(of: recorder) { _ in
            if !recording { removeMonitor() }
        }
        .onDisappear(perform: stopRecording)
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didResignKeyNotification)) { _ in
            stopRecording()
        }
    }

    /// Key caps for the current chord, or the recording prompt, or the unset text.
    private var caps: some View {
        Group {
            if recording {
                Text(UIStrings.tr("translate.hotkey.recording"))
            } else if chord == "none" {
                Text(UIStrings.tr("translate.hotkey.unset"))
            } else {
                HStack(spacing: 4) {
                    ForEach(TranslateHotkey.labels(chord), id: \.self) { label in
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
        message = nil
        recorder = id
        removeMonitor()
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
            case .chord(let captured):
                self.chord = captured
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
        removeMonitor()
        if recording { recorder = nil }
    }

    private func removeMonitor() {
        if let monitor { NSEvent.removeMonitor(monitor) }
        monitor = nil
    }

    private func clear() {
        stopRecording()
        message = nil
        chord = "none"
    }
}
