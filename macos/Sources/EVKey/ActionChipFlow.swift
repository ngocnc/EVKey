import EVKeyKit
import SwiftUI

/// Các chip của một nhóm hành động (Viết lại, Chuyển mã, Chữ). Xuống dòng khi không đủ chỗ (macOS 13 chưa có `Layout`
/// flow sẵn nên dùng ViewThatFits: một hàng, hoặc chia đôi).
struct ActionChipFlow: View {
    let chips: [(String, PanelAction)]
    let run: (PanelAction) -> Void

    var body: some View {
        ViewThatFits(in: .horizontal) {
            row(chips)
            VStack(alignment: .leading, spacing: 6) {
                row(Array(chips.prefix((chips.count + 1) / 2)))
                row(Array(chips.dropFirst((chips.count + 1) / 2)))
            }
        }
    }

    private func row(_ items: [(String, PanelAction)]) -> some View {
        HStack(spacing: 6) {
            ForEach(items, id: \.0) { chip in
                ActionChip(title: UIStrings.tr(chip.0)) { run(chip.1) }
            }
        }
    }
}

/// Chip viên thuốc: nền thẻ, rê chuột thì `brand-fill` chữ trắng, bấm thì lún.
private struct ActionChip: View {
    let title: String
    let action: () -> Void
    @State private var hovered = false
    @Environment(\.colorScheme) private var scheme
    @ObservedObject private var appearance = GlassAppearance.shared

    var body: some View {
        let tokens = Tokens(scheme)
        Button(action: action) {
            Text(title)
                .font(.system(size: 12))
                .foregroundStyle(hovered ? Color.white : tokens.fg)
                .fixedSize()
                .padding(.horizontal, 11)
                .padding(.vertical, 4)
                .background {
                    if hovered {
                        BrandFill().clipShape(Capsule()).shadow(color: appearance.accent.shadow, radius: 4, y: 2)
                    } else {
                        Capsule().fill(tokens.card)
                        Capsule().strokeBorder(tokens.cardBorder, lineWidth: 0.5)
                    }
                }
                .contentShape(Capsule())
        }
        .buttonStyle(PressButtonStyle(chrome: false))
        .onHover { hovered = $0 }
        .motion(.easeOut(duration: 0.12), value: hovered)
    }
}
