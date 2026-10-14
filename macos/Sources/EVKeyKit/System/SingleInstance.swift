import AppKit

/// Keeps one EVKey running. A second launch asks the first copy to open Bảng điều khiển, then quits.
public enum SingleInstance {
    static let openSettingsNotification = Notification.Name("com.chrisnguyen.evkey.openSettings")

    /// Returns true when another copy is running (it was asked to open its control panel). The caller quits.
    public static func claim() -> Bool {
        let me = ProcessInfo.processInfo.processIdentifier
        let others = NSRunningApplication
            .runningApplications(withBundleIdentifier: ForegroundWatcher.ownBundleID)
            .filter { $0.processIdentifier != me }
        guard !others.isEmpty else { return false }
        DistributedNotificationCenter.default().postNotificationName(
            openSettingsNotification, object: nil, userInfo: nil, deliverImmediately: true)
        return true
    }

    /// Calls `handler` on the main actor when a second launch asks this copy to open the control panel.
    public static func observeActivationRequests(_ handler: @escaping @MainActor () -> Void) {
        DistributedNotificationCenter.default().addObserver(
            forName: openSettingsNotification, object: nil, queue: .main
        ) { _ in
            Task { @MainActor in handler() }
        }
    }
}
