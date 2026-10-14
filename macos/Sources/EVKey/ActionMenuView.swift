import EVKeyKit
import SwiftUI

/// The action panel's menu: Translate, Add diacritics, Fix spelling, a Rewrite group of three tone chips, then chip
/// groups for code table conversion (Unicode/TCVN3/VNI) and text tools (remove marks, change case).
/// The AI entries are disabled without a Gemini key, and a link opens the Translate tab where the key is entered.
struct ActionMenuView: View {
    let run: (PanelAction) -> Void
    let setup: () -> Void
    @ObservedObject private var state = AppState.shared

    private static let tones: [(String, PanelAction)] = [GeminiAction.rewriteFormal, .rewriteShorter, .rewriteFriendly]
        .map { ("actions.\($0.rawValue)", .ai($0)) }
    private static let conversions: [(String, PanelAction)] = [
        ("actions.convert_unicode", .text("convert")),
        ("actions.convert_tcvn3", .text("convert_tcvn3")),
        ("actions.convert_vni", .text("convert_vni")),
    ]
    private static let textTools: [(String, PanelAction)] = ["strip_marks", "case_upper", "case_lower", "case_title"]
        .map { ("actions.\($0)", .text($0)) }

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
            chipGroup(symbol: "pencil", title: "actions.rewrite", chips: Self.tones, enabled: ai)
            chipGroup(symbol: "arrow.left.arrow.right", title: "actions.convert", chips: Self.conversions, enabled: true)
            chipGroup(symbol: "textformat", title: "actions.text", chips: Self.textTools, enabled: true)
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

    private func chipGroup(symbol: String, title: String, chips: [(String, PanelAction)], enabled: Bool) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                Image(systemName: symbol)
                    .font(.system(size: 14))
                    .foregroundStyle(Color.accentColor)
                    .frame(width: 18)
                Text(UIStrings.tr(title))
                    .font(.system(size: 14))
            }
            .padding(.horizontal, 10)
            .padding(.top, 8)
            HStack(spacing: 6) {
                ForEach(chips, id: \.0) { chip in
                    Button { run(chip.1) } label: {
                        Text(UIStrings.tr(chip.0))
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
