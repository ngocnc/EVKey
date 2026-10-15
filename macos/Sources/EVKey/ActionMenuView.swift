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

    /// Màu ô icon theo mockup (blue, purple, green, orange, teal, gray).
    private enum Tile {
        static let blue = [Color(hex: 0x64D1FF), Color(hex: 0x5E5CE6)]
        static let purple = [Color(hex: 0xC58BFF), Color(hex: 0x8A4DE8)]
        static let green = [Color(hex: 0x4CD97B), Color(hex: 0x1E9E4A)]
        static let orange = [Color(hex: 0xFFB340), Color(hex: 0xFF7A1A)]
        static let teal = [Color(hex: 0x3FD3C6), Color(hex: 0x0F9C9C)]
        static let gray = [Color(hex: 0x9AA0AA), Color(hex: 0x5F6570)]
    }

    var body: some View {
        let ai = state.geminiKeySaved
        VStack(alignment: .leading, spacing: 2) {
            Text(UIStrings.tr("actions.title").uppercased())
                .font(.system(size: 11, weight: .semibold))
                .tracking(0.4)
                .foregroundStyle(.secondary)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .frame(maxWidth: .infinity, alignment: .leading)
                // The title row is the handle for moving the panel.
                .overlay(WindowDragArea())
            ActionMenuRow(symbol: "globe", tile: Tile.blue, title: UIStrings.tr("actions.translate")) { run(.ai(.translate)) }
            ActionMenuRow(symbol: "character", tile: Tile.purple, title: UIStrings.tr("actions.add_diacritics"), ai: true, enabled: ai) { run(.ai(.addDiacritics)) }
            ActionMenuRow(symbol: "checkmark", tile: Tile.green, title: UIStrings.tr("actions.fix_spelling"), ai: true, enabled: ai) { run(.ai(.fixSpelling)) }
            chipGroup(symbol: "pencil", tile: Tile.orange, title: "actions.rewrite", chips: Self.tones, enabled: ai)
            chipGroup(symbol: "arrow.left.arrow.right", tile: Tile.teal, title: "actions.convert", chips: Self.conversions, enabled: true)
            chipGroup(symbol: "textformat", tile: Tile.gray, title: "actions.text", chips: Self.textTools, enabled: true)
            if !ai {
                Button(action: setup) {
                    Text(UIStrings.tr("actions.setup_gemini"))
                        .font(.system(size: 11.5))
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.leading)
                        .fixedSize(horizontal: false, vertical: true)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 8)
                        .background(.primary.opacity(0.05), in: RoundedRectangle(cornerRadius: 9, style: .continuous))
                        .overlay {
                            RoundedRectangle(cornerRadius: 9, style: .continuous)
                                .strokeBorder(.primary.opacity(0.15), style: StrokeStyle(lineWidth: 1, dash: [4, 3]))
                        }
                        .contentShape(Rectangle())
                }
                .buttonStyle(PressButtonStyle(chrome: false))
                .padding(.top, 6)
            }
        }
        .padding(.horizontal, 8)
        .padding(.top, 4)
        .padding(.bottom, 10)
    }

    private func chipGroup(symbol: String, tile: [Color], title: String, chips: [(String, PanelAction)], enabled: Bool) -> some View {
        HStack(alignment: .top, spacing: 10) {
            ActionTile(symbol: symbol, colors: tile)
            Text(UIStrings.tr(title))
                .font(.system(size: 13.5))
                .lineLimit(1)
                .frame(width: 76, alignment: .leading)
                .padding(.top, 2)
            ActionChipFlow(chips: chips, run: run)
                .disabled(!enabled)
                .opacity(enabled ? 1 : 0.4)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
    }
}
