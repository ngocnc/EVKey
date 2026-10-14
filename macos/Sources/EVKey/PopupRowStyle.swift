import SwiftUI

/// Full-width popup button style with hover and pressed highlights.
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

    init(pressed: Bool, leadingInset: CGFloat, @ViewBuilder content: () -> Content) {
        self.pressed = pressed
        self.leadingInset = leadingInset
        self.content = content()
    }

    var body: some View {
        content
            .padding(.leading, leadingInset)
            .padding(.trailing, 10)
            .padding(.vertical, 6)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                RoundedRectangle(cornerRadius: 7, style: .continuous)
                    .fill(pressed ? Color.primary.opacity(0.14) : (hovered ? Color.primary.opacity(0.07) : Color.clear))
            )
            .contentShape(Rectangle())
            .onHover { hovered = $0 }
            .animation(.easeOut(duration: 0.12), value: hovered)
            .animation(.easeOut(duration: 0.08), value: pressed)
    }
}
