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

    static let defaultTint = "#000000"
    /// A light dark tint by default: untinted glass lets whatever is behind the window show through and hurts legibility.
    static let defaultStrength = 30

    @Published var style: Style { didSet { defaults.set(style.rawValue, forKey: "glass.style") } }
    /// "#rrggbb" tint laid over the glass.
    @Published var tintHex: String { didSet { defaults.set(tintHex, forKey: "glass.tint") } }
    /// "#rrggbb" end colour of a gradient tint; empty keeps the tint a single colour.
    @Published var tintEndHex: String { didSet { defaults.set(tintEndHex, forKey: "glass.tintEnd") } }
    /// 0-100: how strongly the tint covers the glass; 0 leaves the glass untinted.
    @Published var tintStrength: Int { didSet { defaults.set(tintStrength, forKey: "glass.strength") } }

    private let defaults = UserDefaults.standard

    private init() {
        style = Style(rawValue: defaults.string(forKey: "glass.style") ?? "") ?? .regular
        tintHex = defaults.string(forKey: "glass.tint") ?? Self.defaultTint
        tintEndHex = defaults.string(forKey: "glass.tintEnd") ?? ""
        tintStrength = defaults.object(forKey: "glass.strength") as? Int ?? Self.defaultStrength
    }

    var tint: Color? {
        tintStrength > 0 && tintEndHex.isEmpty ? ColorHex.color(tintHex).opacity(Double(tintStrength) / 100) : nil
    }

    /// The tint as a gradient laid over the glass (Glass.tint takes a single colour only); nil unless a gradient is on.
    var gradientTint: AnyShapeStyle? {
        tintStrength > 0 && !tintEndHex.isEmpty
            ? ColorHex.style(tintHex, tintEndHex, opacity: Double(tintStrength) / 100) : nil
    }

    func reset() {
        style = .regular
        tintHex = Self.defaultTint
        tintEndHex = ""
        tintStrength = Self.defaultStrength
    }
}

private struct GlassSurface<S: Shape>: ViewModifier {
    let shape: S
    @ObservedObject private var appearance = GlassAppearance.shared

    init(shape: S) { self.shape = shape }

    @ViewBuilder private var gradientLayer: some View {
        if let gradient = appearance.gradientTint {
            shape.fill(gradient).allowsHitTesting(false)
        }
    }

    @ViewBuilder func body(content: Content) -> some View {
        if #available(macOS 26.0, *) {
            let glass: Glass = appearance.style == .clear ? .clear : .regular
            content
                .glassEffect(glass.tint(appearance.tint), in: shape)
                .overlay { gradientLayer }
        } else {
            content
                .background(.ultraThinMaterial, in: shape)
                .background(appearance.tint ?? .clear, in: shape)
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
