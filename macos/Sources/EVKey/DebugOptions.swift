import Foundation

/// Launch switches for screenshots. `EVKEY_DEBUG_SHOW=popup|settings|banner` opens that surface at launch;
/// `EVKEY_DEBUG_BANNER=<app name>` sets the app named in the debug banner.
enum DebugOptions {
    static let show: String? = ProcessInfo.processInfo.environment["EVKEY_DEBUG_SHOW"]
    static let bannerApp: String? = ProcessInfo.processInfo.environment["EVKEY_DEBUG_BANNER"]
    static let tab: String? = ProcessInfo.processInfo.environment["EVKEY_DEBUG_TAB"]

    /// `--settings` or `EVKEY_DEBUG_SHOW=settings` opens Bảng điều khiển at launch.
    static var opensSettingsAtLaunch: Bool {
        show == "settings" || CommandLine.arguments.contains("--settings")
    }
}
