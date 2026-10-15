import EVKeyKit
import SwiftUI

/// The translate popup: only the translated text, a red error, or a muted "Translating…".
/// A thin bar along the top drains over the display time once the result or error is shown.
struct TranslateView: View {
    @ObservedObject var model: TranslateModel
    /// Observed so a live UI language switch and a changed display time redraw.
    @ObservedObject private var state = AppState.shared

    var body: some View {
        let ms = state.settings.translatePopupMs
        VStack(spacing: 0) {
            DrainBar(active: model.status != .loading && ms > 0, token: model.token, seconds: Double(ms) / 1000)
            // Natural height when it fits; scrolls only past the controller's max height.
            ViewThatFits(in: .vertical) {
                padded
                ScrollView { padded }
            }
        }
        .frame(width: 340)
        .glassSurface(cornerRadius: 12)
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
        .contentShape(Rectangle())
        .onTapGesture { TranslateController.shared.close() }
    }

    private var padded: some View {
        content
            .font(.system(size: 15, weight: .medium))
            .lineSpacing(7)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, 14)
            .padding(.horizontal, 18)
            .padding(.bottom, 16)
    }

    @ViewBuilder private var content: some View {
        switch model.status {
        case .loading:
            Text(UIStrings.tr("translate.loading"))
                .foregroundStyle(.secondary)
        case .ok:
            Text(model.result)
        case .failed(let failure):
            Text(failure.message)
                .foregroundStyle(.red)
        }
    }
}

/// 3 pt bar flush with the top edge. It stays in place (hidden) while loading and when the display time is "never".
struct DrainBar: View {
    let active: Bool
    /// Restarts the drain each time the popup starts loading or settles.
    let token: Int
    let seconds: Double
    @State private var drained = false

    var body: some View {
        Rectangle()
            .fill(LinearGradient(
                colors: [Color.accentColor.opacity(0.55), Color.accentColor],
                startPoint: .leading, endPoint: .trailing))
            .frame(maxWidth: .infinity)
            .frame(height: 3)
            .scaleEffect(x: drained ? 0 : 1, anchor: .leading)
            .opacity(active ? 1 : 0)
            .task(id: token) {
                drained = false
                guard active else { return }
                // Let the full bar render first, so the drain animates from full.
                try? await Task.sleep(nanoseconds: 30_000_000)
                withAnimation(.linear(duration: seconds)) { drained = true }
            }
    }
}
