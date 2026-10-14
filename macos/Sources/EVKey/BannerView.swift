import EVKeyKit
import SwiftUI

/// The Secure Input banner. The settings preview renders this same view with the live appearance.
struct BannerView: View {
    let settings: EVSettings
    let appName: String

    var body: some View {
        let border = ColorHex.style(settings.bannerBorder, settings.bannerBorderEnd)
        let text = ColorHex.style(settings.bannerText, settings.bannerTextEnd)
        HStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 22, weight: .semibold))
                .foregroundStyle(border)
            VStack(alignment: .leading, spacing: 2) {
                Text("Secure Input đang bật")
                    .font(.system(size: 14, weight: .bold))
                Text("\(appName) đang chặn EVKey xử lý Tiếng Việt")
                    .font(.system(size: 12))
                    .opacity(0.75)
            }
            .foregroundStyle(text)
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
        .fixedSize()
        .background { surface }
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(border, lineWidth: 1))
    }

    /// Liquid Glass tinted with the background colour on macOS 26+; tinted material with a blur layer before that.
    @ViewBuilder private var surface: some View {
        let shape = RoundedRectangle(cornerRadius: 14, style: .continuous)
        let tint = ColorHex.style(settings.bannerBg, settings.bannerBgEnd, opacity: Double(settings.bannerOpacity) / 100)
        if #available(macOS 26.0, *) {
            // Glass itself has no blur setting, so the clear style provides the edge and refraction and a material
            // layer scaled by `bannerBlur` provides the adjustable blur.
            ZStack {
                Color.clear.glassEffect(.clear, in: shape)
                shape.fill(.ultraThinMaterial).opacity(Double(settings.bannerBlur) / 100)
                shape.fill(tint)
            }
        } else {
            ZStack {
                shape.fill(.ultraThinMaterial).opacity(Double(settings.bannerBlur) / 100)
                shape.fill(tint)
            }
        }
    }
}

/// Panel content: reads the live state so text and appearance update without rebuilding the panel.
struct BannerHost: View {
    @ObservedObject private var state = AppState.shared
    /// Last holder shown. While the banner fades out `secureHolder` is already nil; without this the text would flip to
    /// the generic wording mid-fade and the banner would grow wider than its panel.
    @State private var lastHolder = ""

    var body: some View {
        let holder = state.secureHolder ?? (lastHolder.isEmpty ? DebugOptions.bannerApp : lastHolder) ?? ""
        BannerView(settings: state.settings, appName: holder.isEmpty ? "Một ứng dụng" : holder)
            .onReceive(state.$secureHolder) { if let h = $0 { lastHolder = h } }
    }
}
