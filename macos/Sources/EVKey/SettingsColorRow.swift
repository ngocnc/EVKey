import SwiftUI

/// A colour setting that can optionally become a gradient: a switch turns on a second colour (the end of a diagonal
/// gradient). An empty `end` means the colour is solid.
struct SettingsColorRow: View {
    let title: String
    @Binding var start: String
    @Binding var end: String

    var body: some View {
        HStack(spacing: 10) {
            Text(title)
                .font(.system(size: 13, weight: .semibold))
            Spacer(minLength: 8)
            Text("Gradient")
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
            Toggle("", isOn: gradient)
                .labelsHidden()
                .toggleStyle(SettingsSwitchStyle())
            picker($start)
            if !end.isEmpty {
                Image(systemName: "arrow.right")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(.secondary)
                picker($end)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .animation(.easeInOut(duration: 0.2), value: end.isEmpty)
    }

    private var gradient: Binding<Bool> {
        Binding(
            get: { !end.isEmpty },
            set: { on in end = on ? ColorHex.companion(start) : "" })
    }

    private func picker(_ hex: Binding<String>) -> some View {
        ColorPicker(
            "",
            selection: Binding(
                get: { ColorHex.color(hex.wrappedValue) },
                set: { if let value = ColorHex.hex($0) { hex.wrappedValue = value } }),
            supportsOpacity: false
        )
        .labelsHidden()
    }
}
