import SwiftUI

/// The extra layers of the Windows action panel's liquid glass, drawn over `glassSurface`: a specular sheen from the
/// top-left and a bright inner rim that fades towards the bottom edge.
private struct LiquidGlassEdge: ViewModifier {
    let cornerRadius: CGFloat
    @Environment(\.colorScheme) private var scheme

    func body(content: Content) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
        let dark = scheme == .dark
        let sheen = dark ? [0.16, 0.03, 0.0, 0.05] : [0.55, 0.08, 0.0, 0.12]
        let rim = dark ? [0.28, 0.10] : [0.85, 0.40]
        content
            .overlay {
                shape
                    .fill(LinearGradient(
                        stops: [
                            .init(color: .white.opacity(sheen[0]), location: 0),
                            .init(color: .white.opacity(sheen[1]), location: 0.35),
                            .init(color: .white.opacity(sheen[2]), location: 0.6),
                            .init(color: .white.opacity(sheen[3]), location: 1),
                        ],
                        startPoint: .topLeading, endPoint: .bottomTrailing))
                    .allowsHitTesting(false)
            }
            .overlay {
                shape
                    .strokeBorder(
                        LinearGradient(
                            colors: [.white.opacity(rim[0]), .white.opacity(rim[1])],
                            startPoint: .top, endPoint: .bottom),
                        lineWidth: 1)
                    .allowsHitTesting(false)
            }
    }
}

extension View {
    /// Sheen and inner rim for a glass panel with rounded corners of `cornerRadius`.
    func liquidGlassEdge(cornerRadius: CGFloat) -> some View {
        modifier(LiquidGlassEdge(cornerRadius: cornerRadius))
    }
}
