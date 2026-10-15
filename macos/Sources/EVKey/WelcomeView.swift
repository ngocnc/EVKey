import EVKeyKit
import SwiftUI

/// First-open card over Bảng điều khiển: pick the input method, see the toggle hotkey, try typing. Same three steps as
/// the Windows welcome screen. The caller decides when it shows (see `WelcomeView.seen`) and what closing does.
struct WelcomeView: View {
    /// UserDefaults flag, set once the card was closed. Same name as the Windows `localStorage` key.
    static let seenKey = "evkey-welcomed"

    static var seen: Bool {
        get { UserDefaults.standard.bool(forKey: seenKey) }
        set { UserDefaults.standard.set(newValue, forKey: seenKey) }
    }

    let onClose: () -> Void
    @ObservedObject private var state = AppState.shared
    @State private var tryText = ""
    @FocusState private var tryFocused: Bool

    private static let inputChoices: [ChoiceOption<InputKind>] = [
        ChoiceOption(value: .telex, label: "Telex"),
        ChoiceOption(value: .vni, label: "VNI"),
        ChoiceOption(value: .simpleTelex1, label: "Simple Telex 1"),
        ChoiceOption(value: .simpleTelex2, label: "Simple Telex 2"),
    ]

    var body: some View {
        ZStack {
            Color.black.opacity(0.28)
            card
        }
        .onAppear { tryFocused = true }
        // The try field has focus and eats Esc before the Skip button's cancel shortcut sees it.
        .onExitCommand(perform: onClose)
    }

    private var card: some View {
        VStack(spacing: 6) {
            LogoMark(letter: "V", size: 64)
            Text(UIStrings.misc("welcome.title"))
                .font(.system(size: 22, weight: .bold))
                .padding(.top, 8)
            Text(UIStrings.misc("welcome.sub"))
                .font(.system(size: 13))
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.bottom, 10)

            if !state.accessibilityTrusted {
                HStack(spacing: 12) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: 18))
                        .foregroundStyle(.orange)
                    SettingsRowLabel(title: UIStrings.tr("typing.access.missing"), subtitle: UIStrings.tr("typing.access.missingDetail"))
                    Spacer(minLength: 8)
                    Button(UIStrings.tr("common.openSettings"), action: state.openAccessibilitySettings)
                        .buttonStyle(PressButtonStyle(primary: true))
                }
                .padding(12)
                .settingsBlock()
                .padding(.bottom, 6)
            }

            VStack(alignment: .leading, spacing: 16) {
                step(1, title: UIStrings.misc("welcome.stepInput")) {
                    SettingsDropdown(choices: Self.inputChoices, selection: state.binding(\.inputType))
                }
                step(2, title: UIStrings.misc("welcome.stepToggle")) {
                    Text(hotkeyLine)
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
                step(3, title: UIStrings.misc("welcome.stepTry")) {
                    TextField(tryPlaceholder, text: $tryText, axis: .vertical)
                        .lineLimit(2...4)
                        .textFieldStyle(.roundedBorder)
                        .focused($tryFocused)
                }
            }
            .padding(.top, 6)

            HStack {
                Spacer()
                Button(UIStrings.misc("welcome.skip"), action: onClose)
                    .buttonStyle(PressButtonStyle())
                    .keyboardShortcut(.cancelAction)
                Button(UIStrings.misc("welcome.start"), action: onClose)
                    .buttonStyle(PressButtonStyle(primary: true))
                    .keyboardShortcut(.defaultAction)
            }
            .padding(.top, 18)
        }
        .padding(.horizontal, 28)
        .padding(.top, 28)
        .padding(.bottom, 22)
        .frame(width: 500)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 20, style: .continuous))
        .liquidGlassEdge(cornerRadius: 20)
        .shadow(color: .black.opacity(0.3), radius: 24, y: 12)
    }

    private var hotkeyLine: String {
        let hotkey = state.settings.toggleHotkey
        if hotkey == "none" { return UIStrings.misc("welcome.toggleNone") }
        return UIStrings.misc("welcome.toggleLine", SettingsKeysTab.hotkeyLabel(hotkey))
    }

    private var tryPlaceholder: String {
        UIStrings.misc(state.settings.inputType == .vni ? "welcome.tryVni" : "welcome.tryTelex")
    }

    /// A numbered step: the number badge on the left, bold title and the control on the right.
    private func step<Content: View>(
        _ number: Int, title: String, @ViewBuilder content: () -> Content
    ) -> some View {
        HStack(alignment: .top, spacing: 12) {
            Text("\(number)")
                .font(.system(size: 12, weight: .bold))
                .foregroundStyle(.white)
                .frame(width: 24, height: 24)
                .background { BrandFill().clipShape(Circle()) }
            VStack(alignment: .leading, spacing: 6) {
                Text(title)
                    .font(.system(size: 13, weight: .semibold))
                    .frame(minHeight: 28, alignment: .leading)
                content()
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}
