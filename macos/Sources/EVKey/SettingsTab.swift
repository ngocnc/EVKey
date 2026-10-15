import EVKeyKit
import Foundation

/// Sidebar sections of Bảng điều khiển, in sidebar order.
enum SettingsTab: String, CaseIterable, Identifiable {
    case typing, macro, keys, apps, system, translate

    var id: String { rawValue }

    var title: String {
        switch self {
        case .typing: UIStrings.tr("tab.typing")
        case .macro: UIStrings.tr("tab.macro")
        case .keys: UIStrings.tr("tab.keys")
        case .apps: UIStrings.tr("tab.apps")
        case .system: UIStrings.tr("tab.system")
        case .translate: UIStrings.tr("tab.translate")
        }
    }
}
