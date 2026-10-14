import AppKit
import Combine
import SwiftUI

/// Owns the popup panel. Normal use: a panel under the menu bar item that closes on click-away.
/// Debug (`EVKEY_DEBUG_SHOW=popup`): a sticky panel at the top-right of the main screen for screenshots.
@MainActor
final class PopupController {
    static let shared = PopupController()

    private enum Anchor {
        case statusItem(NSRect)
        case topRight
    }

    private let panel: PopupPanel
    private var anchor: Anchor = .topRight
    private var sticky = false
    private var hiding = false
    private var observers = Set<AnyCancellable>()

    private init() {
        let hosting = NSHostingController(rootView: PopupView())
        // The window is a fixed size that fits both sections expanded; only the glass inside animates. Resizing the
        // window in step with a SwiftUI animation is what made the popup jump.
        hosting.sizingOptions = []

        panel = PopupPanel(contentRect: .zero, styleMask: [.borderless], backing: .buffered, defer: true)
        panel.contentViewController = hosting
        panel.setContentSize(NSSize(width: 320, height: 470))
        panel.isReleasedWhenClosed = false
        panel.isFloatingPanel = true
        panel.level = .statusBar
        panel.backgroundColor = .clear
        panel.isOpaque = false
        // The glass draws its own edge; a window shadow would outline the transparent padding as a dark rectangle.
        panel.hasShadow = false
        panel.hidesOnDeactivate = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient, .ignoresCycle]

        NotificationCenter.default.publisher(for: NSWindow.didResignKeyNotification, object: panel)
            .sink { [weak self] _ in self?.clickedAway() }
            .store(in: &observers)
    }

    /// Menu bar click: opens the popup under the item, or closes it when already open.
    func toggle(below button: NSStatusBarButton?) {
        if panel.isVisible && !hiding {
            hide()
            return
        }
        guard let button, let window = button.window else { return }
        anchor = .statusItem(window.convertToScreen(button.convert(button.bounds, to: nil)))
        sticky = false
        present()
    }

    /// Debug screenshot mode: stays open at the top-right corner of the main screen.
    func showSticky() {
        anchor = .topRight
        sticky = true
        present()
    }

    /// Fades the panel out, then removes it. A `present()` during the fade cancels the removal.
    func hide() {
        guard panel.isVisible, !hiding else { return }
        hiding = true
        NSAnimationContext.runAnimationGroup({ context in
            context.duration = 0.12
            panel.animator().alphaValue = 0
        }, completionHandler: { [weak self] in
            guard let self, hiding else { return }
            hiding = false
            panel.orderOut(nil)
            panel.alphaValue = 1
        })
    }

    private func present() {
        hiding = false
        NSApp.activate(ignoringOtherApps: true)
        reposition()
        panel.alphaValue = 0
        panel.makeKeyAndOrderFront(nil)
        NSAnimationContext.runAnimationGroup { context in
            context.duration = 0.15
            panel.animator().alphaValue = 1
        }
    }

    private func clickedAway() {
        if !sticky { hide() }
    }

    /// Keeps the panel's top edge at its anchor, including after the content height changes.
    private func reposition() {
        switch anchor {
        case .statusItem(let rect):
            let center = NSPoint(x: rect.midX, y: rect.midY)
            let area = (NSScreen.screens.first { $0.frame.contains(center) } ?? NSScreen.main)?.visibleFrame ?? rect
            let width = panel.frame.width
            let x = min(max(rect.midX - width / 2, area.minX + 6), area.maxX - width - 6)
            let top = rect.minY - 2
            panel.setFrameTopLeftPoint(NSPoint(x: x, y: top))
        case .topRight:
            let area = NSScreen.screens.first?.visibleFrame ?? .zero
            let top = area.maxY - 8
            panel.setFrameTopLeftPoint(NSPoint(x: area.maxX - panel.frame.width - 12, y: top))
        }
    }
}
