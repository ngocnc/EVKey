import Foundation

/// A text action the action panel runs. The raw value is the id the panel and `crates/app/src/ai.rs` use.
public enum GeminiAction: String, CaseIterable {
    /// Unaccented Vietnamese becomes Vietnamese with tone and vowel marks.
    case addDiacritics = "add_diacritics"
    case fixSpelling = "fix_spelling"
    case rewriteFormal = "rewrite_formal"
    case rewriteShorter = "rewrite_shorter"
    case rewriteFriendly = "rewrite_friendly"
    /// Vietnamese goes to English, anything else to Vietnamese.
    case translate

    /// Prompts are the Windows ones word for word (`AiAction::system_prompt`).
    var systemPrompt: String {
        switch self {
        case .addDiacritics:
            return "The user's text is Vietnamese written without diacritics. Add the Vietnamese tone and vowel marks. Do not change, add or remove any word, its order, its case or its punctuation. Return ONLY the resulting text: no quotes, no labels, no explanations. Keep the line breaks."
        case .fixSpelling:
            return "Fix spelling, typing and grammar mistakes in the user's text. Keep its language, meaning and line breaks. Return ONLY the corrected text: no quotes, no labels, no explanations. Keep the line breaks."
        case .rewriteFormal:
            return "Rewrite the user's text in a formal, polite tone. Keep its language and meaning. Return ONLY the rewritten text: no quotes, no labels, no explanations. Keep the line breaks."
        case .rewriteShorter:
            return "Rewrite the user's text to be shorter and more concise while keeping its language and meaning. Return ONLY the rewritten text: no quotes, no labels, no explanations. Keep the line breaks."
        case .rewriteFriendly:
            return "Rewrite the user's text in a friendly, warm, casual tone. Keep its language and meaning. Return ONLY the rewritten text: no quotes, no labels, no explanations. Keep the line breaks."
        case .translate:
            return "If the user's text is mainly Vietnamese, translate it into English; otherwise translate it into Vietnamese. Return ONLY the translation: no quotes, no labels, no explanations. Keep the line breaks."
        }
    }

    /// UIStrings key of the text shown while the action runs; the three rewrite tones share one.
    public var loadingKey: String {
        switch self {
        case .rewriteFormal, .rewriteShorter, .rewriteFriendly: return "actions.loading.rewrite"
        default: return "actions.loading.\(rawValue)"
        }
    }
}

/// A failure with text that is ready to show the user.
public struct GeminiError: LocalizedError, Equatable {
    public let message: String

    init(_ message: String) { self.message = message }

    public var errorDescription: String? { message }
}

/// Gemini `generateContent` over REST, as `crates/app/src/ai.rs` calls it. The key travels in the `x-goog-api-key` header.
/// https://ai.google.dev/gemini-api/docs/generate-content/get-started.md.txt
public enum Gemini {
    static let api = "https://generativelanguage.googleapis.com/v1beta/models"
    /// Fast, low-cost model for short text transforms. Change here only.
    static let model = "gemini-3.5-flash-lite"
    static let timeout: TimeInterval = 20

    static func requestBody(action: GeminiAction, text: String) -> [String: Any] {
        [
            "systemInstruction": ["parts": [["text": action.systemPrompt]]],
            "contents": [["role": "user", "parts": [["text": text]]]],
            "generationConfig": ["temperature": 0.2],
        ]
    }

    /// The generated text, or a user-facing error when Gemini blocked the text or returned nothing.
    static func parse(_ data: Data) throws -> String {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw GeminiError(UIStrings.tr("gemini.error.invalidResponse"))
        }
        if let feedback = json["promptFeedback"] as? [String: Any], feedback["blockReason"] is String {
            throw GeminiError(UIStrings.tr("gemini.error.refused"))
        }
        let candidates = json["candidates"] as? [[String: Any]]
        let parts = (candidates?.first?["content"] as? [String: Any])?["parts"] as? [[String: Any]] ?? []
        // Parts flagged as "thought" are the model's reasoning, not the answer.
        let text = parts
            .filter { ($0["thought"] as? Bool) != true }
            .compactMap { $0["text"] as? String }
            .joined()
            .trimmingCharacters(in: .whitespacesAndNewlines)
        if text.isEmpty { throw GeminiError(UIStrings.tr("gemini.error.noContent")) }
        return text
    }

    /// User-facing text for a non-200 answer. Other statuses show Gemini's own message.
    static func httpError(status: Int, body: Data) -> GeminiError {
        switch status {
        case 400, 401, 403:
            return GeminiError(UIStrings.tr("gemini.error.invalidKey"))
        case 429:
            return GeminiError(UIStrings.tr("gemini.error.quota"))
        default:
            let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any]
            let detail = (json?["error"] as? [String: Any])?["message"] as? String ?? "HTTP \(status)"
            return GeminiError("\(UIStrings.tr("gemini.error.generic")): \(detail)")
        }
    }

    /// Runs one action on `text`. `userAgent` is the app's `EVKey/<version>`.
    public static func run(action: GeminiAction, text: String, key: String, userAgent: String) async throws -> String {
        if text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            throw GeminiError(UIStrings.tr("gemini.error.noText"))
        }
        var request = try makeRequest(path: "\(api)/\(model):generateContent", key: key, userAgent: userAgent)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONSerialization.data(withJSONObject: requestBody(action: action, text: text))
        return try parse(try await send(request))
    }

    /// Checks the key with a metadata lookup: no generation, no text quota spent.
    public static func testKey(_ key: String, userAgent: String) async throws {
        let request = try makeRequest(path: "\(api)/\(model)", key: key, userAgent: userAgent)
        _ = try await send(request)
    }

    private static func makeRequest(path: String, key: String, userAgent: String) throws -> URLRequest {
        guard let url = URL(string: path) else { throw GeminiError(UIStrings.tr("gemini.error.network")) }
        var request = URLRequest(url: url)
        request.timeoutInterval = timeout
        request.setValue(key, forHTTPHeaderField: "x-goog-api-key")
        request.setValue(userAgent, forHTTPHeaderField: "User-Agent")
        return request
    }

    private static func send(_ request: URLRequest) async throws -> Data {
        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await URLSession.shared.data(for: request)
        } catch {
            throw GeminiError(UIStrings.tr("gemini.error.network"))
        }
        guard let http = response as? HTTPURLResponse else { throw GeminiError(UIStrings.tr("gemini.error.invalidResponse")) }
        guard http.statusCode == 200 else { throw httpError(status: http.statusCode, body: data) }
        return data
    }
}
