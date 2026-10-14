import EVKeyKit
import SwiftUI

/// Label on the left (bold title, secondary subtitle) with a switch on the right. Draws no block, so several can share one `SettingsCard`.
struct SettingsToggleLine: View {
    let title: String
    var subtitle: String? = nil
    var beta = false
    @Binding var isOn: Bool

    var body: some View {
        HStack(spacing: 12) {
            SettingsRowLabel(title: title, subtitle: subtitle, beta: beta)
            Spacer(minLength: 12)
            Toggle("", isOn: $isOn)
                .labelsHidden()
                .toggleStyle(SettingsSwitchStyle())
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
    }
}

/// A `SettingsToggleLine` as its own block.
struct SettingsToggleRow: View {
    let title: String
    var subtitle: String? = nil
    @Binding var isOn: Bool

    var body: some View {
        SettingsToggleLine(title: title, subtitle: subtitle, isOn: $isOn)
            .settingsBlock()
    }
}

/// Label on the left with any control (such as a button) on the right, as its own block.
struct SettingsActionRow<Trailing: View>: View {
    let title: String
    var subtitle: String? = nil
    @ViewBuilder let trailing: () -> Trailing

    var body: some View {
        HStack(spacing: 12) {
            SettingsRowLabel(title: title, subtitle: subtitle)
            Spacer(minLength: 12)
            trailing()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }
}

/// Title and subtitle above a segmented control that spans the row.
struct SettingsChoiceRow<Value: Hashable>: View {
    let title: String
    var subtitle: String? = nil
    let choices: [ChoiceOption<Value>]
    @Binding var selection: Value
    /// UIStrings key per option; the active one is shown under the control, as on Windows.
    var hints: [Value: String] = [:]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsRowLabel(title: title, subtitle: subtitle)
            SettingsSegmented(choices: choices, selection: $selection)
            if let key = hints[selection] {
                Text(UIStrings.tr(key))
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }
}

/// Bold title with an optional secondary line.
struct SettingsRowLabel: View {
    let title: String
    var subtitle: String? = nil
    /// Shows a "Beta" capsule after the title.
    var beta = false

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 6) {
                Text(title)
                    .font(.system(size: 13, weight: .semibold))
                if beta {
                    Text("Beta")
                        .font(.system(size: 9, weight: .semibold))
                        .padding(.horizontal, 5)
                        .padding(.vertical, 1)
                        .background(Color.accentColor.opacity(0.18), in: Capsule())
                }
            }
            if let subtitle {
                Text(subtitle)
                    .font(.system(size: 11))
                    .foregroundStyle(Color.primary.opacity(0.65))
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }
}

/// Title with a subtitle on the left and a drop-down menu on the right, for choices that are too long for a segmented bar.
struct SettingsMenuRow<Value: Hashable>: View {
    let title: String
    var subtitle: String? = nil
    let choices: [ChoiceOption<Value>]
    @Binding var selection: Value

    var body: some View {
        HStack(spacing: 12) {
            SettingsRowLabel(title: title, subtitle: subtitle)
            Spacer(minLength: 12)
            Picker("", selection: $selection) {
                ForEach(choices) { Text($0.label).tag($0.value) }
            }
                .pointerCursor()
            .pickerStyle(.menu)
            .labelsHidden()
            .fixedSize()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }
}
