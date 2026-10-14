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
    private static let copyKey: CGKeyCode = 0x08  // kVK_ANSI_C
    private static let pasteKey: CGKeyCode = 0x09  // kVK_ANSI_V
    private static let modifiers: CGEventFlags = [.maskCommand, .maskAlternate, .maskControl, .maskShift]

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
        let text = await captureSelection()
        guard id == runID else { return }
        model.start()
        present()
        do {
            let result = try await translate(text)
            guard id == runID else { return }
            model.succeed(result)
            settle()
            await perform(AppState.shared.settings.translateAction, result: result)
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

    /// Runs the chosen action once the result is on screen. The popup never takes focus, so Cmd+V goes to the app the user was in.
    private func perform(_ action: TranslateAction, result: String) async {
        switch action {
        case .show:
            return
        case .copy:
            writePasteboard(result)
        case .replace:
            await waitForModifiersReleased()
            let saved = NSPasteboard.general.string(forType: .string)
            writePasteboard(result)
            KeyInjector.commandKey(Self.pasteKey)
            try? await Task.sleep(nanoseconds: 300_000_000)
            writePasteboard(saved)
        }
    }

    /// Vietnamese source goes to English, anything else to Vietnamese.
    private func translate(_ text: String) async throws -> String {
        guard !text.isEmpty else { throw TranslateFailure.noSelection }
        guard text.count <= Translate.maxChars else { throw TranslateFailure.tooLong }
        do {
            // Ask for Vietnamese first; only a Vietnamese source needs the second request, into English.
            let first = try await fetch(text, target: "vi")
            let to = Translate.target(detected: first.detected)
            if to == "vi" { return first.text }
            return try await fetch(text, target: to).text
        } catch {
            throw TranslateFailure.failed(error.localizedDescription)
        }
    }

    private func fetch(_ text: String, target: String) async throws -> Translate.Translation {
        guard let url = Translate.url(text: text, target: target) else { throw URLError(.badURL) }
        var request = URLRequest(url: url)
        request.timeoutInterval = 10
        request.setValue("EVKey/\(AppState.shared.version)", forHTTPHeaderField: "User-Agent")
        let (data, response) = try await URLSession.shared.data(for: request)
        guard let http = response as? HTTPURLResponse, http.statusCode == 200,
              let translation = Translate.parse(data) else { throw URLError(.cannotParseResponse) }
        return translation
    }

    /// Copies the current selection with Cmd+C and restores the pasteboard. Returns "" when nothing new was copied.
    private func captureSelection() async -> String {
        await waitForModifiersReleased()
        let pasteboard = NSPasteboard.general
        let saved = pasteboard.string(forType: .string)
        let before = pasteboard.changeCount
        KeyInjector.commandKey(Self.copyKey)
        let deadline = Date().addingTimeInterval(0.6)
        while pasteboard.changeCount == before && Date() < deadline {
            try? await Task.sleep(nanoseconds: 15_000_000)
        }
        guard pasteboard.changeCount != before else { return "" }
        let copied = pasteboard.string(forType: .string) ?? ""
        writePasteboard(saved)
        return copied.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    /// The hotkey's own modifiers are still down; Cmd+C sent while they are held would be a different shortcut.
    private func waitForModifiersReleased() async {
        let deadline = Date().addingTimeInterval(1)
        while !CGEventSource.flagsState(.combinedSessionState).intersection(Self.modifiers).isEmpty && Date() < deadline {
            try? await Task.sleep(nanoseconds: 20_000_000)
        }
    }

    /// Fits the popup to its text, capped at `maxHeight` (the text scrolls beyond that), then places it by the pointer:
    /// below-right, flipped above when it would fall off the bottom, kept inside the screen's visible area.
    private func resize() {
        let fitted = panel.contentView?.fittingSize.height ?? 40
        let height = min(max(fitted, 1), Self.maxHeight)
        let screen = NSScreen.screens.first { $0.frame.contains(anchor) } ?? NSScreen.main
        let area = screen?.visibleFrame ?? .zero
        let margin: CGFloat = 6
        let x = min(max(anchor.x + 12, area.minX + margin), area.maxX - Self.width - margin)
        var y = anchor.y - 16 - height
        if y < area.minY + margin { y = anchor.y + 16 }
        y = min(max(y, area.minY + margin), area.maxY - height - margin)
        panel.setFrame(NSRect(x: x, y: y, width: Self.width, height: height), display: true)
    }
}

/// Replaces the pasteboard string; nil clears it.
private func writePasteboard(_ string: String?) {
    let pasteboard = NSPasteboard.general
    pasteboard.clearContents()
    if let string { pasteboard.setString(string, forType: .string) }
}

/// Why a translation could not be shown.
enum TranslateFailure: Equatable {
    case noSelection
    case tooLong
    case failed(String)

    var message: String {
        switch self {
        case .noSelection: return UIStrings.tr("translate.error.noSelection")
        case .tooLong: return UIStrings.tr("translate.error.tooLong")
        case .failed(let detail): return UIStrings.tr("translate.error.failed", detail)
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
