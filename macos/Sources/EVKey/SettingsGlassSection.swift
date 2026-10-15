import EVKeyKit
import SwiftUI

/// Giao diện cục bộ. `advanced` false: chủ đề. `advanced` true: màu nhấn, chất liệu và độ trong suốt (chỉ khi Liquid Glass).
/// Áp dụng ngay.
struct SettingsGlassSection: View {
    var advanced = false
    @ObservedObject private var glass = GlassAppearance.shared

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
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    Text(UIStrings.tr("glass.accent"))
                        .font(.system(size: 13, weight: .semibold))
                    AccentSwatches()
                }
                .padding(14)
                .settingsBlock()

                VStack(alignment: .leading, spacing: 8) {
                    Text(UIStrings.tr("glass.material"))
                        .font(.system(size: 13, weight: .semibold))
                    SettingsSegmented(
                        choices: SurfaceMaterial.allCases.map { ChoiceOption(value: $0, label: $0.label) },
                        selection: $glass.material)
                    if #unavailable(macOS 26.0) {
                        Text(UIStrings.tr("glass.needsMacOS26"))
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(14)
                .settingsBlock()

                RevealGroup(isShown: glass.material == .glass) {
                    SettingsCard {
                        SettingsPercentRow(title: UIStrings.tr("glass.transparency"), value: $glass.transparency)
                    }
                }
            }
        }
    }
}
