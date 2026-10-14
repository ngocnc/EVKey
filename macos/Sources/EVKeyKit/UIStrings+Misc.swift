import Foundation

/// Strings of the welcome screen, the Advanced sections and the backup/import rows. Same shape as the main table in
/// `UIStrings.swift`, kept apart so the two can change independently. Texts mirror crates/app/ui/i18n.js.
extension UIStrings {
    /// Returns the text for `key` in the current `language`. `args` fill `%@`/`%ld` placeholders in order.
    public static func misc(_ key: String, _ args: CVarArg...) -> String {
        guard let pair = miscTable[key] else {
            assertionFailure("Missing UI string: \(key)")
            return key
        }
        let text = language == .en ? pair.en : pair.vi
        return args.isEmpty ? text : String(format: text, arguments: args)
    }

    private static let miscTable: [String: (vi: String, en: String)] = [
        "advanced": ("Nâng cao", "Advanced"),
        // Welcome screen
        "welcome.title": ("Chào mừng đến EVKey", "Welcome to EVKey"),
        "welcome.sub": (
            "Ba bước nhỏ là gõ được tiếng Việt. Mọi thứ khác chỉnh sau ở các mục bên trái.",
            "Three small steps and you can type Vietnamese. Everything else can be tuned later from the left."),
        "welcome.stepInput": ("Chọn kiểu gõ bạn quen", "Pick the input method you know"),
        "welcome.stepToggle": ("Chuyển Việt ⇄ Anh", "Switch Vietnamese ⇄ English"),
        "welcome.toggleLine": (
            "Bấm %@ bất cứ lúc nào. Đổi phím ở mục Phím tắt.", "Press %@ at any time. Change it under Hotkeys."),
        "welcome.toggleNone": (
            "Chưa đặt phím tắt; bấm icon EVKey trên thanh menu để chuyển.",
            "No hotkey set; click the EVKey menu bar icon to switch."),
        "welcome.stepTry": ("Gõ thử", "Try it"),
        "welcome.tryTelex": (
            "Ví dụ: vieejt nam → việt nam, ddaay → đây", "Example: vieejt nam → việt nam, ddaay → đây"),
        "welcome.tryVni": (
            "Ví dụ: vie65t nam → việt nam, d9a6y → đây", "Example: vie65t nam → việt nam, d9a6y → đây"),
        "welcome.skip": ("Bỏ qua", "Skip"),
        "welcome.start": ("Bắt đầu", "Get started"),

        // Macro import (UniKey / OpenKey)
        "macroImport.title": ("Nhập từ UniKey / OpenKey", "Import from UniKey / OpenKey"),
        "macroImport.desc": (
            "Chọn file gõ tắt đã xuất từ UniKey hoặc OpenKey.", "Pick a macro file exported from UniKey or OpenKey."),
        "macroImport.replace": ("Thay toàn bộ", "Replace all"),
        "macroImport.button": ("Chọn file…", "Choose file…"),
        "macroImport.done": ("Đã nhập %ld từ gõ tắt", "Imported %ld macros"),
        "macroImport.confirm": (
            "Xóa toàn bộ gõ tắt hiện có rồi nhập file này?", "Delete all current macros and import this file?"),
        "macroImport.replaceAction": ("Thay toàn bộ", "Replace all"),

        // Backup
        "backup.heading": ("Sao lưu", "Backup"),
        "backup.exportTitle": ("Xuất cấu hình", "Export settings"),
        "backup.exportDesc": (
            "Lưu cài đặt và gõ tắt thành một file. Không chứa API key.",
            "Saves settings and macros as one file. API keys are never included."),
        "backup.exportButton": ("Xuất", "Export"),
        "backup.exportDone": ("Đã lưu %@", "Saved %@"),
        "backup.importTitle": ("Nhập cấu hình", "Import settings"),
        "backup.importDesc": (
            "Thay cài đặt và gõ tắt bằng file đã xuất.", "Replaces settings and macros with an exported file."),
        "backup.importButton": ("Chọn file…", "Choose file…"),
        "backup.importConfirm": (
            "Thay toàn bộ cài đặt và gõ tắt bằng file này?", "Replace all settings and macros with this file?"),
        "backup.importDone": ("Đã nhập cấu hình", "Settings imported"),
        "backup.importAction": ("Thay thế", "Replace"),
        "backup.failed": ("Không thực hiện được", "Could not complete"),
        "backup.readFailed": ("Không đọc được file: %@", "Could not read the file: %@"),
        "backup.writeFailed": ("Không ghi được file: %@", "Could not write the file: %@"),
    ]
}
