import Foundation

/// A newer EVKey release that ships a macOS dmg.
public struct UpdateRelease: Equatable {
    /// Tag without the `v` prefix, e.g. `2.3.0`.
    public let version: String
    /// Release body with the CI-written `SHA256:` line removed.
    public let notes: String
    public let downloadURL: URL
}

/// Parsing and version comparison for the GitHub Releases check. No network.
public enum UpdateCheck {
    /// True when `candidate` is a newer dotted version, compared numerically (`v` prefix allowed).
    /// Anything unparsable is never newer.
    public static func isNewer(_ candidate: String, than current: String) -> Bool {
        guard let a = parts(candidate), let b = parts(current) else { return false }
        return b.lexicographicallyPrecedes(a)
    }

    /// The release body without the `SHA256:` line that CI appends for the Windows installer.
    public static func notes(fromBody body: String) -> String {
        body.components(separatedBy: .newlines)
            .filter { !$0.trimmingCharacters(in: .whitespaces).hasPrefix("SHA256:") }
            .joined(separator: "\n")
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }

    /// The release from a GitHub `releases/latest` response when it is newer than `current` and has a dmg.
    /// nil means no update to offer. Throws when `data` is not a release.
    public static func latest(fromJSON data: Data, current: String) throws -> UpdateRelease? {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let payload = try decoder.decode(Payload.self, from: data)
        guard isNewer(payload.tagName, than: current) else { return nil }
        guard
            let asset = payload.assets.first(where: { $0.name.hasPrefix("EVKey-") && $0.name.hasSuffix("-universal.dmg") }),
            let url = URL(string: asset.browserDownloadUrl)
        else { return nil }
        return UpdateRelease(
            version: stripV(payload.tagName),
            notes: notes(fromBody: payload.body ?? ""),
            downloadURL: url)
    }

    private struct Payload: Decodable {
        struct Asset: Decodable {
            let name: String
            let browserDownloadUrl: String
        }

        let tagName: String
        let body: String?
        let assets: [Asset]
    }

    private static func stripV(_ version: String) -> String {
        version.hasPrefix("v") ? String(version.dropFirst()) : version
    }

    private static func parts(_ version: String) -> [Int]? {
        let values = stripV(version).split(separator: ".", omittingEmptySubsequences: false).map { Int($0) }
        guard values.allSatisfy({ $0 != nil }) else { return nil }
        return values.compactMap { $0 }
    }
}
