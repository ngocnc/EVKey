import EVKeyKit

/// What a menu entry of the action panel runs: a Gemini (or Google translate) action, or the local code table conversion.
enum PanelAction: Equatable {
    case ai(GeminiAction)
    case convert

    /// UIStrings key of the text shown while the action runs.
    var loadingKey: String {
        switch self {
        case .ai(let action): return action.loadingKey
        case .convert: return "actions.loading.convert"
        }
    }
}
