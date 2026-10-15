import SwiftUI

/// Hàng bấm toàn chiều rộng của popup: rê chuột thì nền `brand-fill` chữ trắng (như menu macOS), bấm thì lún nhẹ.
struct PopupRowStyle: ButtonStyle {
    var leadingInset: CGFloat = 10

    func makeBody(configuration: Configuration) -> some View {
        PopupRowHighlight(pressed: configuration.isPressed, leadingInset: leadingInset) {
            configuration.label
        }
    }
}

private struct PopupRowHighlight<Content: View>: View {
    let pressed: Bool
    let leadingInset: CGFloat
    let content: Content

    @State private var hovered = false
    @ObservedObject private var appearance = GlassAppearance.shared

    init(pressed: Bool, leadingInset: CGFloat, @ViewBuilder content: () -> Content) {
        self.pressed = pressed
        self.leadingInset = leadingInset
        self.content = content()
    }

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: 8, style: .continuous)
        content
            .foregroundStyle(hovered ? Color.white : Color.primary)
            .padding(.leading, leadingInset)
            .padding(.trailing, 10)
            .padding(.vertical, 6)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background {
                if hovered {
                    BrandFill().clipShape(shape).shadow(color: appearance.accent.shadow, radius: 5, y: 2)
                }
            }
            .contentShape(Rectangle())
            .scaleEffect(pressed ? 0.97 : 1)
            .onHover { hovered = $0 }
            .motion(.easeOut(duration: 0.12), value: hovered)
            .motion(pressed ? Motion.press : Motion.bounce, value: pressed)
    }
}
