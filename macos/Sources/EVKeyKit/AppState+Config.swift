import AppKit
import Foundation

public extension AppState {
    /// Writes `settings` and keeps the macros currently on disk.
    func saveSettings() throws {
        let current = try SettingsStore.load(directory: configDir)
        try SettingsStore.save(directory: configDir, settings: settings, macros: current.macros)
        macros = current.macros
        configError = nil
    }

    /// Validates the rows like the Windows settings window, then writes them and keeps the settings on disk.
    func saveMacros(_ rows: [(key: String, value: String)]) throws {
        var map: [String: String] = [:]
        for row in rows {
            let key = row.key.trimmingCharacters(in: .whitespacesAndNewlines)
            if key.isEmpty { throw CoreError(UIStrings.tr("error.macroEmpty")) }
            if map.updateValue(row.value, forKey: key) != nil {
                throw CoreError(UIStrings.tr("error.macroDuplicate", key))
            }
        }
        let current = try SettingsStore.load(directory: configDir)
        try SettingsStore.save(directory: configDir, settings: current.settings, macros: map)
        macros = map
    }

    /// Restores the engine defaults and removes all macros, as the Windows reset does.
    func resetConfig() throws {
        try SettingsStore.save(directory: configDir, settings: EVSettings.defaults, macros: [:])
        reload()
    }

    func setAutostart(_ on: Bool) throws {
        try LaunchAgent.set(on)
        autostart = LaunchAgent.isInstalled
    }

    func openConfigDir() {
        try? FileManager.default.createDirectory(atPath: configDir, withIntermediateDirectories: true)
        NSWorkspace.shared.open(URL(fileURLWithPath: configDir))
    }

    func openGitHub() {
        guard let url = URL(string: "https://github.com/ngocnc/EVKey") else { return }
        NSWorkspace.shared.open(url)
    }

    func requestAccessibility() {
        Accessibility.prompt()
    }

    func openAccessibilitySettings() {
        Accessibility.openSettings()
    }

    /// Regular (Dock) apps for the excluded-app picker, sorted by name. EVKey is left out.
    func runningApps() -> [RunningAppInfo] {
        var seen = Set<String>()
        return NSWorkspace.shared.runningApplications
            .filter { $0.activationPolicy == .regular }
            .compactMap { app -> RunningAppInfo? in
                guard let id = app.bundleIdentifier, id != ForegroundWatcher.ownBundleID,
                      seen.insert(id).inserted else { return nil }
                return RunningAppInfo(bundleID: id, name: app.localizedName ?? id)
            }
            .sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
    }
}
