import SwiftUI

/// Dropdown cho danh sách dài (kiểu gõ, bảng mã, ngôn ngữ…): menu gốc, khung giống nút, chevron bên phải, rộng tối thiểu 156.
struct SettingsDropdown<Value: Hashable>: View {
    let choices: [ChoiceOption<Value>]
    @Binding var selection: Value

    @Environment(\.colorScheme) private var scheme
    @ObservedObject private var appearance = GlassAppearance.shared

    var body: some View {
        let tokens = Tokens(scheme)
        Menu {
            ForEach(choices) { choice in
                Button {
                    selection = choice.value
                } label: {
                    if choice.value == selection { Label(choice.label, systemImage: "checkmark") } else { Text(choice.label) }
                }
            }
        } label: {
            HStack(spacing: 6) {
                Text(choices.first { $0.value == selection }?.label ?? "")
                    .font(.system(size: 13, weight: .medium))
                    .lineLimit(1)
                Spacer(minLength: 4)
                Image(systemName: "chevron.up.chevron.down")
                    .font(.system(size: 9, weight: .bold))
                    .foregroundStyle(tokens.fgMuted)
            }
            .foregroundStyle(tokens.fg)
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            .frame(minWidth: 156)
            .background(appearance.material == .glass ? tokens.lgBtn : tokens.control,
                        in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        }
        .menuStyle(.borderlessButton)
        .menuIndicator(.hidden)
        .fixedSize()
        .pointerCursor()
    }
}
