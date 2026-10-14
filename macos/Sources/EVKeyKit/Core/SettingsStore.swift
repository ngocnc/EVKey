import CEVKeyCore
import Foundation

/// Error text shown to the user: the core's message, or ours.
public struct CoreError: LocalizedError, Equatable {
    public let message: String

    public init(_ message: String) {
        self.message = message
    }

    public var errorDescription: String? { message }
}

/// Settings and macros as the core reads them from `config.toml` and `macros.toml`.
public struct LoadedSettings: Equatable {
    public var settings: EVSettings
    public var macros: [String: String]
}

/// JSON bridge to the core's settings files. The core parses, validates and writes; Swift only moves JSON.
enum SettingsStore {
    /// Default settings directory (`~/Library/Application Support/EVKey`).
    static var defaultDirectory: String? {
        takeString(evkey_default_dir())
    }

    /// A malformed file throws, so callers keep their previous settings.
    static func load(directory: String) throws -> LoadedSettings {
        let json = try jsonString(directory.withCString { evkey_settings_load_json($0) })
        let doc = try JSONDecoder().decode(LoadDoc.self, from: Data(json.utf8))
        if let error = doc.error { throw CoreError(error) }
        return LoadedSettings(settings: doc.settings, macros: doc.macros)
    }

    /// Writes both files. The core validates the hotkey, banner values and macro keys.
    static func save(directory: String, settings: EVSettings, macros: [String: String]) throws {
        let json = try JSONEncoder().encode(SaveDoc(settings: settings, macros: macros))
        let text = String(decoding: json, as: UTF8.self)
        let failure = directory.withCString { dir in text.withCString { evkey_settings_save_json(dir, $0) } }
        if let message = takeString(failure) { throw CoreError(message) }
    }

    /// Engine defaults. A missing settings directory means `Settings::default()`, so Swift never repeats them.
    static func engineDefaults() -> EVSettings {
        do {
            return try load(directory: "/nonexistent/evkey-defaults").settings
        } catch {
            fatalError("EVKey core defaults unavailable: \(error)")
        }
    }

    private static func jsonString(_ pointer: UnsafeMutablePointer<CChar>?) throws -> String {
        guard let json = takeString(pointer) else { throw CoreError(UIStrings.tr("error.configRead")) }
        return json
    }

    private static func takeString(_ pointer: UnsafeMutablePointer<CChar>?) -> String? {
        guard let pointer else { return nil }
        defer { evkey_string_free(pointer) }
        return String(cString: pointer)
    }

    private struct LoadDoc: Decodable {
        let settings: EVSettings
        let macros: [String: String]
        let error: String?
    }

    private struct SaveDoc: Encodable {
        let settings: EVSettings
        let macros: [String: String]
    }
}
