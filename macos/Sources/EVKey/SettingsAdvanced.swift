import EVKeyKit
import SwiftUI

/// Mục "Nâng cao" cuối mỗi tab: mặc định đóng, trạng thái nhớ theo tab (`evkey-adv-<pane>` trong UserDefaults),
/// giống cửa sổ Settings trên Windows nhớ trong localStorage. Mở/đóng bằng chiều cao có hiệu ứng.
struct SettingsAdvanced<Content: View>: View {
    @AppStorage private var isOpen: Bool
    @ViewBuilder let content: () -> Content

    init(pane: String, @ViewBuilder content: @escaping () -> Content) {
        _isOpen = AppStorage(wrappedValue: false, "evkey-adv-" + pane)
        self.content = content
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button { isOpen.toggle() } label: {
                HStack(spacing: 6) {
                    Image(systemName: "chevron.right")
                        .font(.system(size: 10, weight: .bold))
                        .rotationEffect(.degrees(isOpen ? 90 : 0))
                    Text(UIStrings.misc("advanced"))
                        .font(.system(size: 13, weight: .semibold))
                    Spacer(minLength: 0)
                }
                .foregroundStyle(.secondary)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .pointerCursor()
            .motion(Motion.slide(), value: isOpen)

            RevealGroup(isShown: isOpen) {
                LazyVStack(alignment: .leading, spacing: 10) {
                    content()
                }
                .padding(.top, 8)
            }
        }
        .padding(.top, 6)
    }
}
