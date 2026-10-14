import AppKit
import Foundation

/// Macro variables, expanded at the point where an edit is typed (the Windows hook does the same in `expand_vars`):
/// `{date}` is dd/MM/yyyy, `{time}` is HH:mm (24 hours, local time), `{clipboard}` is the current clipboard text.
enum MacroVars {
    /// Applied to every edit, not only macro output; typed words never contain `{`, so plain typing is untouched.
    static func expand(_ units: [UInt16]) -> [UInt16] {
        guard units.contains(0x7B) else { return units }
        let text = String(decoding: units, as: UTF16.self)
        let filled = fill(text, now: Date(), clipboard: { NSPasteboard.general.string(forType: .string) })
        return Array(filled.utf16)
    }

    /// `clipboard` is read only when the text mentions `{clipboard}`, after the date and time are filled in.
    static func fill(_ text: String, now: Date, clipboard: () -> String?) -> String {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = .current
        let c = calendar.dateComponents([.day, .month, .year, .hour, .minute], from: now)
        let date = String(format: "%02d/%02d/%d", c.day ?? 0, c.month ?? 0, c.year ?? 0)
        let time = String(format: "%02d:%02d", c.hour ?? 0, c.minute ?? 0)
        var out = text.replacingOccurrences(of: "{date}", with: date).replacingOccurrences(of: "{time}", with: time)
        if out.contains("{clipboard}") {
            out = out.replacingOccurrences(of: "{clipboard}", with: clipboard() ?? "")
        }
        return out
    }
}
