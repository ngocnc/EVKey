import SwiftUI

/// Hàng ô chọn màu nhấn: mỗi ô là gradient của màu đó, ô đang chọn có vòng viền.
struct AccentSwatches: View {
    @ObservedObject private var appearance = GlassAppearance.shared
    @Environment(\.colorScheme) private var scheme

    var body: some View {
        HStack(spacing: 10) {
            ForEach(Accent.allCases) { accent in
                let selected = appearance.accent == accent
                BrandFill(accent: accent)
                    .clipShape(Circle())
                    .frame(width: 26, height: 26)
                    .padding(3)
                    .overlay { Circle().strokeBorder(Tokens(scheme).fg, lineWidth: selected ? 2 : 0) }
                    .scaleEffect(selected ? 1.08 : 1)
                    .contentShape(Circle())
                    .pointerCursor()
                    .help(accent.label)
                    .accessibilityLabel(accent.label)
                    .onTapGesture { appearance.accent = accent }
            }
        }
        .motion(Motion.slide(0.24), value: appearance.accent)
    }
}
