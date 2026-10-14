import EVKeyKit

/// What a menu entry of the action panel runs: a Gemini (or Google translate) action, or a local text tool of the core
/// (`convert::transform` id: `convert`, `convert_tcvn3`, `convert_vni`, `strip_marks`, `case_upper`, `case_lower`, `case_title`).
enum PanelAction: Equatable {
    case ai(GeminiAction)
    case text(String)

    /// UIStrings key of the text shown while the action runs.
    var loadingKey: String {
        switch self {
        case .ai(let action): return action.loadingKey
        case .text(let id): return id.hasPrefix("convert") ? "actions.loading.convert" : "actions.loading.text"
        }
    }
}
