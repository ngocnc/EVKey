import EVKeyKit
import SwiftUI

/// The action panel: the menu, then "Đang …" while an action runs, then the result or a red error. Copy and close sit
/// top-right; a thin bar along the top drains over the display time once a result or error is shown.
struct ActionPanelView: View {
    @ObservedObject var model: ActionPanelModel
    /// Observed so a live UI language switch and a changed display time redraw.
    @ObservedObject private var state = AppState.shared
    @Environment(\.colorScheme) private var scheme

    private static let corner: CGFloat = 14

    var body: some View {
        let ms = state.settings.translatePopupMs
        VStack(spacing: 0) {
            DrainBar(active: model.mode.isSettled && ms > 0, token: model.token, seconds: Double(ms) / 1000)
            content
        }
        .frame(width: ActionPanelController.width)
        .background(WindowDragArea())
        .glassSurface(cornerRadius: Self.corner)
        .liquidGlassEdge(cornerRadius: Self.corner)
        .clipShape(RoundedRectangle(cornerRadius: Self.corner, style: .continuous))
        .overlay(alignment: .topTrailing) { tools }
    }

    @ViewBuilder private var content: some View {
        switch model.mode {
        case .menu:
            ActionMenuView(
                run: { action in Task { await ActionPanelController.shared.perform(action) } },
                setup: { ActionPanelController.shared.openSetup() })
        case .loading(let action):
            text(UIStrings.tr(action.loadingKey)).foregroundStyle(.secondary)
        case .ok(let result):
            text(result)
        case .failed(let message):
            text(message).foregroundStyle(Tokens(scheme).danger)
        }
    }

    /// Result, error or progress text; dragging it moves the panel. A click on a result or error closes the panel, as on
    /// the translate popup; while loading a click does nothing, so extra clicks on a chosen entry cannot close it.
    private func text(_ string: String) -> some View {
        let body = Text(string)
            .font(.system(size: 15, weight: .medium))
            .lineSpacing(7)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, 14)
            .padding(.leading, 18)
            .padding(.trailing, 70)
            .padding(.bottom, 16)
        // Natural height when it fits; scrolls only past the controller's max height.
        return ViewThatFits(in: .vertical) {
            body.overlay(WindowDragArea { if model.mode.isSettled { ActionPanelController.shared.close() } })
            ScrollView { body }
        }
    }

    private var tools: some View {
        HStack(spacing: 4) {
            if case .ok = model.mode {
                toolButton("doc.on.doc", label: "actions.copy") { ActionPanelController.shared.copyResult() }
            }
            toolButton("xmark", label: "actions.close") { ActionPanelController.shared.close() }
        }
        .padding(8)
    }

    private func toolButton(_ symbol: String, label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: symbol)
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(.secondary)
                .frame(width: 26, height: 26)
                .contentShape(Circle())
        }
        .buttonStyle(.plain)
        .help(UIStrings.tr(label))
    }
}
