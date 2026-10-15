import SwiftUI

/// Nhãn, slider thương hiệu và nhãn phần trăm bên phải.
struct SettingsPercentRow: View {
    let title: String
    @Binding var value: Int

    var body: some View {
        HStack(spacing: 12) {
            Text(title)
                .font(.system(size: 13, weight: .semibold))
                .frame(width: 130, alignment: .leading)
            SettingsSlider(value: $value)
            Text("\(value)%")
                .font(.system(size: 12).monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(width: 40, alignment: .trailing)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }
}
