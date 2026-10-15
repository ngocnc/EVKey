import EVKeyKit
import SwiftUI

extension Color {
    /// Colour from 0xRRGGBB, in sRGB.
    init(hex: UInt32, opacity: Double = 1) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255,
            opacity: opacity)
    }
}

/// Màu nhấn (design-system/evkey/MASTER.md). `a` là màu nhạt cho quầng sáng và logo; `b → m → c` là 3 điểm của gradient,
/// chữ trắng đạt tương phản ≥ 5.1:1 trên cả ba.
enum Accent: String, CaseIterable, Identifiable {
    case ocean, orchid, sunset, mint, graphite

    var id: String { rawValue }
    var label: String { UIStrings.tr("accent.\(rawValue)") }

    private var hexes: (a: UInt32, b: UInt32, m: UInt32, c: UInt32) {
        switch self {
        case .ocean: (0x64D1FF, 0x1270B0, 0x3A66D6, 0x5B4FE0)
        case .orchid: (0xE0A8FF, 0x7A3AD0, 0xA0359F, 0xC02E6E)
        case .sunset: (0xFFC27A, 0xC2410C, 0xC8325E, 0x9B3BC4)
        case .mint: (0x7EE8D8, 0x0F766E, 0x0E7490, 0x0369A1)
        case .graphite: (0xC3CAD8, 0x3C4250, 0x4E5566, 0x2C303A)
        }
    }

    var a: Color { Color(hex: hexes.a) }
    var b: Color { Color(hex: hexes.b) }
    var m: Color { Color(hex: hexes.m) }
    var c: Color { Color(hex: hexes.c) }
    /// Bóng màu dưới các phần tử brand-fill.
    var shadow: Color { m.opacity(0.4) }
}

/// Chất liệu của thẻ, nút và thanh chọn: đặc, hoặc Liquid Glass.
enum SurfaceMaterial: String, CaseIterable, Identifiable {
    case plain, glass

    var id: String { rawValue }
    var label: String { UIStrings.tr(self == .plain ? "material.plain" : "material.glass") }
}

/// Nền có chữ trắng (viên chọn, switch bật, nút chính, mục sidebar đang chọn): gradient chéo 135° `b → m → c` cùng
/// quầng sáng `a` ở góc trên trái. Đọc màu nhấn đang chọn từ `GlassAppearance`.
struct BrandFill: View {
    @ObservedObject private var appearance = GlassAppearance.shared
    /// Ép một màu cố định (ô chọn màu nhấn); nil theo màu đang chọn.
    var accent: Accent?

    var body: some View {
        let accent = accent ?? appearance.accent
        GeometryReader { proxy in
            ZStack {
                LinearGradient(
                    stops: [
                        .init(color: accent.b, location: 0),
                        .init(color: accent.m, location: 0.55),
                        .init(color: accent.c, location: 1),
                    ],
                    startPoint: .topLeading, endPoint: .bottomTrailing)
                RadialGradient(
                    colors: [accent.a.opacity(0.45), .clear],
                    center: .topLeading, startRadius: 0, endRadius: max(proxy.size.width * 0.8, proxy.size.height * 1.1) * 0.45)
            }
        }
    }
}

/// Track slider và thanh tiến trình: gradient ngang `b → m → c`.
struct BrandLine: View {
    @ObservedObject private var appearance = GlassAppearance.shared

    var body: some View {
        let accent = appearance.accent
        LinearGradient(colors: [accent.b, accent.m, accent.c], startPoint: .leading, endPoint: .trailing)
    }
}

/// Màu của token trong MASTER.md theo sáng/tối.
struct Tokens {
    let dark: Bool

    init(_ scheme: ColorScheme) { dark = scheme == .dark }

    var fg: Color { dark ? Color(hex: 0xF5F5F7) : Color(hex: 0x1D1D1F) }
    var fgMuted: Color { dark ? Color(hex: 0xA1A1A6) : Color(hex: 0x5B5B60) }
    var card: Color { dark ? .white.opacity(0.06) : .white.opacity(0.80) }
    var cardBorder: Color { dark ? .white.opacity(0.10) : .black.opacity(0.08) }
    var lgCard: Color { dark ? .white.opacity(0.07) : .white.opacity(0.46) }
    var lgBtn: Color { dark ? .white.opacity(0.11) : .white.opacity(0.55) }
    var separator: Color { dark ? .white.opacity(0.08) : .black.opacity(0.08) }
    var control: Color { dark ? .white.opacity(0.10) : .black.opacity(0.06) }
    var switchOff: Color { dark ? .white.opacity(0.16) : .black.opacity(0.16) }
    var hover: Color { dark ? .white.opacity(0.06) : .black.opacity(0.05) }
    var rimTop: Color { .white.opacity(dark ? 0.38 : 0.95) }
    var rimBottom: Color { .white.opacity(dark ? 0.06 : 0.22) }
    var sheen: Color { .white.opacity(dark ? 0.12 : 0.55) }
    var success: Color { dark ? Color(hex: 0x30D158) : Color(hex: 0x1E8E3E) }
    var warning: Color { dark ? Color(hex: 0xFFB340) : Color(hex: 0xB25000) }
    var danger: Color { dark ? Color(hex: 0xFF6961) : Color(hex: 0xC62828) }
}

/// Đường cong chuyển động chung (MASTER.md › Motion).
enum Motion {
    /// Viên trượt, đổi trang, mở "Nâng cao": cubic-bezier(.32,.72,0,1).
    static func slide(_ duration: Double = 0.34) -> Animation { .timingCurve(0.32, 0.72, 0, 1, duration: duration) }
    /// Thả nút: nảy lại sau khi lún.
    static let bounce = Animation.timingCurve(0.3, 1.6, 0.5, 1, duration: 0.26)
    /// Bấm nút: lún xuống nhanh.
    static let press = Animation.easeOut(duration: 0.09)
}

private struct MotionModifier<V: Equatable>: ViewModifier {
    let animation: Animation
    let value: V
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    func body(content: Content) -> some View {
        content.animation(reduceMotion ? nil : animation, value: value)
    }
}

extension View {
    /// `.animation(_:value:)` nhưng tắt khi bật Reduce Motion.
    func motion<V: Equatable>(_ animation: Animation, value: V) -> some View {
        modifier(MotionModifier(animation: animation, value: value))
    }
}
