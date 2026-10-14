import Foundation

/// Backup, config import and macro import for the System and Macro tabs. Every write goes through the core, which
/// validates first, so a bad file leaves the current settings untouched.
public extension AppState {
    /// Writes settings and macros to `url` as one TOML file, the same format as the Windows export.
    func exportConfig(to url: URL) throws {
        let text = try ConfigBackup.export(directory: configDir)
        do {
            try text.write(to: url, atomically: true, encoding: .utf8)
        } catch {
            throw CoreError(UIStrings.misc("backup.writeFailed", error.localizedDescription))
        }
    }

    /// Replaces settings and macros with the exported file at `url`.
    func importConfig(from url: URL) throws {
        let text = try readText(url)
        try ConfigBackup.importConfig(text, directory: configDir)
        reload()
    }

    /// Imports a UniKey or OpenKey macro file. `replace` drops the current macros first; otherwise imported keys
    /// overwrite same-named macros and the rest are kept. Returns the number of macros in the file.
    func importMacros(from url: URL, replace: Bool) throws -> Int {
        let data: Data
        do {
            data = try Data(contentsOf: url)
        } catch {
            throw CoreError(UIStrings.misc("backup.readFailed", error.localizedDescription))
        }
        let imported = try MacroFile.parse(data)
        try ConfigBackup.importMacros(imported, replace: replace, directory: configDir)
        reload()
        return imported.count
    }

    private func readText(_ url: URL) throws -> String {
        do {
            return try String(contentsOf: url, encoding: .utf8)
        } catch {
            throw CoreError(UIStrings.misc("backup.readFailed", error.localizedDescription))
        }
    }
}
