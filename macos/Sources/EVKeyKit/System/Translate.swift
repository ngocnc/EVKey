import Foundation

/// The Google Translate endpoint the Windows app uses (`crates/app/src/translate.rs`): URL, response parsing, target choice.
public enum Translate {
    public struct Translation: Equatable {
        public let text: String
        /// Source language as Google reports it, e.g. "en" or "vi".
        public let detected: String
    }

    public static let maxChars = 4000
    static let endpoint = "https://translate.googleapis.com/translate_a/single"

    /// Query values are percent-encoded by hand: `URLComponents` leaves `+` and `&` unescaped, and `+` would read as a space.
    public static func url(text: String, target: String) -> URL? {
        let unreserved = CharacterSet(charactersIn: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~")
        func encode(_ value: String) -> String {
            value.addingPercentEncoding(withAllowedCharacters: unreserved) ?? ""
        }
        let query = [
            ("client", "gtx"), ("sl", "auto"), ("tl", target), ("dt", "t"), ("q", text),
        ].map { "\($0.0)=\(encode($0.1))" }.joined(separator: "&")
        return URL(string: "\(endpoint)?\(query)")
    }

    /// The response is `[[["translated","orig",...],...], null, "<detected>", ...]`: join element 0 of each segment.
    public static func parse(_ data: Data) -> Translation? {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [Any],
              json.count > 2,
              let segments = json[0] as? [Any],
              let detected = json[2] as? String else { return nil }
        let text = segments.compactMap { ($0 as? [Any])?.first as? String }.joined()
        return Translation(text: text, detected: detected)
    }

    /// Vietnamese source goes to English; anything else goes to Vietnamese.
    public static func target(detected: String) -> String {
        language(of: detected) == "vi" ? "en" : "vi"
    }

    /// The base language code: "vi-VN" becomes "vi".
    public static func language(of detected: String) -> String {
        String(detected.lowercased().prefix { $0 != "-" })
    }
}
