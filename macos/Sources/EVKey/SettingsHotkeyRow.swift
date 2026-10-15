import AppKit
import Combine
import EVKeyKit
import SwiftUI

/// Ô ghi phím tắt nội tuyến (MASTER > Control > Ô phím tắt). Bấm ô, nhấn tổ hợp; Esc hủy; × xoá về `none`.
/// Tổ hợp chỉ có phím bổ trợ (ví dụ `ctrl+shift`) chỉ nhận khi `allowsModifierOnly`. Chỉ một hàng ghi tại một thời điểm:
/// `recorder` giữ id của hàng đang ghi.
struct SettingsHotkeyRow: View {
    let id: String
    let title: String
    let subtitle: String
    @Binding var chord: String
    @Binding var recorder: String?
    /// Tổ hợp của các hàng khác. Tổ hợp trùng bị từ chối và giữ giá trị cũ.
    var others: [String] = []
    /// Tên hiển thị của hàng khác theo tổ hợp, để báo “Đã dùng cho “<tên>”.”. Thiếu tên thì báo chung.
    var conflictNames: [String: String] = [:]
    /// Cho phép tổ hợp chỉ có phím bổ trợ (chỉ dùng cho phím chuyển Việt/Anh).
    var allowsModifierOnly = false

    @State private var message: String?
    /// Monitor sự kiện cục bộ khi đang ghi; gỡ ngay khi ghi xong.
    @State private var monitor: Any?
    @State private var pulse = false
    @Environment(\.colorScheme) private var scheme
    @ObservedObject private var appearance = GlassAppearance.shared

    private var recording: Bool { recorder == id }
    private var tokens: Tokens { Tokens(scheme) }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 12) {
                SettingsRowLabel(title: title, subtitle: subtitle)
                Spacer(minLength: 12)
                HStack(spacing: 4) {
                    Button(action: toggleRecording) { field }
                        .buttonStyle(PressButtonStyle())
                        .background {
                            if recording {
                                RoundedRectangle(cornerRadius: 8, style: .continuous)
                                    .stroke(appearance.accent.m, lineWidth: 2)
                                    .padding(pulse ? -5 : -2)
                                    .opacity(pulse ? 0.3 : 1)
                            }
                        }
                    if chord != "none" {
                        Button(action: clear) {
                            Image(systemName: "xmark")
                                .font(.system(size: 9, weight: .bold))
                                .foregroundStyle(tokens.fgMuted)
                                .frame(width: 22, height: 22)
                                .contentShape(Circle())
                        }
                        .buttonStyle(.plain)
                        .help(UIStrings.tr("hotkey.clear"))
                    }
                }
            }
            if let message {
                Text(message)
                    .font(.system(size: 11))
                    .foregroundStyle(tokens.danger)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
        .onChange(of: recorder) { _ in
            if recording {
                startPulse()
            } else {
                removeMonitor()
                pulse = false
            }
        }
        .onDisappear(perform: stopRecording)
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didResignKeyNotification)) { _ in
            stopRecording()
        }
    }

    /// Nội dung ô: dòng nhắc khi đang ghi, "Chưa đặt" khi chưa có tổ hợp, hoặc keycap của tổ hợp.
    private var field: some View {
        Group {
            if recording {
                Text(UIStrings.tr("hotkey.recording"))
                    .font(.system(size: 12))
                    .foregroundStyle(tokens.fgMuted)
            } else if chord == "none" {
                Text(UIStrings.tr("hotkey.unset"))
                    .font(.system(size: 12))
                    .foregroundStyle(tokens.fgMuted)
            } else {
                keycaps
            }
        }
        .frame(minWidth: 108, minHeight: 18)
    }

    /// Mỗi phần của tổ hợp một keycap, ví dụ ⌃ ⌥ T.
    private var keycaps: some View {
        let parts = TranslateHotkey.labels(chord)
        return HStack(spacing: 4) {
            ForEach(parts.indices, id: \.self) { index in
                Text(parts[index])
                    .font(.system(size: 12, weight: .semibold))
                    .frame(minWidth: 22)
                    .padding(.horizontal, 5)
                    .padding(.vertical, 2)
                    .background(RoundedRectangle(cornerRadius: 5, style: .continuous).fill(tokens.card))
            }
        }
    }

    private func toggleRecording() {
        if recording { stopRecording() } else { startRecording() }
    }

    /// Ghi tổ hợp tiếp theo qua monitor cục bộ. Phím bổ trợ đang giữ được theo dõi qua `peak` (số phím giữ cùng lúc lớn
    /// nhất); khi nhả hết và `peak` đủ hai phím, tổ hợp chỉ có phím bổ trợ được ghi nếu `allowsModifierOnly`.
    private func startRecording() {
        message = nil
        var peak: NSEvent.ModifierFlags = []
        monitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .flagsChanged]) { event in
            let flags = event.modifierFlags.intersection(HotkeyChord.modifierMask)
            if event.type == .flagsChanged {
                if !flags.isEmpty {
                    if HotkeyChord.modifierCount(flags) > HotkeyChord.modifierCount(peak) { peak = flags }
                    return nil
                }
                guard !peak.isEmpty else { return nil }
                if allowsModifierOnly && HotkeyChord.modifierCount(peak) >= 2 {
                    commit(HotkeyChord.modifierOnly(peak))
                } else {
                    message = UIStrings.tr(HotkeyChord.modifierCount(peak) < 2 ? "hotkey.modifierOnly" : "hotkey.needKey")
                }
                peak = []
                return nil
            }
            // keyDown: Esc không kèm phím bổ trợ là hủy; các phím khác phải có ít nhất một phím bổ trợ.
            if event.keyCode == 53 && flags.isEmpty {
                stopRecording()
                return nil
            }
            if event.isARepeat { return nil }
            guard !flags.isEmpty else {
                message = UIStrings.tr("hotkey.needModifier")
                return nil
            }
            switch TranslateHotkey.capture(
                keycode: event.keyCode,
                control: flags.contains(.control),
                option: flags.contains(.option),
                shift: flags.contains(.shift),
                command: flags.contains(.command)
            ) {
            case .chord(let captured):
                commit(captured)
            case .needsModifier:
                message = UIStrings.tr("hotkey.needModifier")
            case .badKey:
                message = UIStrings.tr("hotkey.badKey")
            }
            return nil
        }
        HotkeyRecording.setActive(true)
        recorder = id
    }

    /// Ghi `captured` nếu không trùng hàng khác và kết thúc ghi. Trùng thì báo lỗi và giữ giá trị cũ.
    private func commit(_ captured: String) {
        if let other = others.first(where: { HotkeyChord.same($0, captured) }) {
            message = conflictNames[other].map { UIStrings.tr("hotkey.takenBy", $0) } ?? UIStrings.tr("hotkey.taken")
        } else {
            chord = captured
            message = nil
        }
        stopRecording()
    }

    private func startPulse() {
        pulse = false
        withAnimation(.easeInOut(duration: 0.7).repeatForever(autoreverses: true)) { pulse = true }
    }

    private func stopRecording() {
        removeMonitor()
        if recording { recorder = nil }
    }

    private func removeMonitor() {
        if let monitor { NSEvent.removeMonitor(monitor) }
        monitor = nil
        HotkeyRecording.setActive(false)
    }

    private func clear() {
        stopRecording()
        message = nil
        chord = "none"
    }
}
