import EVKeyKit
import SwiftUI

/// Banner appearance: sliders, colours and a live preview of the real banner view.
struct SettingsBannerSection: View {
    @ObservedObject private var state = AppState.shared

    var body: some View {
        LazyVStack(alignment: .leading, spacing: 10) {
            SettingsCard { SettingsPercentRow(title: UIStrings.tr("banner.opacity"), value: state.binding(\.bannerOpacity)) }
            SettingsCard { SettingsPercentRow(title: UIStrings.tr("banner.blur"), value: state.binding(\.bannerBlur)) }
            colorRow(UIStrings.tr("banner.background"), \.bannerBg, \.bannerBgEnd)
            colorRow(UIStrings.tr("banner.border"), \.bannerBorder, \.bannerBorderEnd)
            colorRow(UIStrings.tr("banner.text"), \.bannerText, \.bannerTextEnd)

            VStack(alignment: .leading, spacing: 10) {
                Text(UIStrings.tr("banner.preview"))
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(.secondary)
                BannerView(settings: state.settings, appName: "Terminal")
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                    .background(Color.primary.opacity(0.05), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            }
            .padding(14)
            .settingsBlock()
        }
    }

    private func colorRow(
        _ title: String, _ start: WritableKeyPath<EVSettings, String>, _ end: WritableKeyPath<EVSettings, String>
    ) -> some View {
        SettingsCard { SettingsColorRow(
            title: title,
            start: Binding(
                get: { state.settings[keyPath: start] },
                set: { value in state.updateSettings { $0[keyPath: start] = value } }),
            end: Binding(
                get: { state.settings[keyPath: end] },
                set: { value in state.updateSettings { $0[keyPath: end] = value } })) }
    }

    /// Puts every banner setting back to its default.
    static func reset() {
        let defaults = EVSettings.defaults
        AppState.shared.updateSettings { settings in
            settings.bannerOpacity = defaults.bannerOpacity
            settings.bannerBlur = defaults.bannerBlur
            settings.bannerBg = defaults.bannerBg
            settings.bannerBorder = defaults.bannerBorder
            settings.bannerText = defaults.bannerText
            settings.bannerBgEnd = defaults.bannerBgEnd
            settings.bannerBorderEnd = defaults.bannerBorderEnd
            settings.bannerTextEnd = defaults.bannerTextEnd
        }
    }
}
