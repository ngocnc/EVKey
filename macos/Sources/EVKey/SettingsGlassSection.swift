import SwiftUI

/// Liquid Glass look of the popup and Bảng điều khiển: style, tint colour and tint strength. Applies live.
struct SettingsGlassSection: View {
    @ObservedObject private var glass = GlassAppearance.shared
    @Environment(\.colorScheme) private var scheme

    private var tintBinding: Binding<String> {
        Binding(get: { glass.tintHex(scheme) }, set: { glass.customTintHex = $0 })
    }

    var body: some View {
        LazyVStack(alignment: .leading, spacing: 10) {
            VStack(alignment: .leading, spacing: 8) {
                Text("Chủ đề")
                    .font(.system(size: 13, weight: .semibold))
                SettingsSegmented(
                    choices: GlassAppearance.Theme.allCases.map { ChoiceOption(value: $0, label: $0.label) },
                    selection: $glass.theme)
            }
            .padding(14)
            .settingsBlock()

            VStack(alignment: .leading, spacing: 8) {
                Text("Kiểu kính")
                    .font(.system(size: 13, weight: .semibold))
                SettingsSegmented(
                    choices: GlassAppearance.Style.allCases.map { ChoiceOption(value: $0, label: $0.label) },
                    selection: $glass.style)
                if #unavailable(macOS 26.0) {
                    Text("Liquid Glass cần macOS 26 trở lên; bản cũ dùng nền mờ.")
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                }
            }
            .padding(14)
            .settingsBlock()

            SettingsCard {
                SettingsPercentRow(title: "Độ đậm màu phủ", value: $glass.tintStrength)
                SettingsDivider()
                SettingsColorRow(title: "Màu phủ", start: tintBinding, end: $glass.tintEndHex)
            }
        }
    }
}
