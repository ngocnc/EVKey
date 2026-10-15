import AppKit
import EVKeyKit
import SwiftUI

/// What the action panel shows. `ActionPanelController` sets it; the view redraws on change.
@MainActor
final class ActionPanelModel: ObservableObject {
    enum Mode: Equatable {
        case menu
        case loading(PanelAction)
        case ok(String)
        case failed(String)

        /// A result or error: the panel hides itself after the display time.
        var isSettled: Bool {
            switch self {
            case .ok, .failed: return true
            case .menu, .loading: return false
            }
        }
    }

    @Published private(set) var mode = Mode.menu
    /// Changes with every mode, so the progress bar restarts only then.
    @Published private(set) var token = 0

    func show(_ mode: Mode) {
        self.mode = mode
        token += 1
    }
}

/// The action hotkey flow (`crates/app/src/actions.rs`): when focus is in an editable field, copy the selection, show a
/// menu by the pointer, run the chosen action and show its result. The panel never takes focus.
@MainActor
final class ActionPanelController {
    static let shared = ActionPanelController()
    static let width: CGFloat = 340
    private static let maxHeight: CGFloat = 480

    private let model = ActionPanelModel()
    private let hosting: NSHostingController<ActionPanelView>
    private let panel: TranslatePanel
    /// Pointer position when the hotkey fired; the panel is placed from it.
    private var anchor = NSPoint.zero
    /// The selection being copied (Cmd+C waits for the hotkey's modifiers to be released); a menu choice awaits it.
    private var selection: Task<String, Never>?
    /// True while the selection is copied, so a second hotkey cannot start an overlapping Cmd+C.
    private var capturing = false
    /// Bumped by each hotkey and by close, so an action that finishes after the panel was closed shows nothing.
    private var runID = 0
    /// Bumped whenever the panel shows or closes, so an auto-close timer only closes the showing it was started for.
    private var generation = 0
    /// Clicks and Esc outside the menu close it; installed only while the menu waits for a choice.
    private var dismissMonitors: [Any] = []
    /// Set once the user drags the panel; later fits keep its top-left instead of jumping back to the pointer.
    private var dragged = false
    /// Set once the panel is on screen for this hotkey; later fits resize it in place instead of placing it anew.
    private var placed = false
    /// True while `fit` moves the panel, so only user drags set `dragged`.
    private var placing = false

    private init() {
        let host = NSHostingController(rootView: ActionPanelView(model: model))
        host.sizingOptions = []
        hosting = host
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
        NotificationCenter.default.addObserver(forName: NSWindow.didMoveNotification, object: panel, queue: .main) {
            [weak self] _ in
            MainActor.assumeIsolated { if self?.placing == false { self?.dragged = true } }
        }
    }

    /// Hotkey entry point. Secure Input is checked by the event tap before this runs. Outside an editable field it does
    /// nothing: no copy, no message. In a known text field the menu shows at once while the selection is copied; when the
    /// app hides its focus from Accessibility (Lark), the menu shows only once something was copied.
    func open() async {
        guard !capturing else { return }
        let editable = EditableField.isFocused()
        guard editable != false else { return }
        runID += 1
        let id = runID
        stopDismissMonitors()
        anchor = NSEvent.mouseLocation
        dragged = false
        placed = false
        let task = Task { await TextSelection.capture() }
        selection = task
        if editable == true { showMenu() }
        capturing = true
        let text = await task.value
        capturing = false
        guard id == runID else { return }
        // Focus unknown and nothing copied: most likely not in text, so stay silent.
        if editable == nil && text.isEmpty { return }
        if let problem = Self.problem(text) {
            stopDismissMonitors()
            return fail(problem)
        }
        if editable == nil { showMenu() }
    }

    private func showMenu() {
        model.show(.menu)
        present()
        startDismissMonitors()
    }

    /// A menu entry was chosen: run it on the captured selection, show the result, then apply `translate_action`.
    /// Only the menu accepts a choice, so extra clicks while an action runs do nothing.
    func perform(_ action: PanelAction) async {
        guard case .menu = model.mode, let selection else { return }
        stopDismissMonitors()
        let id = runID
        model.show(.loading(action))
        present()
        let text = await selection.value
        // An empty or too long selection is reported by `open`.
        guard id == runID, Self.problem(text) == nil else { return }
        do {
            let result = try await TextActions.run(action, text: text)
            guard id == runID else { return }
            model.show(.ok(result))
            settle()
            await TextSelection.apply(AppState.shared.settings.translateAction, result: result)
        } catch {
            guard id == runID else { return }
            model.show(.failed(TextActions.message(for: error)))
            settle()
        }
    }

    private static func problem(_ text: String) -> String? {
        if text.isEmpty { return UIStrings.tr("translate.error.noSelection") }
        if text.count > Translate.maxChars { return UIStrings.tr("actions.error.tooLong") }
        return nil
    }

    /// Copy button on a result: puts the shown text on the pasteboard and hides the panel.
    func copyResult() {
        if case .ok(let text) = model.mode { TextSelection.write(text) }
        close()
    }

    /// "Thiết lập…": hides the panel and opens the Translate tab, where the Gemini key is entered.
    func openSetup() {
        close()
        SettingsWindowController.shared.show(tab: .translate)
    }

    /// Hides the panel. Bumping `generation` also cancels a pending timer; bumping `runID` drops a running action's result.
    func close() {
        generation += 1
        runID += 1
        stopDismissMonitors()
        guard panel.isVisible else { return }
        panel.orderOut(nil)
    }

    private func fail(_ message: String) {
        model.show(.failed(message))
        present()
        settle()
    }

    private func present() {
        generation += 1
        fit()
        panel.orderFrontRegardless()
        placed = true
    }

    /// Fits the panel to the new content and starts its auto-close timer, if the display time is not "never".
    private func settle() {
        fit()
        generation += 1
        let token = generation
        let ms = AppState.shared.settings.translatePopupMs
        guard ms > 0 else { return }
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: UInt64(ms) * 1_000_000)
            if token == self.generation { self.close() }
        }
    }

    /// Fits the panel to its content, capped at `maxHeight`, then places it by the pointer, before anything is drawn.
    /// Placing can move the panel to a screen of another scale, which changes text widths and so the chip line breaks;
    /// the second pass measures again on that screen.
    private func fit() {
        for _ in 0..<2 {
            // Applies a just-changed mode now instead of on the next run-loop turn, so the height is the new content's.
            hosting.view.layoutSubtreeIfNeeded()
            // `sizingOptions = []` leaves the view's fittingSize at zero, so ask SwiftUI for the content height.
            let fitted = hosting.sizeThatFits(in: NSSize(width: Self.width, height: .infinity)).height
            let height = min(max(fitted, 1), Self.maxHeight)
            let frame = place(NSSize(width: Self.width, height: height))
            placing = true
            panel.setFrame(frame, display: true)
            placing = false
        }
    }

    /// Below-right of the pointer, or above it near the bottom of the screen. Once shown, a new size keeps the edge by
    /// the pointer (the top edge once dragged) where it is, so the panel never jumps to the other side of the pointer.
    private func place(_ size: NSSize) -> NSRect {
        guard placed else { return TextSelection.popupFrame(anchor: anchor, size: size) }
        let keepBottom = !dragged && panel.frame.minY > anchor.y
        let y = keepBottom ? panel.frame.minY : panel.frame.maxY - size.height
        let area = (panel.screen ?? NSScreen.main)?.visibleFrame ?? .zero
        let margin: CGFloat = 6
        let clamped = min(max(y, area.minY + margin), area.maxY - size.height - margin)
        return NSRect(origin: NSPoint(x: panel.frame.minX, y: clamped), size: size)
    }

    /// Esc or a click outside the menu closes it. The panel never takes focus, so the events come from the app below
    /// (global monitor) or from EVKey's own windows (local monitor); neither is consumed.
    private func startDismissMonitors() {
        let clicks: NSEvent.EventTypeMask = [.leftMouseDown, .rightMouseDown, .otherMouseDown]
        let panel = self.panel
        let global = NSEvent.addGlobalMonitorForEvents(matching: clicks.union(.keyDown)) { [weak self] event in
            if event.type == .keyDown && event.keyCode != 53 { return }  // 53 = Esc
            Task { @MainActor in self?.close() }
        }
        let local = NSEvent.addLocalMonitorForEvents(matching: clicks) { [weak self] event in
            if event.window !== panel { Task { @MainActor in self?.close() } }
            return event
        }
        dismissMonitors = [global, local].compactMap { $0 }
    }

    private func stopDismissMonitors() {
        dismissMonitors.forEach(NSEvent.removeMonitor)
        dismissMonitors = []
    }
}
