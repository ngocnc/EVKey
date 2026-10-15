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
        VStack(alignment: .leading, spacing: 6) {
            Text(UIStrings.tr("sidebar.version", state.version))
                .font(.system(size: 11))
                .foregroundStyle(tokens.fgMuted)
            footerLink(
                updater.status == .checking ? UIStrings.tr("update.checking") : UIStrings.tr("update.checkNow"),
                symbol: "arrow.triangle.2.circlepath", tokens: tokens
            ) {
                Task { await updater.check(manual: true) }
            }
            .disabled(updater.status == .checking)
            if let updateStatus {
                Text(updateStatus)
                    .font(.system(size: 11))
                    .foregroundStyle(tokens.fgMuted)
                    .fixedSize(horizontal: false, vertical: true)
                    .padding(.leading, 18)
            }
            footerLink(UIStrings.tr("info.openGitHub"), symbol: "arrow.up.right.square", tokens: tokens, action: state.openGitHub)
        }
        .padding(.horizontal, 12)
    }

    /// Liên kết chân sidebar: icon nhỏ + chữ phụ, rê chuột thì chữ đậm màu lên. Nhẹ hơn nút để không tranh với các mục.
    private func footerLink(_ title: String, symbol: String, tokens: Tokens, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Label(title, systemImage: symbol)
                .font(.system(size: 11.5))
                .labelStyle(.titleAndIcon)
                .contentShape(Rectangle())
        }
        .buttonStyle(FooterLinkStyle(tokens: tokens))
    }
}

/// Chữ phụ, rê chuột thì sáng lên, bấm thì lún nhẹ.
private struct FooterLinkStyle: ButtonStyle {
    let tokens: Tokens

    func makeBody(configuration: Configuration) -> some View {
        FooterLinkLabel(label: configuration.label, pressed: configuration.isPressed, tokens: tokens)
    }
}

private struct FooterLinkLabel<Label: View>: View {
    let label: Label
    let pressed: Bool
    let tokens: Tokens
    @State private var hovered = false

    var body: some View {
        label
            .foregroundStyle(hovered ? tokens.fg : tokens.fgMuted)
            .scaleEffect(pressed ? 0.97 : 1)
            .onHover { hovered = $0 }
            .pointerCursor()
            .motion(.easeOut(duration: 0.12), value: hovered)
    }
}
