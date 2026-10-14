import AppKit
import SwiftUI

/// Look-only preference for the Liquid Glass of the popup and Bảng điều khiển. Kept in UserDefaults, not config.toml
/// (the Windows build keeps its own glass preference in the web view's local storage).
@MainActor
final class GlassAppearance: ObservableObject {
    static let shared = GlassAppearance()

    enum Style: String, CaseIterable, Identifiable {
        case regular, clear
        var id: String { rawValue }
        var label: String { self == .regular ? "Thường" : "Trong suốt" }
    }

    /// Colour theme of every EVKey window: follow macOS, or force light or dark.
    enum Theme: String, CaseIterable, Identifiable {
        case system, light, dark
        var id: String { rawValue }
        var label: String {
            switch self {
            case .system: "Theo hệ thống"
            case .light: "Sáng"
            case .dark: "Tối"
            }
        }
        /// nil follows the system.
        var appearance: NSAppearance? {
            switch self {
            case .system: nil
            case .light: NSAppearance(named: .aqua)
            case .dark: NSAppearance(named: .darkAqua)
            }
        }
    }

    /// Untinted glass lets whatever is behind the window show through; the default tint is dark in dark mode and white in
    /// light mode, so the window never turns muddy grey in the light theme.
    static func defaultTint(_ scheme: ColorScheme) -> String { scheme == .dark ? "#000000" : "#ffffff" }
    /// A light dark tint by default: untinted glass lets whatever is behind the window show through and hurts legibility.
    static let defaultStrength = 30

    @Published var theme: Theme {
        didSet {
            defaults.set(theme.rawValue, forKey: "glass.theme")
            NSApp.appearance = theme.appearance
        }
    }
    @Published var style: Style { didSet { defaults.set(style.rawValue, forKey: "glass.style") } }
    /// "#rrggbb" tint the user picked; nil follows the theme (see `defaultTint`).
    @Published var customTintHex: String? { didSet { defaults.set(customTintHex, forKey: "glass.tint") } }
    /// "#rrggbb" end colour of a gradient tint; empty keeps the tint a single colour.
    @Published var tintEndHex: String { didSet { defaults.set(tintEndHex, forKey: "glass.tintEnd") } }
    /// 0-100: how strongly the tint covers the glass; 0 leaves the glass untinted.
    @Published var tintStrength: Int { didSet { defaults.set(tintStrength, forKey: "glass.strength") } }

    private let defaults = UserDefaults.standard

    private init() {
        theme = Theme(rawValue: defaults.string(forKey: "glass.theme") ?? "") ?? .system
        style = Style(rawValue: defaults.string(forKey: "glass.style") ?? "") ?? .regular
        customTintHex = defaults.string(forKey: "glass.tint")
        tintEndHex = defaults.string(forKey: "glass.tintEnd") ?? ""
        tintStrength = defaults.object(forKey: "glass.strength") as? Int ?? Self.defaultStrength
    }

    /// A white tint at full strength turns light glass into flat grey, so the default light tint is half as strong.
    private func lightScale(_ scheme: ColorScheme) -> Double { scheme == .light && customTintHex == nil ? 0.4 : 1 }

    func tintHex(_ scheme: ColorScheme) -> String { customTintHex ?? Self.defaultTint(scheme) }

    func tint(_ scheme: ColorScheme) -> Color? {
        tintStrength > 0 && tintEndHex.isEmpty ? ColorHex.color(tintHex(scheme)).opacity(Double(tintStrength) / 100 * lightScale(scheme)) : nil
    }

    /// The tint as a gradient laid over the glass (Glass.tint takes a single colour only); nil unless a gradient is on.
    func gradientTint(_ scheme: ColorScheme) -> AnyShapeStyle? {
        tintStrength > 0 && !tintEndHex.isEmpty
            ? ColorHex.style(tintHex(scheme), tintEndHex, opacity: Double(tintStrength) / 100) : nil
    }

    /// Applies the saved theme; call once at launch (the setter only runs on changes).
    func applyTheme() { NSApp.appearance = theme.appearance }

    func reset() {
        theme = .system
        style = .regular
        customTintHex = nil
        tintEndHex = ""
        tintStrength = Self.defaultStrength
    }
}

private struct GlassSurface<S: Shape>: ViewModifier {
    let shape: S
    @ObservedObject private var appearance = GlassAppearance.shared
    @Environment(\.colorScheme) private var scheme

    init(shape: S) { self.shape = shape }

    @ViewBuilder private var gradientLayer: some View {
        if let gradient = appearance.gradientTint(scheme) {
            shape.fill(gradient).allowsHitTesting(false)
        } else if scheme == .light {
            // Light glass has no dark edges to read as glass, so a soft sheen from the top-left corner stands in for them.
            shape.fill(LinearGradient(
                colors: [.white.opacity(0.5), .white.opacity(0.05), .white.opacity(0.22)],
                startPoint: .topLeading, endPoint: .bottomTrailing)).allowsHitTesting(false)
        }
    }

    @ViewBuilder func body(content: Content) -> some View {
        if #available(macOS 26.0, *) {
            let glass: Glass = appearance.style == .clear ? .clear : .regular
            content
                .glassEffect(glass.tint(appearance.tint(scheme)), in: shape)
                .overlay { gradientLayer }
        } else {
            content
                .background(.ultraThinMaterial, in: shape)
                .background(appearance.tint(scheme) ?? .clear, in: shape)
                .overlay { gradientLayer }
        }
    }
}

extension View {
    /// Liquid Glass on macOS 26+ (style and tint from `GlassAppearance`), ultra-thin material on macOS 13–25.
    func glassSurface(cornerRadius: CGFloat) -> some View {
        modifier(GlassSurface(shape: RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)))
    }

    /// Glass behind a whole window: the window itself must be non-opaque (see `SettingsWindowController`).
    func glassBackdrop() -> some View {
        background { Color.clear.modifier(GlassSurface(shape: Rectangle())).ignoresSafeArea() }
    }
}

extension View {
    /// Hides the blue keyboard-focus ring that otherwise lands on the first row/button when a window opens (macOS 14+).
    @ViewBuilder func noFocusRing() -> some View {
        if #available(macOS 14.0, *) { focusEffectDisabled() } else { self }
    }
}
