import EVKeyKit
import SwiftUI

/// Liquid Glass look of the popup and Bảng điều khiển. `advanced` false shows the theme; true shows style, tint colour
/// and tint strength. Applies live.
struct SettingsGlassSection: View {
    var advanced = false
    @ObservedObject private var glass = GlassAppearance.shared
    @ObservedObject private var state = AppState.shared
    @Environment(\.colorScheme) private var scheme

    private var tintBinding: Binding<String> {
        Binding(get: { glass.tintHex(scheme) }, set: { glass.customTintHex = $0 })
    }

    var body: some View {
        LazyVStack(alignment: .leading, spacing: 10) {
            if !advanced {
                VStack(alignment: .leading, spacing: 8) {
                    Text(UIStrings.tr("glass.theme"))
                        .font(.system(size: 13, weight: .semibold))
                    SettingsSegmented(
                        choices: GlassAppearance.Theme.allCases.map { ChoiceOption(value: $0, label: $0.label) },
                        selection: $glass.theme)
                    Text(UIStrings.tr("hint.local-theme.\(glass.theme.rawValue)"))
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
                .padding(14)
                .settingsBlock()
            }

            if advanced {
                VStack(alignment: .leading, spacing: 8) {
                    Text(UIStrings.tr("glass.style"))
                        .font(.system(size: 13, weight: .semibold))
                    SettingsSegmented(
                        choices: GlassAppearance.Style.allCases.map { ChoiceOption(value: $0, label: $0.label) },
                        selection: $glass.style)
                    Text(UIStrings.tr("hint.local-glass-style.\(glass.style.rawValue)"))
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                    if #unavailable(macOS 26.0) {
                        Text(UIStrings.tr("glass.needsMacOS26"))
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(14)
                .settingsBlock()

                SettingsCard {
                    SettingsPercentRow(title: UIStrings.tr("glass.tintStrength"), value: $glass.tintStrength)
                    SettingsDivider()
                    SettingsColorRow(title: UIStrings.tr("glass.tint"), start: tintBinding, end: $glass.tintEndHex)
                }
            }
        }
    }
}
