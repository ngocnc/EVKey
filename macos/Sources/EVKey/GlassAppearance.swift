import AppKit
import EVKeyKit
import SwiftUI

/// Look-only preference for the Liquid Glass of the popup and Bảng điều khiển. Kept in UserDefaults, not config.toml
/// (the Windows build keeps its own glass preference in the web view's local storage).
@MainActor
final class GlassAppearance: ObservableObject {
    static let shared = GlassAppearance()

    /// Colour theme of every EVKey window: follow macOS, or force light or dark.
    enum Theme: String, CaseIterable, Identifiable {
        case system, light, dark
        var id: String { rawValue }
        var label: String {
            switch self {
            case .system: UIStrings.tr("glass.theme.system")
            case .light: UIStrings.tr("glass.theme.light")
            case .dark: UIStrings.tr("glass.theme.dark")
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

    @Published var theme: Theme {
        didSet {
            defaults.set(theme.rawValue, forKey: "glass.theme")
            NSApp.appearance = theme.appearance
        }
    }
    /// Màu nhấn của cả ứng dụng (gradient thương hiệu).
    @Published var accent: Accent { didSet { defaults.set(accent.rawValue, forKey: "glass.accent") } }
    /// Thẻ, nút và thanh chọn: đặc hoặc Liquid Glass.
    @Published var material: SurfaceMaterial { didSet { defaults.set(material.rawValue, forKey: "glass.material") } }
    /// 0-100: độ trong suốt của kính (chỉ có tác dụng khi `material == .glass`).
    @Published var transparency: Int { didSet { defaults.set(transparency, forKey: "glass.transparency") } }

    static let defaultTransparency = 60

    private let defaults = UserDefaults.standard

    private init() {
        theme = Theme(rawValue: defaults.string(forKey: "glass.theme") ?? "") ?? .system
        accent = Accent(rawValue: defaults.string(forKey: "glass.accent") ?? "") ?? .ocean
        material = SurfaceMaterial(rawValue: defaults.string(forKey: "glass.material") ?? "") ?? .glass
        transparency = defaults.object(forKey: "glass.transparency") as? Int ?? Self.defaultTransparency
    }

    /// Độ đậm lớp phủ nền kính: càng trong suốt càng nhẹ. Kính sáng không có cạnh tối nên cần lớp trắng đậm hơn,
    /// nếu không cửa sổ thành màu xám đục.
    func tintOpacity(_ scheme: ColorScheme) -> Double {
        scheme == .dark
            ? (1 - Double(transparency) / 100) * 0.6 + 0.05
            : (1 - Double(transparency) / 100) * 0.55 + 0.35
    }

    /// Applies the saved theme; call once at launch (the setter only runs on changes).
    func applyTheme() { NSApp.appearance = theme.appearance }

    func reset() {
        theme = .system
        accent = .ocean
        material = .glass
        transparency = Self.defaultTransparency
    }
}

private struct GlassSurface<S: Shape>: ViewModifier {
    let shape: S
    @ObservedObject private var appearance = GlassAppearance.shared
    @Environment(\.colorScheme) private var scheme

    init(shape: S) { self.shape = shape }

    @ViewBuilder func body(content: Content) -> some View {
        let tint = (scheme == .dark ? Color.black : Color.white).opacity(appearance.tintOpacity(scheme))
        if appearance.material == .plain {
            content.background(Color(nsColor: .windowBackgroundColor), in: shape)
        } else if #available(macOS 26.0, *) {
            content.glassEffect(.regular.tint(tint), in: shape).overlay { lightSheen }
        } else {
            content
                .background(.ultraThinMaterial, in: shape)
                .background(tint, in: shape)
                .overlay { lightSheen }
        }
    }

    /// Vệt sáng chéo từ góc trên trái cho kính sáng, thay cho cạnh tối mà kính tối có sẵn.
    @ViewBuilder private var lightSheen: some View {
        if scheme == .light {
            shape.fill(LinearGradient(
                colors: [.white.opacity(0.5), .white.opacity(0.05), .white.opacity(0.22)],
                startPoint: .topLeading, endPoint: .bottomTrailing))
                .allowsHitTesting(false)
        }
    }
}

extension View {
    /// Liquid Glass on macOS 26+ (material and transparency from `GlassAppearance`), ultra-thin material on macOS 13–25.
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
