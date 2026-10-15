import EVKeyKit
import SwiftUI

/// Sidebar chữ thuần của Bảng điều khiển: logo, 6 mục (pill gradient trượt giữa các mục), chân trang gồm phiên bản, kiểm tra cập nhật và GitHub.
struct SettingsSidebar: View {
    @ObservedObject var navigation: SettingsNavigation
    let onSelect: (SettingsTab) -> Void
    @ObservedObject private var state = AppState.shared
    @ObservedObject private var updater = AppUpdater.shared
    @Environment(\.colorScheme) private var scheme
    @Namespace private var pill

    /// Kết quả lần kiểm tra gần nhất; nil khi đang rảnh hoặc đang kiểm tra.
    private var updateStatus: String? {
        switch updater.status {
        case .idle, .checking: return nil
        case .latest: return UIStrings.tr("update.latest")
        case .failed(let message): return UIStrings.tr("update.error", message)
        }
    }

    var body: some View {
        let tokens = Tokens(scheme)
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 10) {
                LogoMark(letter: state.isEnglish ? "E" : "V", size: 34)
                Text("EVKey")
                    .font(.system(size: 15, weight: .bold))
                    .foregroundStyle(tokens.fg)
            }
            .padding(.top, 34)
            .padding(.horizontal, 16)
            .padding(.bottom, 14)

            ForEach(SettingsTab.allCases) { tab in
                item(tab, tokens: tokens)
            }

            Spacer(minLength: 12)
            footer(tokens: tokens)
        }
        .padding(.horizontal, 10)
        .padding(.bottom, 14)
        .frame(maxHeight: .infinity, alignment: .top)
        .motion(Motion.slide(), value: navigation.tab)
    }

    private func item(_ tab: SettingsTab, tokens: Tokens) -> some View {
        let selected = navigation.tab == tab
        return Button {
            onSelect(tab)
        } label: {
            Text(tab.title)
                .font(.system(size: 13, weight: selected ? .semibold : .regular))
                .foregroundStyle(selected ? Color.white : tokens.fg)
                .lineLimit(1)
                .truncationMode(.tail)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 12)
                .padding(.vertical, 7)
                .background {
                    if selected {
                        BrandFill()
                            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                            .shadow(color: GlassAppearance.shared.accent.shadow, radius: 7, y: 3)
                            .matchedGeometryEffect(id: "pill", in: pill)
                    }
                }
                .contentShape(Rectangle())
        }
        .buttonStyle(PressButtonStyle(chrome: false))
    }

    private func footer(tokens: Tokens) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(UIStrings.tr("sidebar.version", state.version))
                .font(.system(size: 11))
                .foregroundStyle(tokens.fgMuted)
            Button(UIStrings.tr("update.checkNow")) {
                Task { await updater.check(manual: true) }
            }
            .buttonStyle(PressButtonStyle())
            .disabled(updater.status == .checking)
            .pointerCursor()
            if let updateStatus {
                Text(updateStatus)
                    .font(.system(size: 11))
                    .foregroundStyle(tokens.fgMuted)
                    .fixedSize(horizontal: false, vertical: true)
            }
            Button(UIStrings.tr("info.openGitHub"), action: state.openGitHub)
                .buttonStyle(.plain)
                .font(.system(size: 11))
                .foregroundStyle(tokens.fgMuted)
                .pointerCursor()
        }
        .padding(.horizontal, 12)
    }
}
