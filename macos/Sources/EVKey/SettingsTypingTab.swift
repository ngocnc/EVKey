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

    private static let toneChoices: [ChoiceOption<TonePlacement>] = [
        ChoiceOption(value: .new, label: "Kiểu mới (hòa)"),
        ChoiceOption(value: .old, label: "Kiểu cũ (hoà)"),
    ]

    private static let tableChoices: [ChoiceOption<CodeTable>] = [
        ChoiceOption(value: .unicode, label: "Unicode"),
        ChoiceOption(value: .tcvn3, label: "TCVN3"),
        ChoiceOption(value: .vniWindows, label: "VNI Windows"),
        ChoiceOption(value: .unicodeCompound, label: "Unicode tổ hợp"),
    ]

    var body: some View {
        SettingsPage(title: "Gõ tiếng Việt") {
            SettingsGroup {
                HStack(spacing: 12) {
                    Image(systemName: state.accessibilityTrusted ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                        .foregroundStyle(state.accessibilityTrusted ? Color.green : Color.orange)
                        .font(.system(size: 18))
                    SettingsRowLabel(
                        title: state.accessibilityTrusted ? "Đã cấp quyền Trợ năng" : "Chưa cấp quyền Trợ năng",
                        subtitle: state.accessibilityTrusted ? "EVKey đang xử lý phím gõ." : "Cần cấp quyền để EVKey gõ được tiếng Việt.")
                    Spacer(minLength: 12)
                    if !state.accessibilityTrusted {
                        Button("Mở cài đặt", action: state.openAccessibilitySettings)
                            .pointerCursor()
                    }
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .settingsBlock()
            }
            SettingsGroup {
                SettingsChoiceRow(
                    title: "Kiểu gõ", subtitle: "Cách gõ dấu và chữ đặc biệt.",
                    choices: Self.inputChoices, selection: state.binding(\.inputType))
                SettingsChoiceRow(
                    title: "Vị trí dấu", subtitle: "Dấu trên nguyên âm đôi như oa, oe, uy.",
                    choices: Self.toneChoices, selection: state.binding(\.tonePlacement))
                SettingsChoiceRow(
                    title: "Bảng mã", subtitle: "Unicode phù hợp với hầu hết ứng dụng.",
                    choices: Self.tableChoices, selection: state.binding(\.codeTable))
                SettingsToggleRow(
                    title: "Đặt dấu tự do", subtitle: "Cho phép gõ dấu sau khi đã kết thúc âm tiết.",
                    isOn: state.binding(\.freeMark))
            }
        }
    }
}
