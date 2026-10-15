import SwiftUI

/// One settings tab: large title, optional note, then the sections.
struct SettingsPage<Content: View>: View {
    let title: String
    var note: String? = nil
    @ViewBuilder let content: () -> Content

    var body: some View {
        // Lazy: blocks below the fold are built only when scrolled to, which keeps switching to long tabs cheap.
        LazyVStack(alignment: .leading, spacing: 10) {
            Text(title)
                .font(.system(size: 22, weight: .bold))
            if let note {
                Text(note)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            content()
        }
    }
}

/// Small heading above a card.
struct SettingsHeading: View {
    let text: String

    init(_ text: String) { self.text = text }

    var body: some View {
        Text(text)
            .font(.system(size: 13, weight: .semibold))
            .foregroundStyle(.secondary)
            .padding(.top, 2)
    }
}

extension View {
    /// One separate block: a soft fill, so it reads clearly against the glass behind it.
    func settingsBlock() -> some View {
        modifier(SettingsBlockStyle())
    }
}

/// Thẻ: chất liệu Thường là nền đặc có viền mảnh; Liquid Glass là nền trong kèm viền khúc xạ và vệt sáng.
private struct SettingsBlockStyle: ViewModifier {
    @Environment(\.colorScheme) private var scheme
    @ObservedObject private var appearance = GlassAppearance.shared

    func body(content: Content) -> some View {
        let tokens = Tokens(scheme)
        let shape = RoundedRectangle(cornerRadius: 12, style: .continuous)
        let framed = content.frame(maxWidth: .infinity, alignment: .leading)
        if appearance.material == .glass {
            framed
                .background(tokens.lgCard, in: shape)
                .liquidGlassEdge(cornerRadius: 12)
        } else {
            framed
                .background(tokens.card, in: shape)
                .overlay { shape.strokeBorder(tokens.cardBorder, lineWidth: 0.5) }
        }
    }
}

/// A single block holding whatever the caller puts in it; the content draws its own padding.
struct SettingsCard<Content: View>: View {
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(spacing: 0) {
            content()
        }
        .settingsBlock()
    }
}

/// Hairline between rows that share one `SettingsCard`.
struct SettingsDivider: View {
    var body: some View {
        Divider().padding(.horizontal, 14)
    }
}

/// Stacks blocks with a gap between them; each child (a row, or a view that calls `settingsBlock()`) is its own block.
struct SettingsGroup<Content: View>: View {
    @ViewBuilder let content: () -> Content

    var body: some View {
        LazyVStack(spacing: 6) {
            content()
        }
    }
}
