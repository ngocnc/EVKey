import SwiftUI

/// Lớp thêm của Liquid Glass: vệt sáng chéo từ góc trên trái (vẽ DƯỚI nội dung, để không phủ trắng lên chữ) và viền
/// sáng mảnh nhạt dần về phía dưới (vẽ trên).
private struct LiquidGlassEdge: ViewModifier {
    let cornerRadius: CGFloat
    @Environment(\.colorScheme) private var scheme

    func body(content: Content) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
        let dark = scheme == .dark
        let sheen = dark ? [0.16, 0.03, 0.0, 0.05] : [0.55, 0.08, 0.0, 0.12]
        let rim = dark ? [0.28, 0.10] : [0.85, 0.40]
        content
            .background {
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
