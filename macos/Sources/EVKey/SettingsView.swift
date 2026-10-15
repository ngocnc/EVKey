import EVKeyKit
import SwiftUI

/// Bảng điều khiển: sidebar of tabs, notices for missing permission or a broken config, and the selected tab.
struct SettingsView: View {
    @ObservedObject var navigation: SettingsNavigation
    @ObservedObject private var state = AppState.shared
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    /// True on the very first open; closing the welcome card records it in UserDefaults.
    @State private var showWelcome = !WelcomeView.seen
    /// +1 when the last tab change moved down the sidebar, -1 when it moved up; the new page slides in from that side.
    @State private var direction: CGFloat = 1

    var body: some View {
        HStack(spacing: 0) {
            SettingsSidebar(navigation: navigation, onSelect: select)
                .frame(width: 200)
            Divider()
            VStack(alignment: .leading, spacing: 0) {
                notices
                ScrollView {
                    detail
                        .id(navigation.tab)
                        .transition(pageTransition)
                        .padding(24)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                // Scroll views draw under the transparent titlebar; fade the content out before it reaches the title.
                .mask(alignment: .top) {
                    VStack(spacing: 0) {
                        LinearGradient(stops: [.init(color: .clear, location: 0.7), .init(color: .black, location: 1)],
                                       startPoint: .top, endPoint: .bottom)
                            .frame(height: titlebarFade)
                        Color.black
                    }
                }
                .motion(Motion.slide(), value: navigation.tab)
            }
            .animation(.easeInOut(duration: 0.25), value: state.accessibilityTrusted)
            .animation(.easeInOut(duration: 0.25), value: state.configError)
        }
        .frame(minWidth: 760, minHeight: 540)
        .glassBackdrop()
        .overlay(alignment: .topTrailing) {
            SettingsPinButton()
                .padding(.trailing, 12)
                .ignoresSafeArea(edges: .top)
        }
        .overlay {
            if showWelcome {
                WelcomeView(onClose: closeWelcome)
                    .transition(.opacity)
            }
        }
        .noFocusRing()
    }

    /// Đổi tab và ghi nhớ hướng để trang mới trượt từ phía mục vừa bấm.
    private func select(_ tab: SettingsTab) {
        guard tab != navigation.tab else { return }
        let tabs = SettingsTab.allCases
        let from = tabs.firstIndex(of: navigation.tab) ?? 0
        let to = tabs.firstIndex(of: tab) ?? 0
        direction = to > from ? 1 : -1
        navigation.tab = tab
    }

    /// Trang mới trượt 28pt từ phía mục được bấm kèm mờ dần; trang cũ chỉ mờ đi để không chồng hai trang. Reduce Motion: không hiệu ứng.
    private var pageTransition: AnyTransition {
        reduceMotion
            ? .identity
            : .asymmetric(insertion: .offset(x: 28 * direction).combined(with: .opacity), removal: .opacity)
    }

    private func closeWelcome() {
        WelcomeView.seen = true
        withAnimation(.easeOut(duration: 0.2)) { showWelcome = false }
    }

    /// Height of the titlebar zone (traffic lights and title) the scrolled content must not show through; the
    /// notices strip already sits below it.
    private var titlebarFade: CGFloat {
        state.accessibilityTrusted && state.configError == nil ? 36 : 0
    }

    @ViewBuilder
    private var notices: some View {
        if !state.accessibilityTrusted {
            SettingsNotice(
                text: UIStrings.tr("settings.accessNotice"),
                buttonTitle: UIStrings.tr("settings.openAccessibility")
            ) { state.openAccessibilitySettings() }
        }
        if let error = state.configError {
            SettingsNotice(text: error, buttonTitle: UIStrings.tr("settings.reset")) {
                if confirmResetConfig() { state.perform { try state.resetConfig() } }
            }
        }
    }

    @ViewBuilder
    private var detail: some View {
        switch navigation.tab {
        case .typing: SettingsTypingTab()
        case .macro: SettingsMacroTab()
        case .keys: SettingsKeysTab()
        case .apps: SettingsAppsTab()
        case .system: SettingsSystemTab()
        case .translate: SettingsTranslateTab()
        }
    }
}

/// Warning strip shown above the tabs; the button resolves the problem.
private struct SettingsNotice: View {
    let text: String
    let buttonTitle: String
    let action: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.orange)
            Text(text)
                .font(.system(size: 12))
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 8)
            Button(buttonTitle, action: action)
                .pointerCursor()
        }
        .padding(12)
        .background(Color.orange.opacity(0.12), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .padding([.horizontal, .top], 16)
        .transition(.move(edge: .top).combined(with: .opacity))
    }
}
