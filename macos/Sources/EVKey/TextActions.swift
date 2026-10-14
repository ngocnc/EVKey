import EVKeyKit
import Foundation

/// Produces the text an action yields for a selection (`compute` in `crates/app/src/actions.rs`).
@MainActor
enum TextActions {
    /// Translate goes to Google unless Gemini is chosen; every other action is a Gemini prompt.
    static func run(_ action: GeminiAction, text: String) async throws -> String {
        if action == .translate && AppState.shared.settings.translateProvider == .google {
            return try await googleTranslate(text)
        }
        return try await AppState.shared.runGemini(action, text: text)
    }

    /// A panel entry: the local code table conversion, or the AI / translate action behind it.
    static func run(_ action: PanelAction, text: String) async throws -> String {
        switch action {
        case .ai(let ai):
            return try await run(ai, text: text)
        case .convert:
            guard let converted = CodeConvert.toUnicode(text) else { throw ConvertFailure() }
            return converted
        }
    }

    /// The core rejected the text of a conversion.
    struct ConvertFailure: Error {}

    /// Text for the user when `run` threw: Gemini errors are already worded; a failed conversion has its own text;
    /// anything else is a Google failure.
    static func message(for error: Error) -> String {
        if let gemini = error as? GeminiError { return gemini.message }
        if error is ConvertFailure { return UIStrings.tr("actions.error.convert") }
        return UIStrings.tr("translate.error.failed", error.localizedDescription)
    }

    /// Vietnamese source goes to English, anything else to Vietnamese.
    private static func googleTranslate(_ text: String) async throws -> String {
        // Ask for Vietnamese first; only a Vietnamese source needs the second request, into English.
        let first = try await fetch(text, target: "vi")
        let to = Translate.target(detected: first.detected)
        if to == "vi" { return first.text }
        return try await fetch(text, target: to).text
    }

    private static func fetch(_ text: String, target: String) async throws -> Translate.Translation {
        guard let url = Translate.url(text: text, target: target) else { throw URLError(.badURL) }
        var request = URLRequest(url: url)
        request.timeoutInterval = 10
        request.setValue(AppState.shared.userAgent, forHTTPHeaderField: "User-Agent")
        let (data, response) = try await URLSession.shared.data(for: request)
        guard let http = response as? HTTPURLResponse, http.statusCode == 200,
              let translation = Translate.parse(data) else { throw URLError(.cannotParseResponse) }
        return translation
    }
}
