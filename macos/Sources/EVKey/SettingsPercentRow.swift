import SwiftUI

/// Label, slider and an editable percentage: drag the slider or type the number.
struct SettingsPercentRow: View {
    let title: String
    @Binding var value: Int

    var body: some View {
        HStack(spacing: 12) {
            Text(title)
                .font(.system(size: 13, weight: .semibold))
                .frame(width: 130, alignment: .leading)
            Slider(value: Binding(get: { Double(value) }, set: { value = Int($0.rounded()) }), in: 0...100)
            HStack(spacing: 2) {
                TextField("", value: Binding(get: { value }, set: { value = min(max($0, 0), 100) }), format: .number)
                    .textFieldStyle(.plain)
                    .multilineTextAlignment(.trailing)
                    .frame(width: 34)
                Text("%").foregroundStyle(.secondary)
            }
            .font(.system(size: 12).monospacedDigit())
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(Color.primary.opacity(0.08), in: RoundedRectangle(cornerRadius: 6, style: .continuous))
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }
}
