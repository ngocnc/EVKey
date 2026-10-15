import AppKit
import EVKeyKit
import Foundation

/// Failures while checking or installing an update. Shown in the update window or on the Info tab.
enum UpdateError: LocalizedError {
    case http(Int)
    case signature
    case notWritable(String)
    case tool(String)

    var errorDescription: String? {
        switch self {
        case .http(let code):
            return "HTTP \(code)"
        case .signature:
            return UIStrings.tr("update.signature")
        case .notWritable(let path):
            return UIStrings.tr("update.notWritable", path)
        case .tool(let output):
            return output
        }
    }
}

/// Checks GitHub for a newer EVKey, offers it in the update window and installs it on request.
/// Schedule: about 20 s after launch, then every 24 h, only while `autoCheckUpdates` is on.
@MainActor
final class AppUpdater: ObservableObject {
    static let shared = AppUpdater()

    enum Status: Equatable {
        case idle
        case checking
        case latest
        case failed(String)
    }

    /// Result of the last check, shown on the Info tab.
    @Published private(set) var status: Status = .idle
    /// The newer release the update window shows.
    @Published private(set) var pending: UpdateRelease?
    @Published private(set) var installing = false
    @Published private(set) var installError: String?

    private static let latestURL = URL(string: "https://api.github.com/repos/ngocnc/EVKey/releases/latest")!
    private static let firstCheckDelay: UInt64 = 20 * 1_000_000_000
    private static let checkEvery: UInt64 = 24 * 60 * 60 * 1_000_000_000

    private init() {}

    func start() {
        Task {
            try? await Task.sleep(nanoseconds: Self.firstCheckDelay)
            while true {
                if AppState.shared.settings.autoCheckUpdates { await check(manual: false) }
                try? await Task.sleep(nanoseconds: Self.checkEvery)
            }
        }
    }

    /// Looks for a newer release. A manual check ignores `autoCheckUpdates` and the skipped version;
    /// an automatic check offers a release only when it was not skipped.
    func check(manual: Bool) async {
        status = .checking
        do {
            var request = URLRequest(url: Self.latestURL)
            request.setValue("EVKey/\(AppState.shared.version)", forHTTPHeaderField: "User-Agent")
            request.setValue("application/vnd.github+json", forHTTPHeaderField: "Accept")
            request.timeoutInterval = 20
            let (data, response) = try await URLSession.shared.data(for: request)
            try Self.ok(response)
            guard let release = try UpdateCheck.latest(fromJSON: data, current: AppState.shared.version) else {
                status = .latest
                return
            }
            status = .idle
            if manual || release.version != AppState.shared.settings.skippedUpdate {
                pending = release
                installError = nil
                UpdateWindowController.shared.show()
            }
        } catch {
            status = .failed(error.localizedDescription)
        }
    }

    /// Downloads and checks the pending release, then replaces this app after it quits. Errors stay in the window.
    func install() async {
        guard let release = pending, !installing else { return }
        installing = true
        installError = nil
        do {
            let newApp = try await Self.stage(release)
            try Self.relaunch(with: newApp)
            NSApp.terminate(nil)
        } catch {
            installError = UIStrings.tr("update.failed", error.localizedDescription)
            installing = false
        }
    }

    /// Downloads the dmg, copies the app out of it and verifies its signature. Runs off the main actor.
    private nonisolated static func stage(_ release: UpdateRelease) async throws -> URL {
        let fm = FileManager.default
        let work = fm.temporaryDirectory.appendingPathComponent("evkey-update-\(UUID().uuidString)")
        try fm.createDirectory(at: work, withIntermediateDirectories: true)
        var staged = false
        // Runs after the detach below (defers unwind in reverse); a successful stage is removed by the relaunch script.
        defer { if !staged { try? fm.removeItem(at: work) } }
        let (file, response) = try await URLSession.shared.download(from: release.downloadURL)
        try ok(response)
        let dmg = work.appendingPathComponent("EVKey.dmg")
        try fm.moveItem(at: file, to: dmg)
        let mount = work.appendingPathComponent("mnt")
        try fm.createDirectory(at: mount, withIntermediateDirectories: true)
        try run("/usr/bin/hdiutil", ["attach", "-nobrowse", "-readonly", "-mountpoint", mount.path, dmg.path])
        defer { _ = try? run("/usr/bin/hdiutil", ["detach", mount.path, "-quiet"]) }
        let newApp = work.appendingPathComponent("new/EVKey.app")
        try fm.createDirectory(at: newApp.deletingLastPathComponent(), withIntermediateDirectories: true)
        try run("/usr/bin/ditto", [mount.appendingPathComponent("EVKey.app").path, newApp.path])
        do {
            try run("/usr/bin/codesign", ["--verify", "--deep", "--strict", newApp.path])
        } catch {
            throw UpdateError.signature
        }
        staged = true
        return newApp
    }

    /// Starts a detached shell that waits for this process to exit, swaps the bundle and opens it again.
    /// Paths go in as positional arguments, never into the script text.
    private static func relaunch(with newApp: URL) throws {
        let target = Bundle.main.bundleURL
        let parent = target.deletingLastPathComponent().path
        let work = newApp.deletingLastPathComponent().deletingLastPathComponent()
        guard FileManager.default.isWritableFile(atPath: parent) else {
            try? FileManager.default.removeItem(at: work)
            throw UpdateError.notWritable(parent)
        }
        // Copy beside the target first, so a failed copy leaves the current app in place.
        let script = """
        while kill -0 "$3" 2>/dev/null; do sleep 0.2; done
        ditto "$1" "$2.new" && rm -rf "$2" && mv "$2.new" "$2"; status=$?
        rm -rf "$4"
        [ $status -eq 0 ] || exit 1
        xattr -dr com.apple.quarantine "$2" 2>/dev/null
        open "$2"
        """
        let shell = Process()
        shell.executableURL = URL(fileURLWithPath: "/bin/sh")
        shell.arguments = ["-c", script, "evkey-update", newApp.path, target.path, String(ProcessInfo.processInfo.processIdentifier), work.path]
        try shell.run()
    }

    private nonisolated static func ok(_ response: URLResponse) throws {
        let code = (response as? HTTPURLResponse)?.statusCode ?? 0
        guard code == 200 else { throw UpdateError.http(code) }
    }

    /// Runs a tool and returns its output; a non-zero exit throws with that output.
    @discardableResult
    private nonisolated static func run(_ path: String, _ args: [String]) throws -> String {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: path)
        process.arguments = args
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe
        try process.run()
        let output = String(decoding: pipe.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
        process.waitUntilExit()
        guard process.terminationStatus == 0 else {
            let text = output.trimmingCharacters(in: .whitespacesAndNewlines)
            throw UpdateError.tool(text.isEmpty ? path : text)
        }
        return output
    }
}
