import Foundation

public extension AppState {
    /// `EVKey/<version>`, sent to Google as the User-Agent.
    var userAgent: String { "EVKey/\(version)" }

    /// Tests `key` with Gemini, then saves it to the Keychain. Throws a message that is ready to show.
    func saveGeminiKey(_ key: String) async throws {
        let key = key.trimmingCharacters(in: .whitespacesAndNewlines)
        if key.isEmpty { throw GeminiError(UIStrings.tr("gemini.error.emptyKey")) }
        try await Gemini.testKey(key, userAgent: userAgent)
        try GeminiKeychain.save(key)
        geminiKeySaved = true
    }

    func deleteGeminiKey() throws {
        try GeminiKeychain.delete()
        geminiKeySaved = false
    }

    /// Runs a Gemini action with the saved key. Reads the Keychain here, so a locked item asks at the moment of use.
    func runGemini(_ action: GeminiAction, text: String) async throws -> String {
        guard let key = GeminiKeychain.load() else { throw GeminiError(UIStrings.tr("gemini.error.noKey")) }
        return try await Gemini.run(action: action, text: text, key: key, userAgent: userAgent)
    }
}
