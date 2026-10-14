import EVKeyKit
import SwiftUI

/// Bảng điều khiển: sidebar of tabs, notices for missing permission or a broken config, and the selected tab.
struct SettingsView: View {
    @ObservedObject var navigation: SettingsNavigation
    @ObservedObject private var state = AppState.shared

    var body: some View {
        HStack(spacing: 0) {
            sidebar
                .frame(width: 200)
            Divider()
            VStack(alignment: .leading, spacing: 0) {
                notices
                ScrollView {
                    detail
                        .id(navigation.tab)
                        // Only the new tab fades in; fading the old one out too would draw two heavy tabs at once.
                        .transition(.asymmetric(insertion: .opacity, removal: .identity))
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
                .animation(.easeOut(duration: 0.18), value: navigation.tab)
            }
            .animation(.easeInOut(duration: 0.25), value: state.accessibilityTrusted)
            .animation(.easeInOut(duration: 0.25), value: state.configError)
        }
        .frame(minWidth: 760, minHeight: 540)
        // A thin material between the glass and the content keeps titles and captions legible over busy backgrounds.
        .background(.thinMaterial)
        .glassBackdrop()
        .overlay(alignment: .topTrailing) {
            SettingsPinButton()
                .padding(.trailing, 12)
                .ignoresSafeArea(edges: .top)
        }
        .noFocusRing()
    }

    /// Height of the titlebar zone (traffic lights and title) the scrolled content must not show through; the
    /// notices strip already sits below it.
    private var titlebarFade: CGFloat {
        state.accessibilityTrusted && state.configError == nil ? 36 : 0
    }

    private var sidebar: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 10) {
                LogoMark(letter: state.isEnglish ? "E" : "V", size: 34)
                VStack(alignment: .leading, spacing: 0) {
                    Text("EVKey").font(.system(size: 13, weight: .semibold))
                    Text("v\(state.version)").font(.system(size: 11)).foregroundStyle(.secondary)
                }
            }
            .padding(.top, 34)
            .padding(.horizontal, 12)
            .padding(.bottom, 12)

            ForEach(SettingsTab.allCases) { tab in
                Button {
                    navigation.tab = tab
                } label: {
                    HStack(spacing: 8) {
                        Image(systemName: tab.systemImage).frame(width: 20)
                        Text(tab.title)
                        if tab == .translate {
                            Text("Beta")
                                .font(.system(size: 9, weight: .semibold))
                                .padding(.horizontal, 5)
                                .padding(.vertical, 1)
                                .background(Color.accentColor.opacity(0.18), in: Capsule())
                        }
                    }
                    .font(.system(size: 13))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                        .background(
                            navigation.tab == tab ? Color.accentColor.opacity(0.18) : Color.clear,
                            in: RoundedRectangle(cornerRadius: 7, style: .continuous)
                        )
                        .contentShape(Rectangle())
                        .animation(.easeInOut(duration: 0.2), value: navigation.tab)
                }
                .buttonStyle(.plain)
                .padding(.horizontal, 8)
            }
            Spacer()
        }
        .padding(.bottom, 12)
        .frame(maxHeight: .infinity, alignment: .top)
    }

    @ViewBuilder
    private var notices: some View {
        if !state.accessibilityTrusted {
            SettingsNotice(
                text: "EVKey cần quyền Trợ năng để gõ tiếng Việt. Bật EVKey trong Cài đặt hệ thống > Quyền riêng tư & Bảo mật > Trợ năng.",
                buttonTitle: "Mở cài đặt Trợ năng"
            ) { state.openAccessibilitySettings() }
        }
        if let error = state.configError {
            SettingsNotice(text: error, buttonTitle: "Đặt lại về mặc định") {
                if confirmResetConfig() { state.perform { try state.resetConfig() } }
            }
        }
    }

    @ViewBuilder
    private var detail: some View {
        switch navigation.tab {
        case .typing: SettingsTypingTab()
        case .keys: SettingsKeysTab()
        case .smart: SettingsSmartTab()
        case .macro: SettingsMacroTab()
        case .apps: SettingsAppsTab()
        case .appearance: SettingsAppearanceTab()
        case .system: SettingsSystemTab()
        case .translate: SettingsTranslateTab()
        case .info: SettingsInfoTab()
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
        }
        .padding(12)
        .background(Color.orange.opacity(0.12), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .padding([.horizontal, .top], 16)
        .transition(.move(edge: .top).combined(with: .opacity))
    }
}
