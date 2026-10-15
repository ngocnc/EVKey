import AppKit
import QuartzCore
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
        // Chính EVKey giữ Secure Input (ô API key Gemini đang được gõ): banner chỉ che mất ô nhập, không có ích.
        let holder = AppState.shared.secureHolder
        let ownName = Bundle.main.object(forInfoDictionaryKey: "CFBundleName") as? String ?? ProcessInfo.processInfo.processName
        guard (holder != nil && holder != ownName) || DebugOptions.show == "banner" else {
            fadeOut()
            return
        }
        let size = hosting.fittingSize
        let area = Self.viewport()
        let frame = NSRect(
            x: area.midX - size.width / 2, y: area.minY + 32, width: size.width, height: size.height)
        guard !panel.isVisible || hiding else {
            panel.setFrame(frame, display: true)
            return
        }
        // Slides up from below while fading in; a banner caught mid-fade-out slides back from where it is.
        hiding = false
        if !panel.isVisible {
            panel.alphaValue = 0
            panel.setFrame(frame.offsetBy(dx: 0, dy: -Self.slide), display: true)
        }
        panel.orderFrontRegardless()
        NSAnimationContext.runAnimationGroup { context in
            context.duration = 0.35
            context.timingFunction = CAMediaTimingFunction(controlPoints: 0.2, 1.2, 0.4, 1)
            panel.animator().alphaValue = 1
            panel.animator().setFrame(frame, display: true)
        }
    }

    /// How far the banner travels while it enters and leaves, in points.
    private static let slide: CGFloat = 18

    /// The area to centre the banner in: the web view or window that holds the focused control, so apps with a side
    /// bar (browsers) are centred on their content and not on the screen. Falls back to the screen's visible area.
    private static func viewport() -> NSRect {
        let screen = (NSScreen.main ?? NSScreen.screens.first)?.visibleFrame ?? .zero
        guard AXIsProcessTrusted(), let primary = NSScreen.screens.first else { return screen }
        guard let pid = NSWorkspace.shared.frontmostApplication?.processIdentifier else { return screen }
        let app = AXUIElementCreateApplication(pid)
        AXUIElementSetMessagingTimeout(app, 0.2)
        // Chromium and Electron build their web accessibility tree only when asked; without it the page has no web area.
        AXUIElementSetAttributeValue(app, "AXManualAccessibility" as CFString, kCFBooleanTrue)
        guard let focused: AXUIElement = attribute(app, kAXFocusedUIElementAttribute) else { return screen }
        // Walk up to the web area (page content); a native app has none, so use its window.
        var node: AXUIElement? = focused
        var window: AXUIElement?
        while let current = node {
            AXUIElementSetMessagingTimeout(current, 0.2)
            let role: String? = attribute(current, kAXRoleAttribute)
            if role == "AXWebArea", let rect = frame(current, primary) { return rect.intersection(screen).isNull ? screen : rect }
            if role == kAXWindowRole { window = current; break }
            node = attribute(current, kAXParentAttribute) as AXUIElement?
        }
        if let window, let rect = frame(window, primary), !rect.intersection(screen).isNull { return rect }
        return screen
    }

    private static func attribute<T>(_ element: AXUIElement, _ name: String) -> T? {
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success else { return nil }
        return value as? T
    }

    /// The element's frame in AppKit coordinates (AX uses a top-left origin on the primary screen).
    private static func frame(_ element: AXUIElement, _ primary: NSScreen) -> NSRect? {
        guard let pos: AXValue = attribute(element, kAXPositionAttribute),
            let size: AXValue = attribute(element, kAXSizeAttribute)
        else { return nil }
        var p = CGPoint.zero
        var s = CGSize.zero
        guard AXValueGetValue(pos, .cgPoint, &p), AXValueGetValue(size, .cgSize, &s), s.width > 0 else { return nil }
        return NSRect(x: p.x, y: primary.frame.maxY - (p.y + s.height), width: s.width, height: s.height)
    }

    private func fadeOut() {
        guard panel.isVisible, !hiding else { return }
        hiding = true
        NSAnimationContext.runAnimationGroup({ context in
            context.duration = 0.22
            context.timingFunction = CAMediaTimingFunction(name: .easeIn)
            panel.animator().alphaValue = 0
            panel.animator().setFrame(panel.frame.offsetBy(dx: 0, dy: -Self.slide), display: true)
        }, completionHandler: { [weak self] in
            guard let self, hiding else { return }
            hiding = false
            panel.orderOut(nil)
        })
    }
}
