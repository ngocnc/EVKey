import Foundation
import Security

/// The user's Gemini API key, kept as a generic password in the login Keychain (the Windows build uses Credential Manager).
/// The key is never written to config files or logs.
public enum GeminiKeychain {
    static let service = "EVKey/gemini"
    static let account = "gemini"

    private static var query: [String: Any] {
        [kSecClass as String: kSecClassGenericPassword, kSecAttrService as String: service, kSecAttrAccount as String: account]
    }

    /// Whether a key is saved. Reads attributes only, so it never asks for Keychain access.
    public static func exists() -> Bool {
        var q = query
        q[kSecReturnAttributes as String] = true
        return SecItemCopyMatching(q as CFDictionary, nil) == errSecSuccess
    }

    /// The saved key, nil when none or it is empty.
    public static func load() -> String? {
        var q = query
        q[kSecReturnData as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var item: CFTypeRef?
        guard SecItemCopyMatching(q as CFDictionary, &item) == errSecSuccess, let data = item as? Data else { return nil }
        let key = String(decoding: data, as: UTF8.self)
        return key.isEmpty ? nil : key
    }

    /// Adds the key, or replaces the one already saved.
    public static func save(_ key: String) throws {
        let data = Data(key.utf8)
        var status = SecItemUpdate(query as CFDictionary, [kSecValueData as String: data] as CFDictionary)
        if status == errSecItemNotFound {
            var q = query
            q[kSecValueData as String] = data
            status = SecItemAdd(q as CFDictionary, nil)
        }
        guard status == errSecSuccess else { throw GeminiError(failure("gemini.error.keychainSave", status)) }
    }

    /// Deleting a missing key succeeds, so callers can clear it without checking first.
    public static func delete() throws {
        let status = SecItemDelete(query as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw GeminiError(failure("gemini.error.keychainDelete", status))
        }
    }

    private static func failure(_ key: String, _ status: OSStatus) -> String {
        let detail = SecCopyErrorMessageString(status, nil) as String? ?? "OSStatus \(status)"
        return "\(UIStrings.tr(key)): \(detail)"
    }
}
