import EVKeyKit
import Foundation

/// Sidebar sections of Bảng điều khiển, in sidebar order.
enum SettingsTab: String, CaseIterable, Identifiable {
    case typing, smart, macro, keys, apps, appearance, system, translate, info

    var id: String { rawValue }

    var title: String {
        switch self {
        case .typing: UIStrings.tr("tab.typing")
        case .smart: UIStrings.tr("tab.smart")
        case .macro: UIStrings.tr("tab.macro")
        case .keys: UIStrings.tr("tab.keys")
        case .apps: UIStrings.tr("tab.apps")
        case .appearance: UIStrings.tr("tab.appearance")
        case .system: UIStrings.tr("tab.system")
        case .translate: UIStrings.tr("tab.translate")
        case .info: UIStrings.tr("tab.info")
        }
    }

    var systemImage: String {
        switch self {
        case .typing: "keyboard"
        case .smart: "checkmark.seal"
        case .macro: "text.badge.plus"
        case .keys: "command"
        case .apps: "app.badge.checkmark"
        case .appearance: "paintbrush"
        case .system: "gearshape"
        case .translate: "character.bubble"
        case .info: "info.circle"
        }
    }
}
