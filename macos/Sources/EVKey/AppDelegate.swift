import AppKit
import EVKeyKit

/// Owns the app's surfaces: menu bar item, popup, control panel and secure-input banner.
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: StatusItemController?
    private var banner: BannerController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // claim() is true when another copy is already running; that copy has been asked to open Bảng điều khiển, so this one quits.
        if SingleInstance.claim() {
            NSApp.terminate(nil)
            return
        }

        GlassAppearance.shared.applyTheme()
        AppState.shared.start()
        statusItem = StatusItemController()
        banner = BannerController()
        banner?.start()

        SingleInstance.observeActivationRequests {
            SettingsWindowController.shared.show(tab: nil)
        }

        if DebugOptions.opensSettingsAtLaunch {
            SettingsWindowController.shared.show(tab: nil)
        }
        if DebugOptions.show == "popup" {
            PopupController.shared.showSticky()
        }
    }
}
