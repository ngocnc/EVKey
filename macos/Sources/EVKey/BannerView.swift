import EVKeyKit
import SwiftUI

/// Banner Secure Input: thẻ glass bo 16, ô khoá nền `brand-fill` (không viền), tiêu đề và tên ứng dụng đang giữ.
/// Độ mờ/độ trong theo `banner_opacity`/`banner_blur`; màu theo token, không còn tuỳ chỉnh màu riêng.
struct BannerView: View {
    let settings: EVSettings
    let appName: String
    @Environment(\.colorScheme) private var scheme

    var body: some View {
        let tokens = Tokens(scheme)
        HStack(spacing: 12) {
            BrandFill()
                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                .overlay {
                    Image(systemName: "lock.fill")
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundStyle(.white)
                }
                .frame(width: 34, height: 34)
            VStack(alignment: .leading, spacing: 2) {
                Text(UIStrings.tr("banner.title"))
                    .font(.system(size: 13.5, weight: .semibold))
                    .foregroundStyle(tokens.fg)
                Text(UIStrings.tr("banner.message", appName))
                    .font(.system(size: 11.5))
                    .foregroundStyle(tokens.fgMuted)
            }
        }
        .padding(.leading, 12)
        .padding(.trailing, 18)
        .padding(.vertical, 12)
        .fixedSize()
        .background { surface }
        .liquidGlassEdge(cornerRadius: 16)
    }

    /// Liquid Glass trên macOS 26+, material trước đó; lớp nền token phủ theo `banner_opacity`.
    @ViewBuilder private var surface: some View {
        let shape = RoundedRectangle(cornerRadius: 16, style: .continuous)
        let fill = Tokens(scheme).surface.opacity(Double(settings.bannerOpacity) / 100)
        if #available(macOS 26.0, *) {
            ZStack {
                Color.clear.glassEffect(.clear, in: shape)
                shape.fill(.ultraThinMaterial).opacity(Double(settings.bannerBlur) / 100)
                shape.fill(fill)
            }
        } else {
            ZStack {
                shape.fill(.ultraThinMaterial).opacity(Double(settings.bannerBlur) / 100)
                shape.fill(fill)
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
        BannerView(settings: state.settings, appName: holder.isEmpty ? UIStrings.tr("banner.anApp") : holder)
            .onReceive(state.$secureHolder) { if let h = $0 { lastHolder = h } }
    }
}
