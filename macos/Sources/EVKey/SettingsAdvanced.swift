import EVKeyKit
import SwiftUI

/// The collapsed "Nâng cao" section at the end of a settings tab. Closed by default; the open state is remembered per
/// tab (`evkey-adv-<pane>` in UserDefaults), like the Windows settings window remembers it in localStorage.
struct SettingsAdvanced<Content: View>: View {
    @AppStorage private var isOpen: Bool
    @ViewBuilder let content: () -> Content

    init(pane: String, @ViewBuilder content: @escaping () -> Content) {
        _isOpen = AppStorage(wrappedValue: false, "evkey-adv-" + pane)
        self.content = content
    }

    var body: some View {
        DisclosureGroup(isExpanded: $isOpen) {
            LazyVStack(alignment: .leading, spacing: 10) {
                content()
            }
            .padding(.top, 8)
        } label: {
            // macOS only toggles on the chevron; the label should open it too.
            Text(UIStrings.misc("advanced"))
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(.secondary)
                .contentShape(Rectangle())
                .onTapGesture { withAnimation { isOpen.toggle() } }
                .pointerCursor()
        }
        .padding(.top, 6)
    }
}
