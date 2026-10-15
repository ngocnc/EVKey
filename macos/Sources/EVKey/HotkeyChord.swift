import AppKit

/// Phần của phím tắt mà `TranslateHotkey` chưa có: tổ hợp chỉ có phím bổ trợ, đếm, so sánh trùng.
/// Tổ hợp có phím thường do `TranslateHotkey.capture` tạo; nhãn hiển thị do `TranslateHotkey.labels`.
enum HotkeyChord {
    static let modifierMask: NSEvent.ModifierFlags = [.control, .option, .shift, .command]

    /// Số phím bổ trợ đang giữ trong `flags`.
    static func modifierCount(_ flags: NSEvent.ModifierFlags) -> Int {
        [NSEvent.ModifierFlags.control, .option, .shift, .command].filter { flags.contains($0) }.count
    }

    /// Tổ hợp chỉ có phím bổ trợ theo thứ tự engine, ví dụ `ctrl+shift`. Command ghi thành `win`.
    static func modifierOnly(_ flags: NSEvent.ModifierFlags) -> String {
        var parts: [String] = []
        if flags.contains(.control) { parts.append("ctrl") }
        if flags.contains(.option) { parts.append("alt") }
        if flags.contains(.shift) { parts.append("shift") }
        if flags.contains(.command) { parts.append("win") }
        return parts.joined(separator: "+")
    }

    /// Hai tổ hợp trùng nhau, không phân biệt hoa thường; `cmd` và `win` là một. `none` không trùng với gì.
    static func same(_ a: String, _ b: String) -> Bool {
        guard a != "none", b != "none" else { return false }
        return normalized(a) == normalized(b)
    }

    private static func normalized(_ chord: String) -> [String] {
        let order = ["ctrl", "alt", "shift", "win"]
        let parts = chord.lowercased().split(separator: "+").map { $0 == "cmd" ? "win" : String($0) }
        return order.filter { parts.contains($0) } + parts.filter { !order.contains($0) }
    }
}
