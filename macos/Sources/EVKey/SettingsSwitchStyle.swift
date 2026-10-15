import SwiftUI

/// Switch thương hiệu: bật = `brand-fill`, tắt = `switch-off`. Cỡ thường 36×21, cỡ lớn 50×30 (hero).
/// Núm trắng giãn ngang khi đang bấm rồi nảy sang vị trí mới.
struct SettingsSwitchStyle: ToggleStyle {
    var large = false

    func makeBody(configuration: Configuration) -> some View {
        Button {
            configuration.isOn.toggle()
        } label: {
            EmptyView()
        }
        .pointerCursor()
        .buttonStyle(Track(isOn: configuration.isOn, large: large))
    }

    private struct Track: ButtonStyle {
        let isOn: Bool
        let large: Bool
        @Environment(\.colorScheme) private var scheme

        func makeBody(configuration: Configuration) -> some View {
            let width: CGFloat = large ? 50 : 36
            let height: CGFloat = large ? 30 : 21
            let inset: CGFloat = 2
            let knob = height - inset * 2
            let knobWidth = configuration.isPressed ? knob * 1.35 : knob
            let travel = width - inset * 2 - knobWidth
            ZStack(alignment: .leading) {
                Capsule().fill(Tokens(scheme).switchOff)
                BrandFill().clipShape(Capsule()).opacity(isOn ? 1 : 0)
                Capsule()
                    .fill(.white)
                    .shadow(color: .black.opacity(0.25), radius: 1.5, y: 1)
                    .frame(width: knobWidth, height: knob)
                    .offset(x: inset + (isOn ? travel : 0))
            }
            .frame(width: width, height: height)
            .motion(.spring(response: 0.34, dampingFraction: 0.62), value: isOn)
            .motion(.easeOut(duration: 0.14), value: configuration.isPressed)
        }
    }
}
