import EVKeyKit
import SwiftUI

/// Thẻ đầu trang Gõ tiếng Việt: trạng thái Việt/Anh, phím chuyển, quyền Trợ năng và công tắc lớn.
struct SettingsHero: View {
    /// Tên kiểu gõ đang dùng, ví dụ "Telex"; chỉ hiển thị khi đang gõ tiếng Việt.
    let inputLabel: String
    @ObservedObject private var state = AppState.shared
    @Environment(\.colorScheme) private var scheme

    /// Công tắc bật = gõ tiếng Việt; dùng cùng đường đi với popup.
    private var vietnameseBinding: Binding<Bool> {
        Binding(get: { !state.isEnglish }, set: { state.setEnglish(!$0) })
    }

    private var title: String {
        state.isEnglish ? UIStrings.tr("hero.english") : UIStrings.tr("hero.vietnamese", inputLabel)
    }

    private var hint: String {
        let chord = state.settings.toggleHotkey
        guard chord != "none" else { return UIStrings.tr("hero.noToggle") }
        return UIStrings.tr("hero.toggleHint", SettingsKeysTab.hotkeyLabel(chord))
    }

    var body: some View {
        let tokens = Tokens(scheme)
        HStack(alignment: .center, spacing: 16) {
            LogoMark(letter: state.isEnglish ? "E" : "V", size: 50)
            VStack(alignment: .leading, spacing: 6) {
                Text(title)
                    .font(.system(size: 17, weight: .bold))
                    .foregroundStyle(tokens.fg)
                Text(hint)
                    .font(.system(size: 12))
                    .foregroundStyle(tokens.fgMuted)
                    .fixedSize(horizontal: false, vertical: true)
                accessibilityStatus(tokens: tokens)
            }
            Spacer(minLength: 12)
            Toggle("", isOn: vietnameseBinding)
                .labelsHidden()
                .toggleStyle(SettingsSwitchStyle(large: true))
        }
        .padding(18)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background { HeroMesh().clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous)) }
        .settingsBlock()
    }

    private func accessibilityStatus(tokens: Tokens) -> some View {
        let granted = state.accessibilityTrusted
        return HStack(spacing: 8) {
            Circle()
                .fill(granted ? Color.green : Color.orange)
                .frame(width: 8, height: 8)
            Text(granted ? UIStrings.tr("typing.access.granted") : UIStrings.tr("typing.access.missing"))
                .font(.system(size: 12, weight: .medium))
                .foregroundStyle(tokens.fg)
            if !granted {
                Button(UIStrings.tr("common.openSettings"), action: state.openAccessibilitySettings)
                    .buttonStyle(PressButtonStyle())
                    .pointerCursor()
            }
        }
    }
}
