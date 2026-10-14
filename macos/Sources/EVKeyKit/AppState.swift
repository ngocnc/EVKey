import AppKit
import CEVKeyCore
import Foundation

/// A running app: its bundle ID and display name.
public struct RunningAppInfo: Hashable, Identifiable {
    public let bundleID: String
    public let name: String

    public var id: String { bundleID }

    public init(bundleID: String, name: String) {
        self.bundleID = bundleID
        self.name = name
    }
}

/// Shared state for the menu bar, popup, settings window and banner. Owns the core session, the keyboard tap and
/// the watchers. Config edits go through `AppState+Config.swift`.
@MainActor
public final class AppState: ObservableObject {
    public static let shared = AppState()

    @Published public var isEnglish = false
    @Published public var settings = EVSettings.defaults
    @Published public var macros: [String: String] = [:]
    @Published public var configError: String?
    @Published public var accessibilityTrusted = false
    /// nil = Secure Input off; "" = on with an unknown holder; otherwise the holder's name.
    @Published public var secureHolder: String?
    @Published public var lastApp: RunningAppInfo?
    @Published public var autostart = LaunchAgent.isInstalled

    public var configDir: String { SettingsStore.defaultDirectory ?? "" }
    public var version: String { String(cString: evkey_version()) }

    private var core: CoreSession?
    private var tapHost: EventTapHost?
    private var secure: SecureInputMonitor?
    private var secureTimer: Timer?
    private let foreground = ForegroundWatcher()

    private init() {}

    /// Starts the tap, the watchers and polling. Idempotent.
    public func start() {
        guard core == nil, let session = CoreSession(directory: nil) else { return }
        core = session
        reload()
        isEnglish = session.isEnglish

        let monitor = SecureInputMonitor(core: session) { holder in
            Task { @MainActor in AppState.shared.secureHolder = holder }
        }
        secure = monitor
        secureTimer = Timer.scheduledTimer(withTimeInterval: 0.3, repeats: true) { _ in
            monitor.refresh()
        }

        let host = EventTapHost(
            core: session,
            secure: monitor,
            onHotkeyToggle: { Task { @MainActor in AppState.shared.syncMode(); AppState.shared.playToggleSound() } },
            onAccessibility: { trusted in Task { @MainActor in AppState.shared.setTrusted(trusted) } }
        )
        tapHost = host
        host.start()

        foreground.start(core: session) { app in
            Task { @MainActor in
                AppState.shared.lastApp = app
                // set_foreground may have restored this app's remembered Mode (remember_app_mode).
                AppState.shared.syncMode()
            }
        }
    }

    public func setEnglish(_ on: Bool) {
        guard core?.isEnglish != on else { return }
        core?.setEnglish(on)
        syncMode()
        playToggleSound()
    }

    public func toggleMode() {
        core?.toggle()
        syncMode()
        playToggleSound()
    }

    /// Plays the V/E cue for a switch the user made. Automatic per-app restores never call this.
    func playToggleSound() {
        guard settings.toggleSound else { return }
        NSSound(named: isEnglish ? "Pop" : "Tink")?.play()
    }

    /// Reloads settings and macros from disk. A malformed file keeps the current values and sets `configError`.
    public func reload() {
        do {
            let loaded = try SettingsStore.load(directory: configDir)
            settings = loaded.settings
            macros = loaded.macros
            configError = nil
        } catch {
            configError = (error as? CoreError)?.message ?? error.localizedDescription
        }
    }

    func syncMode() {
        if let core { isEnglish = core.isEnglish }
    }

    func setTrusted(_ trusted: Bool) {
        if accessibilityTrusted != trusted { accessibilityTrusted = trusted }
    }
}
