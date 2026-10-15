import SwiftUI

/// Nút có phản hồi bấm: lún về 0.95 rồi nảy lại. `primary` dùng nền `brand-fill` chữ trắng; còn lại là nút thường
/// (nền `control`, hoặc `lg-btn` khi chất liệu là Liquid Glass). `chrome` false chỉ giữ hiệu ứng bấm, không vẽ nền
/// (dùng cho mục sidebar, nơi nền là pill trượt).
struct PressButtonStyle: ButtonStyle {
    var primary = false
    var chrome = true

    @Environment(\.colorScheme) private var scheme
    @Environment(\.isEnabled) private var isEnabled
    @ObservedObject private var appearance = GlassAppearance.shared

    @ViewBuilder func makeBody(configuration: Configuration) -> some View {
        if chrome { styled(configuration) } else { pressed(configuration.label, configuration.isPressed, scale: 0.97) }
    }

    private func styled(_ configuration: Configuration) -> some View {
        let tokens = Tokens(scheme)
        let shape = RoundedRectangle(cornerRadius: 8, style: .continuous)
        return pressed(
            configuration.label
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(primary ? Color.white : tokens.fg)
                .padding(.horizontal, 12)
                .padding(.vertical, 5)
                .background {
                    if primary {
                        BrandFill().clipShape(shape).shadow(color: appearance.accent.shadow, radius: 4, y: 2)
                    } else {
                        shape.fill(appearance.material == .glass ? tokens.lgBtn : tokens.control)
                    }
                }
                .overlay { if !primary && appearance.material == .glass { Color.clear.liquidGlassEdge(cornerRadius: 8) } },
            configuration.isPressed, scale: 0.95)
    }

    private func pressed(_ label: some View, _ isPressed: Bool, scale: CGFloat) -> some View {
        label
            .opacity(isEnabled ? 1 : 0.45)
            .scaleEffect(isPressed ? scale : 1)
            .motion(isPressed ? Motion.press : Motion.bounce, value: isPressed)
            .pointerCursor()
    }
}
