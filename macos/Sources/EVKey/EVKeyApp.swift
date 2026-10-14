import AppKit

/// Entry point. A plain AppKit main keeps the menu bar item, popup, control panel and banner under direct control.
@main
@MainActor
enum EVKeyApp {
    private static let delegate = AppDelegate()

    static func main() {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        app.delegate = delegate
        app.run()
    }
}
