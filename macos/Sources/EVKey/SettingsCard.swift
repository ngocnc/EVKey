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

/// Dark mode: a soft light fill. Light mode: a milky white fill with a hairline edge, so blocks stand out from the glass.
private struct SettingsBlockStyle: ViewModifier {
    @Environment(\.colorScheme) private var scheme
    @ObservedObject private var appearance = GlassAppearance.shared

    func body(content: Content) -> some View {
        // The fill follows the tint strength: a stronger tint means more opaque cards, 0 leaves them nearly clear.
        let level = Double(appearance.tintStrength) / 100
        let shape = RoundedRectangle(cornerRadius: 12, style: .continuous)
        content
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(scheme == .dark ? Color.primary.opacity(0.06 + 0.2 * level) : Color.white.opacity(0.15 + 0.8 * level), in: shape)
            .overlay { if scheme == .light { shape.strokeBorder(Color.black.opacity(0.08), lineWidth: 0.5) } }
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
