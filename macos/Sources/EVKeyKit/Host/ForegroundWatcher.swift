import AppKit

/// Reports the frontmost app to the core (for excluded apps). EVKey itself is ignored.
final class ForegroundWatcher {
    static let ownBundleID = "com.chrisnguyen.evkey"

    private var observer: NSObjectProtocol?

    /// `onChange` receives each frontmost app that is not EVKey. Call on the main thread.
    func start(core: CoreSession, onChange: @escaping (RunningAppInfo) -> Void) {
        observer = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification, object: nil, queue: .main
        ) { note in
            let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
            Self.handle(app, core: core, onChange: onChange)
        }
        Self.handle(NSWorkspace.shared.frontmostApplication, core: core, onChange: onChange)
    }

    private static func handle(_ app: NSRunningApplication?, core: CoreSession, onChange: (RunningAppInfo) -> Void) {
        guard let app, let id = app.bundleIdentifier, id != ownBundleID else { return }
        core.setForeground(id)
        onChange(RunningAppInfo(bundleID: id, name: app.localizedName ?? id))
    }
}
