import AppKit
import Combine
import EVKeyKit
import SwiftUI

/// Shows the Secure Input banner at the bottom centre of the active screen while Secure Input is on.
/// The panel ignores the mouse and never activates, so it cannot steal focus or block clicks.
@MainActor
final class BannerController {
    private let hosting: NSHostingView<BannerHost>
    private let panel: NSPanel
    private var cancellable: AnyCancellable?
    private var hiding = false

    init() {
        hosting = NSHostingView(rootView: BannerHost())
        panel = NSPanel(
            contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: true)
        panel.contentView = hosting
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = false
        panel.hidesOnDeactivate = false
        panel.isReleasedWhenClosed = false
        panel.ignoresMouseEvents = true
        panel.level = .statusBar
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary, .ignoresCycle]
    }

    /// Shows or hides the banner now, then follows every state change.
    func start() {
        // Deferred one turn so the panel sizes against the new state rather than the old one.
        cancellable = AppState.shared.objectWillChange.sink { [weak self] _ in
            DispatchQueue.main.async { self?.refresh() }
        }
        refresh()
    }

    private func refresh() {
        guard AppState.shared.secureHolder != nil || DebugOptions.show == "banner" else {
            fadeOut()
            return
        }
        let size = hosting.fittingSize
        // The screen that holds the key window is the one the user is working on.
        let area = (NSScreen.main ?? NSScreen.screens.first)?.visibleFrame ?? .zero
        let frame = NSRect(
            x: area.midX - size.width / 2, y: area.minY + 48, width: size.width, height: size.height)
        panel.setFrame(frame, display: true)
        guard !panel.isVisible || hiding else { return }
        hiding = false
        panel.alphaValue = panel.isVisible ? panel.alphaValue : 0
        panel.orderFrontRegardless()
        NSAnimationContext.runAnimationGroup { context in
            context.duration = 0.25
            panel.animator().alphaValue = 1
        }
    }

    private func fadeOut() {
        guard panel.isVisible, !hiding else { return }
        hiding = true
        NSAnimationContext.runAnimationGroup({ context in
            context.duration = 0.2
            panel.animator().alphaValue = 0
        }, completionHandler: { [weak self] in
            guard let self, hiding else { return }
            hiding = false
            panel.orderOut(nil)
        })
    }
}
