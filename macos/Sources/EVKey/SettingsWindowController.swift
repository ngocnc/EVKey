import AppKit
import EVKeyKit
import SwiftUI

/// Selected sidebar tab, shared between the window and callers that want a specific tab.
final class SettingsNavigation: ObservableObject {
    /// `EVKEY_DEBUG_TAB=<tab>` mở thẳng mục đó, để chụp màn hình từng trang.
    @Published var tab: SettingsTab = DebugOptions.tab.flatMap(SettingsTab.init(rawValue:)) ?? .typing
}

/// Owns the Bảng điều khiển window. It is created once and reused; closing only hides it.
@MainActor
final class SettingsWindowController {
    static let shared = SettingsWindowController()

    private let navigation = SettingsNavigation()
    private let window: NSWindow

    private init() {
        let hosting = NSHostingController(rootView: SettingsView(navigation: navigation))
        window = NSWindow(contentViewController: hosting)
        window.styleMask = [.titled, .closable, .miniaturizable, .fullSizeContentView]
        window.titlebarAppearsTransparent = true
        window.isOpaque = false
        window.backgroundColor = .clear
        window.setContentSize(NSSize(width: 760, height: 540))
        window.isReleasedWhenClosed = false
        window.center()
        window.level = .floating
    }

    /// Shows the window, optionally jumping to a tab. `nil` keeps the current tab.
    func show(tab: SettingsTab?) {
        if let tab { navigation.tab = tab }
        // Refreshed on every show so the title follows a live language change.
        window.title = UIStrings.tr("window.title")
        NSApp.activate(ignoringOtherApps: true)
        let opening = !window.isVisible
        if opening { window.alphaValue = 0 }
        window.makeKeyAndOrderFront(nil)
        if opening {
            NSAnimationContext.runAnimationGroup { context in
                context.duration = 0.2
                window.animator().alphaValue = 1
            }
        }
    }

    /// Whether the window currently floats above others. Read by the pin button so its state matches the real level.
    var isPinned: Bool { window.level == .floating }

    /// Keeps the window above other windows, or returns it to the normal level. Not persisted.
    func setPinned(_ pinned: Bool) {
        window.level = pinned ? .floating : .normal
    }
}
