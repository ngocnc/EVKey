import AppKit
import EVKeyKit
import SwiftUI

/// Non-activating panel: showing it never takes focus, so the app the user is in stays frontmost.
final class TranslatePanel: NSPanel {
    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }
}

/// The translate flow: copy the selection, translate it, show the result in a small popup by the pointer,
/// then run the chosen action (show only, copy, or paste over the selection).
@MainActor
final class TranslateController {
    static let shared = TranslateController()

    private static let width: CGFloat = 340
    private static let maxHeight: CGFloat = 360

    private let model = TranslateModel()
    private let panel: TranslatePanel
    /// Pointer position when the hotkey fired; the popup is placed from it.
    private var anchor = NSPoint.zero
    /// Bumped by each translation, so an older one that finishes late cannot overwrite a newer popup.
    private var runID = 0
    /// Bumped whenever the popup shows or closes, so an auto-close timer only closes the showing it was started for.
    private var generation = 0

    private init() {
        let hosting = NSHostingController(rootView: TranslateView(model: model))
        hosting.sizingOptions = []
        panel = TranslatePanel(
            contentRect: NSRect(x: 0, y: 0, width: Self.width, height: 40),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered, defer: true)
        panel.contentViewController = hosting
        panel.isReleasedWhenClosed = false
        panel.isFloatingPanel = true
        panel.level = .statusBar
        panel.backgroundColor = .clear
        panel.isOpaque = false
        // The glass draws its own edge; a window shadow would outline the transparent padding.
        panel.hasShadow = false
        panel.hidesOnDeactivate = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient, .ignoresCycle]
    }

    /// Hotkey entry point. Secure Input is checked by the event tap before this runs.
    func run() async {
        anchor = NSEvent.mouseLocation
        runID += 1
        let id = runID
        let text = await TextSelection.capture()
        guard id == runID else { return }
        model.start()
        present()
        do {
            let result = try await translate(text)
            guard id == runID else { return }
            model.succeed(result)
            settle()
            await TextSelection.apply(AppState.shared.settings.translateAction, result: result)
        } catch {
            guard id == runID else { return }
            model.fail((error as? TranslateFailure) ?? .failed(error.localizedDescription))
            settle()
        }
    }

    /// Closes the popup on click or when its display time ends. Bumping `generation` also cancels a pending timer.
    func close() {
        generation += 1
        guard panel.isVisible else { return }
        panel.orderOut(nil)
    }

    private func present() {
        generation += 1
        resize()
        panel.orderFrontRegardless()
    }

    /// Fits the popup to the new text and starts its auto-close timer, if the display time is not "never".
    private func settle() {
        resize()
        generation += 1
        let token = generation
        let ms = AppState.shared.settings.translatePopupMs
        guard ms > 0 else { return }
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: UInt64(ms) * 1_000_000)
            if token == self.generation { self.close() }
        }
    }

    /// The configured service translates: Vietnamese source goes to English, anything else to Vietnamese.
    private func translate(_ text: String) async throws -> String {
        guard !text.isEmpty else { throw TranslateFailure.noSelection }
        guard text.count <= Translate.maxChars else { throw TranslateFailure.tooLong }
        do {
            return try await TextActions.run(.translate, text: text)
        } catch let error as GeminiError {
            throw TranslateFailure.shown(error.message)
        } catch {
            throw TranslateFailure.failed(error.localizedDescription)
        }
    }

    /// Fits the popup to its text, capped at `maxHeight` (the text scrolls beyond that), then places it by the pointer.
    /// SwiftUI redraws on the next run-loop turn, so the fit runs again shortly after to pick up the new content.
    private func resize() {
        fit()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in self?.fit() }
    }

    private func fit() {
        // The hosting view's fittingSize is zero here; ask SwiftUI for the content height instead.
        let fitted = (panel.contentViewController as? NSHostingController<TranslateView>)?
            .sizeThatFits(in: NSSize(width: Self.width, height: .infinity)).height ?? 40
        let height = min(max(fitted, 1), Self.maxHeight)
        panel.setFrame(
            TextSelection.popupFrame(anchor: anchor, size: NSSize(width: Self.width, height: height)), display: true)
    }
}

/// Why a translation could not be shown.
enum TranslateFailure: Error, Equatable {
    case noSelection
    case tooLong
    case failed(String)
    /// Text that is already worded for the user (a Gemini error).
    case shown(String)

    var message: String {
        switch self {
        case .noSelection: return UIStrings.tr("translate.error.noSelection")
        case .tooLong: return UIStrings.tr("translate.error.tooLong")
        case .failed(let detail): return UIStrings.tr("translate.error.failed", detail)
        case .shown(let text): return text
        }
    }
}

/// What the popup shows. `TranslateController` sets it; the view redraws on change.
@MainActor
final class TranslateModel: ObservableObject {
    enum Status: Equatable {
        case loading
        case ok
        case failed(TranslateFailure)
    }

    @Published var status = Status.loading
    @Published var result = ""
    /// Changes whenever a popup starts loading or settles, so the progress bar restarts only then.
    @Published var token = 0

    func start() {
        status = .loading
        result = ""
        token += 1
    }

    func succeed(_ text: String) {
        result = text
        status = .ok
        token += 1
    }

    func fail(_ failure: TranslateFailure) {
        status = .failed(failure)
        token += 1
    }
}
