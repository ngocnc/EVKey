import CEVKeyCore
import Foundation

/// Backup and import of the user's data, over the core's settings JSON so no field is lost (the typed `EVSettings` only
/// knows the fields the Swift UI edits). The export is the Windows file: `format_version`, `[settings]`, `[macros]`.
enum ConfigBackup {
    static let formatVersion = 1

    /// Settings and macros as one TOML document. Holds no secrets: the Gemini key is not part of the config files.
    static func export(directory: String) throws -> String {
        let doc = try load(directory: directory)
        return TomlCodec.encode([
            "format_version": .int(formatVersion),
            "settings": doc.settings,
            "macros": doc.macros,
        ])
    }

    /// Replaces settings and macros with an exported document. The core validates before it writes anything, so a bad
    /// file leaves the current files untouched.
    static func importConfig(_ text: String, directory: String) throws {
        let root = try TomlCodec.decode(text)
        if let extra = root.keys.sorted().first(where: { !["format_version", "settings", "macros"].contains($0) }) {
            throw CoreError("unknown field `\(extra)`")
        }
        guard case .int(let version)? = root["format_version"] else { throw CoreError("missing field `format_version`") }
        if version != formatVersion {
            throw CoreError("unsupported config format_version \(version) (expected \(formatVersion))")
        }
        guard case .object(let settings)? = root["settings"] else { throw CoreError("missing table [settings]") }
        guard case .object(let macros)? = root["macros"] else { throw CoreError("missing table [macros]") }
        for (key, value) in macros {
            guard case .string = value else { throw CoreError("macros: \"\(key)\" must be a string") }
            if key.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                throw CoreError("macros: an abbreviation cannot be empty")
            }
        }
        try save(directory: directory, settings: .object(settings), macros: .object(macros))
    }

    /// Adds `imported` to the macros on disk (same keys are overwritten), or replaces them all when `replace` is set.
    static func importMacros(
        _ imported: [(key: String, text: String)], replace: Bool, directory: String
    ) throws {
        var seen = Set<String>()
        var incoming: [String: ConfigValue] = [:]
        for item in imported {
            if !seen.insert(item.key).inserted { throw CoreError(UIStrings.tr("error.macroDuplicate", item.key)) }
            incoming[item.key] = .string(item.text)
        }
        let doc = try load(directory: directory)
        var macros: [String: ConfigValue] = [:]
        if !replace, case .object(let current) = doc.macros { macros = current }
        macros.merge(incoming) { _, new in new }
        try save(directory: directory, settings: doc.settings, macros: .object(macros))
    }

    // MARK: Core JSON

    private struct LoadDoc: Decodable {
        let settings: ConfigValue
        let macros: ConfigValue
        let error: String?
    }

    private struct SaveDoc: Encodable {
        let settings: ConfigValue
        let macros: ConfigValue
    }

    private static func load(directory: String) throws -> LoadDoc {
        let pointer = directory.withCString { evkey_settings_load_json($0) }
        guard let pointer else { throw CoreError(UIStrings.tr("error.configRead")) }
        defer { evkey_string_free(pointer) }
        let doc = try JSONDecoder().decode(LoadDoc.self, from: Data(String(cString: pointer).utf8))
        if let error = doc.error { throw CoreError(error) }
        return doc
    }

    private static func save(directory: String, settings: ConfigValue, macros: ConfigValue) throws {
        let json = try JSONEncoder().encode(SaveDoc(settings: settings, macros: macros))
        let text = String(decoding: json, as: UTF8.self)
        let failure = directory.withCString { dir in text.withCString { evkey_settings_save_json(dir, $0) } }
        guard let failure else { return }
        defer { evkey_string_free(failure) }
        throw CoreError(String(cString: failure))
    }
}
