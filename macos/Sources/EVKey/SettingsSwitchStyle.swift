import SwiftUI

/// Switch with a flat track and a dimmed opaque thumb, without the glass highlight or white rim of the system style.
/// The on colour is the accent colour, as with the system switch. The thumb stretches while pressed and springs over.
struct SettingsSwitchStyle: ToggleStyle {
    func makeBody(configuration: Configuration) -> some View {
        Button {
            configuration.isOn.toggle()
        } label: {
            EmptyView()
        }
            .pointerCursor()
        .buttonStyle(Track(isOn: configuration.isOn))
    }

    private struct Track: ButtonStyle {
        let isOn: Bool
        @Environment(\.colorScheme) private var scheme

        func makeBody(configuration: Configuration) -> some View {
            let thumbWidth: CGFloat = configuration.isPressed ? 22 : 16
            // Offset (not alignment) so the spring drives the thumb across; pressed stretches it toward the travel direction.
            let travel: CGFloat = 34 - 4 - thumbWidth
            ZStack(alignment: .leading) {
                Capsule()
                    .fill(isOn ? Color.accentColor : Color.primary.opacity(scheme == .dark ? 0.22 : 0.32))
                Capsule()
                    // Light mode: a bright thumb, else it vanishes against the grey off track.
                    .fill(Color(white: scheme == .dark ? (isOn ? 0.74 : 0.52) : (isOn ? 0.96 : 0.99)))
                    .frame(width: thumbWidth, height: 16)
                    .offset(x: 2 + (isOn ? travel : 0))
            }
            .frame(width: 34, height: 20)
            .animation(.spring(response: 0.38, dampingFraction: 0.55), value: isOn)
            .animation(.easeOut(duration: 0.14), value: configuration.isPressed)
        }
    }
}
