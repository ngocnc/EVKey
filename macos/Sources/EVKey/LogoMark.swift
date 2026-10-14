import SwiftUI

/// The EVKey mark: the outlined letter box of the menu bar icon, filled with a linear gradient. The gradient follows the
/// Liquid Glass tint (start and, when set, end colour); with no gradient tint it stays cyan-indigo.
struct LogoMark: View {
    var letter = "V"
    var size: CGFloat = 34
    @ObservedObject private var glass = GlassAppearance.shared

    private var gradient: LinearGradient {
        let colors = glass.tintEndHex.isEmpty
            ? [Color(red: 0.39, green: 0.82, blue: 1.0), Color(red: 0.37, green: 0.36, blue: 0.9)]
            : [ColorHex.color(glass.tintHex(.dark)), ColorHex.color(glass.tintEndHex)]
        return LinearGradient(colors: colors, startPoint: .topLeading, endPoint: .bottomTrailing)
    }

    var body: some View {
        RoundedRectangle(cornerRadius: size * 0.26, style: .continuous)
            .strokeBorder(gradient, lineWidth: size * 0.07)
            .overlay {
                Text(letter)
                    .font(.system(size: size * 0.6, weight: .heavy))
                    .foregroundStyle(gradient)
            }
            .frame(width: size, height: size)
    }
}
