import EVKeyKit
import SwiftUI

/// The action panel's menu: Translate, Add diacritics, Fix spelling, a Rewrite group of three tone chips, Convert to Unicode.
/// The AI entries are disabled without a Gemini key, and a link opens the Translate tab where the key is entered.
struct ActionMenuView: View {
    let run: (PanelAction) -> Void
    let setup: () -> Void
    @ObservedObject private var state = AppState.shared

    private static let tones: [GeminiAction] = [.rewriteFormal, .rewriteShorter, .rewriteFriendly]

    var body: some View {
        let ai = state.geminiKeySaved
        VStack(alignment: .leading, spacing: 2) {
            Text(UIStrings.tr("actions.title").uppercased())
                .font(.system(size: 11, weight: .semibold))
                .tracking(0.5)
                .foregroundStyle(.secondary)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
            ActionMenuRow(symbol: "globe", title: UIStrings.tr("actions.translate")) { run(.ai(.translate)) }
            ActionMenuRow(symbol: "character", title: UIStrings.tr("actions.add_diacritics"), enabled: ai) { run(.ai(.addDiacritics)) }
            ActionMenuRow(symbol: "textformat.abc", title: UIStrings.tr("actions.fix_spelling"), enabled: ai) { run(.ai(.fixSpelling)) }
            rewriteGroup(enabled: ai)
            ActionMenuRow(symbol: "arrow.left.arrow.right", title: UIStrings.tr("actions.convert")) { run(.convert) }
            if !ai {
                Button(action: setup) {
                    Text(UIStrings.tr("actions.setup_gemini"))
                        .font(.system(size: 12))
                        .foregroundStyle(Color.accentColor)
                        .multilineTextAlignment(.leading)
                        .fixedSize(horizontal: false, vertical: true)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .padding(.top, 4)
            }
        }
        .padding(.horizontal, 8)
        .padding(.top, 4)
        .padding(.bottom, 10)
    }

    private func rewriteGroup(enabled: Bool) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                Image(systemName: "pencil")
                    .font(.system(size: 14))
                    .foregroundStyle(Color.accentColor)
                    .frame(width: 18)
                Text(UIStrings.tr("actions.rewrite"))
                    .font(.system(size: 14))
            }
            .padding(.horizontal, 10)
            .padding(.top, 8)
            HStack(spacing: 6) {
                ForEach(Self.tones, id: \.self) { tone in
                    Button { run(.ai(tone)) } label: {
                        Text(UIStrings.tr("actions.\(tone.rawValue)"))
                            .font(.system(size: 13))
                            .padding(.horizontal, 10)
                            .padding(.vertical, 5)
                            .background(Color.primary.opacity(0.1), in: Capsule())
                            .overlay { Capsule().strokeBorder(Color.white.opacity(0.25), lineWidth: 0.5) }
                            .contentShape(Capsule())
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.leading, 28)
            .padding(.trailing, 10)
            .padding(.bottom, 6)
            .disabled(!enabled)
            .opacity(enabled ? 1 : 0.4)
        }
    }
}
