import SwiftUI

/// Thanh chọn 2–3 lựa chọn ngắn: viên `brand-fill` trượt giữa các ô (340ms), chữ đang chọn màu trắng.
struct SettingsSegmented<Value: Hashable>: View {
    let choices: [ChoiceOption<Value>]
    @Binding var selection: Value

    @Namespace private var thumb
    @Environment(\.colorScheme) private var scheme
    @ObservedObject private var appearance = GlassAppearance.shared

    var body: some View {
        let tokens = Tokens(scheme)
        HStack(spacing: 0) {
            ForEach(choices) { choice in
                let selected = selection == choice.value
                Text(choice.label)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(selected ? Color.white : tokens.fg)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 5)
                    .background {
                        if selected {
                            BrandFill()
                                .clipShape(Capsule())
                                .shadow(color: appearance.accent.shadow, radius: 4, y: 2)
                                .matchedGeometryEffect(id: "thumb", in: thumb)
                        }
                    }
                    .contentShape(Rectangle())
                    .pointerCursor()
                    .onTapGesture { selection = choice.value }
            }
        }
        .padding(2)
        .background(appearance.material == .glass ? tokens.lgBtn : tokens.control, in: Capsule())
        .motion(Motion.slide(), value: selection)
    }
}
