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
    private static let maxHeight: CGFloat = 360

    private let model = ActionPanelModel()
    private let hosting: NSHostingController<ActionPanelView>
    private let panel: TranslatePanel
    /// Pointer position when the hotkey fired; the panel is placed from it.
    private var anchor = NSPoint.zero
    /// The selection the panel acts on.
    private var source = ""
    /// True while the selection is captured or an action runs, so a second hotkey or click cannot overlap.
    private var busy = false
    /// Bumped by each hotkey and by close, so an action that finishes after the panel was closed shows nothing.
    private var runID = 0
    /// Bumped whenever the panel shows or closes, so an auto-close timer only closes the showing it was started for.
    private var generation = 0
    /// Clicks and Esc outside the menu close it; installed only while the menu waits for a choice.
    private var dismissMonitors: [Any] = []

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
    }

    /// Hotkey entry point. Secure Input is checked by the event tap before this runs. Outside an editable field it does
    /// nothing: no copy, no message.
    func open() async {
        guard !busy, EditableField.isFocused() else { return }
        busy = true
        defer { busy = false }
        runID += 1
        stopDismissMonitors()
        anchor = NSEvent.mouseLocation
        let text = await TextSelection.capture()
        if text.isEmpty { return fail(UIStrings.tr("translate.error.noSelection")) }
        if text.count > Translate.maxChars { return fail(UIStrings.tr("actions.error.tooLong")) }
        source = text
        model.show(.menu)
        present()
        startDismissMonitors()
    }

    /// A menu entry was chosen: run it on the captured selection, show the result, then apply `translate_action`.
    func perform(_ action: PanelAction) async {
        guard !busy else { return }
        busy = true
        defer { busy = false }
        stopDismissMonitors()
        let id = runID
        model.show(.loading(action))
        present()
        do {
            let result = try await TextActions.run(action, text: source)
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
        resize()
        panel.orderFrontRegardless()
    }

    /// Fits the panel to the new content and starts its auto-close timer, if the display time is not "never".
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

    /// Fits the panel to its content, capped at `maxHeight`, then places it by the pointer. SwiftUI redraws on the next
    /// run-loop turn, so the fit runs again shortly after to pick up the new content.
    private func resize() {
        fit()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in self?.fit() }
    }

    private func fit() {
        hosting.view.layoutSubtreeIfNeeded()
        let height = min(max(hosting.view.fittingSize.height, 1), Self.maxHeight)
        panel.setFrame(
            TextSelection.popupFrame(anchor: anchor, size: NSSize(width: Self.width, height: height)), display: true)
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
