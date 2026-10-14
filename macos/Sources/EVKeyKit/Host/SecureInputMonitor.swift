import AppKit
import Carbon.HIToolbox
import IOKit

/// Secure Input (a password field has focus, or an app turned it on). While it is on, the core stops converting keys.
final class SecureInputMonitor {
    private let core: CoreSession
    private let publish: (String?) -> Void
    private let lock = NSLock()
    // Guarded by `lock`: the tap thread and the main-thread timer both refresh.
    private var isOn = false
    private var holder: String?

    /// `publish` gets nil when Secure Input is off, "" when it is on and the holder is unknown, else the holder's name.
    init(core: CoreSession, publish: @escaping (String?) -> Void) {
        self.core = core
        self.publish = publish
    }

    /// Cheap check for the tap thread: runs a full refresh only when Secure Input flipped.
    func checkOnKeyDown() {
        lock.lock()
        let flipped = IsSecureEventInputEnabled() != isOn
        lock.unlock()
        if flipped { refresh() }
    }

    /// Full check, run every 300 ms.
    func refresh() {
        let on = IsSecureEventInputEnabled()
        let name = on ? Self.holderName() : nil
        lock.lock()
        let changed = on != isOn || name != holder
        isOn = on
        holder = name
        lock.unlock()
        guard changed else { return }
        core.setSecure(on)
        publish(on ? (name ?? "") : nil)
    }

    /// Name of the app holding Secure Input, from `kCGSSessionSecureInputPID` in the IORegistry console users.
    static func holderName() -> String? {
        let root = IORegistryGetRootEntry(kIOMainPortDefault)
        defer { _ = IOObjectRelease(root) }
        let users = IORegistryEntryCreateCFProperty(root, "IOConsoleUsers" as CFString, kCFAllocatorDefault, 0)?
            .takeRetainedValue()
        guard let sessions = users as? [[String: Any]] else { return nil }
        for session in sessions {
            guard let pid = (session["kCGSSessionSecureInputPID"] as? NSNumber)?.int32Value else { continue }
            return NSRunningApplication(processIdentifier: pid)?.localizedName
        }
        return nil
    }
}
