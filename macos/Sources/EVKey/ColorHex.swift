import AppKit
import SwiftUI

/// Converts between the "#rrggbb" strings stored in config and SwiftUI colors.
enum ColorHex {
    static func color(_ hex: String) -> Color {
        Color(nsColor: nsColor(hex))
    }

    /// Returns nil when the picked color has no sRGB representation.
    static func hex(_ color: Color) -> String? {
        guard let rgb = NSColor(color).usingColorSpace(.sRGB) else { return nil }
        let r = Int((rgb.redComponent * 255).rounded())
        let g = Int((rgb.greenComponent * 255).rounded())
        let b = Int((rgb.blueComponent * 255).rounded())
        return String(format: "#%02lx%02lx%02lx", r, g, b)
    }

    /// A solid colour, or a diagonal gradient from `start` to `end` when `end` is not empty.
    static func style(_ start: String, _ end: String, opacity: Double = 1) -> AnyShapeStyle {
        guard !end.isEmpty else { return AnyShapeStyle(color(start).opacity(opacity)) }
        return AnyShapeStyle(
            LinearGradient(
                colors: [color(start).opacity(opacity), color(end).opacity(opacity)],
                startPoint: .topLeading, endPoint: .bottomTrailing))
    }

    /// A hue-shifted partner for `hex`, used as the end colour when a gradient is first switched on.
    static func companion(_ hex: String) -> String {
        guard let rgb = nsColor(hex).usingColorSpace(.sRGB) else { return hex }
        var h: CGFloat = 0, s: CGFloat = 0, b: CGFloat = 0, a: CGFloat = 0
        rgb.getHue(&h, saturation: &s, brightness: &b, alpha: &a)
        let shifted = NSColor(hue: (h + 0.12).truncatingRemainder(dividingBy: 1), saturation: max(s, 0.5),
                              brightness: max(b, 0.6), alpha: 1)
        return Self.hex(Color(nsColor: shifted)) ?? hex
    }

    private static func nsColor(_ hex: String) -> NSColor {
        let digits = hex.hasPrefix("#") ? String(hex.dropFirst()) : hex
        guard digits.count == 6, let value = UInt32(digits, radix: 16) else { return .black }
        return NSColor(
            srgbRed: CGFloat((value >> 16) & 0xff) / 255,
            green: CGFloat((value >> 8) & 0xff) / 255,
            blue: CGFloat(value & 0xff) / 255,
            alpha: 1)
    }
}
