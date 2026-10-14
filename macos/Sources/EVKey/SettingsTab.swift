import Foundation

/// Sidebar sections of Bảng điều khiển, in sidebar order.
enum SettingsTab: String, CaseIterable, Identifiable {
    case typing, smart, macro, keys, apps, appearance, system, translate, info

    var id: String { rawValue }

    var title: String {
        switch self {
        case .typing: "Gõ tiếng Việt"
        case .smart: "Chính tả & gõ nhanh"
        case .macro: "Gõ tắt"
        case .keys: "Phím tắt"
        case .apps: "Ứng dụng loại trừ"
        case .appearance: "Giao diện"
        case .system: "Hệ thống"
        case .translate: "Dịch thuật"
        case .info: "Thông tin"
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
