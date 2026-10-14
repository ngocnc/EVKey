import Foundation

/// Autostart as a LaunchAgent plist in `~/Library/LaunchAgents`, launched with `--hidden` (no settings window).
enum LaunchAgent {
    static let label = "com.chrisnguyen.evkey"

    static var plistURL: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/LaunchAgents/\(label).plist")
    }

    static var isInstalled: Bool {
        FileManager.default.fileExists(atPath: plistURL.path)
    }

    static func set(_ on: Bool) throws {
        let fm = FileManager.default
        guard on else {
            if isInstalled { try fm.removeItem(at: plistURL) }
            return
        }
        let executable = Bundle.main.executablePath ?? CommandLine.arguments[0]
        let plist: [String: Any] = [
            "Label": label,
            "ProgramArguments": [executable, "--hidden"],
            "RunAtLoad": true,
        ]
        try fm.createDirectory(at: plistURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        let data = try PropertyListSerialization.data(fromPropertyList: plist, format: .xml, options: 0)
        try data.write(to: plistURL, options: .atomic)
    }
}
