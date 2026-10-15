import SwiftUI

/// Logo EVKey: ô bo góc tô gradient `logo` của màu nhấn (a → m → c, đốm sáng góc trên trái), chữ V trắng.
struct LogoMark: View {
    var letter = "V"
    var size: CGFloat = 34
    @ObservedObject private var glass = GlassAppearance.shared

    var body: some View {
        let accent = glass.accent
        let shape = RoundedRectangle(cornerRadius: size * 0.28, style: .continuous)
        shape
            .fill(LinearGradient(
                stops: [.init(color: accent.a, location: 0), .init(color: accent.m, location: 0.6), .init(color: accent.c, location: 1)],
                startPoint: .topLeading, endPoint: .bottomTrailing))
            .overlay {
                shape.fill(RadialGradient(
                    colors: [Color.white.opacity(0.45), .clear],
                    center: UnitPoint(x: 0.25, y: 0.15), startRadius: 0, endRadius: size * 0.6))
            }
            .overlay { shape.strokeBorder(Color.white.opacity(0.35), lineWidth: 0.5) }
            .overlay {
                Text(letter)
                    .font(.system(size: size * 0.5, weight: .heavy))
                    .foregroundStyle(.white)
                    .shadow(color: .black.opacity(0.25), radius: 1, y: 1)
            }
            .frame(width: size, height: size)
            .shadow(color: accent.shadow, radius: size * 0.15, y: size * 0.06)
    }
}
