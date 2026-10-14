import Foundation

/// Reads UniKey and OpenKey macro files. Port of `parse_macros` in crates/engine/src/import.rs (read its header for the
/// format sources). Header `;[DO NOT DELETE THIS LINE]***version=N`: version 1 means UTF-8 items, any other version
/// legacy VIQR. A file without a header is plain UTF-8. UTF-16 LE with a BOM is accepted too.
enum MacroFile {
    /// `UKMACRO_VERSION_UTF8` in UniKey's `mactab.cpp`.
    private static let unikeyUTF8Version = 1

    /// `(key, text)` pairs in file order. Blank lines and `;` comment lines are skipped; any other line without a
    /// usable key throws, naming its line.
    static func parse(_ data: Data) throws -> [(key: String, text: String)] {
        let text = try decodeText(data)
        let lines = text.unicodeScalars
            .split(separator: "\n", omittingEmptySubsequences: false)
            .map { slice -> String in
                var line = String(String.UnicodeScalarView(slice))
                if line.hasSuffix("\r") { line.removeLast() }
                return line
            }
        var viqr = false
        var start = 0
        if let first = lines.first, let version = headerVersion(first) {
            viqr = version != unikeyUTF8Version
            start = 1
        }
        var out: [(key: String, text: String)] = []
        for index in start..<lines.count {
            let line = lines[index]
            let number = index + 1
            if line.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || line.hasPrefix(";") { continue }
            guard let colon = line.firstIndex(of: ":") else {
                throw CoreError("line \(number): no ':' between macro key and text")
            }
            var key = String(line[..<colon])
            var value = String(line[line.index(after: colon)...])
            if viqr {
                key = viqrDecode(key)
                value = viqrDecode(value)
            }
            key = key.trimmingCharacters(in: .whitespacesAndNewlines)
            if key.isEmpty { throw CoreError("line \(number): empty macro key") }
            out.append((key: key, text: value))
        }
        return out
    }

    /// Strips a UTF-16 LE or UTF-8 BOM and returns the text. Rejects anything that is not valid text.
    private static func decodeText(_ data: Data) throws -> String {
        let bytes = [UInt8](data)
        if bytes.starts(with: [0xFF, 0xFE]) {
            let rest = Data(bytes.dropFirst(2))
            if rest.count % 2 != 0 { throw CoreError("UTF-16 LE file has an odd number of bytes") }
            guard let text = String(data: rest, encoding: .utf16LittleEndian) else {
                throw CoreError("invalid UTF-16 LE text")
            }
            return text
        }
        let body = bytes.starts(with: [0xEF, 0xBB, 0xBF]) ? Array(bytes.dropFirst(3)) : bytes
        // Without this, UTF-16 without a BOM would parse as text with NUL characters in every key.
        if body.contains(0) {
            throw CoreError("file contains NUL bytes; save it as UTF-8 or UTF-16 LE with BOM")
        }
        guard let text = String(data: Data(body), encoding: .utf8) else {
            throw CoreError("not valid UTF-8; save the file as UTF-8 or UTF-16 LE with BOM")
        }
        return text
    }

    /// The version in a `...***version=N...` header, read like UniKey's `readHeader`.
    private static func headerVersion(_ line: String) -> Int? {
        guard let marker = line.range(of: "***") else { return nil }
        var rest = line[marker.upperBound...].drop { $0.isWhitespace }
        guard rest.hasPrefix("version=") else { return nil }
        rest = rest.dropFirst("version=".count)
        return Int(rest.prefix { $0.isASCII && $0.isNumber })
    }

    // MARK: VIQR

    /// Vowel + mark -> precomposed vowel (lowercase). Marks: `^` circumflex, `(` breve, `+` or `*` horn.
    private static let viqrMods: [(base: Character, mark: Character, result: Character)] = [
        ("a", "^", "â"), ("a", "(", "ă"), ("e", "^", "ê"), ("o", "^", "ô"), ("o", "+", "ơ"), ("u", "+", "ư"),
    ]

    /// Vowel -> its five toned forms in VIQR tone order: `` ` `` `'` `?` `~` `.`.
    private static let viqrTones: [(base: Character, forms: [Character])] = [
        ("a", ["à", "á", "ả", "ã", "ạ"]), ("ă", ["ằ", "ắ", "ẳ", "ẵ", "ặ"]), ("â", ["ầ", "ấ", "ẩ", "ẫ", "ậ"]),
        ("e", ["è", "é", "ẻ", "ẽ", "ẹ"]), ("ê", ["ề", "ế", "ể", "ễ", "ệ"]), ("i", ["ì", "í", "ỉ", "ĩ", "ị"]),
        ("o", ["ò", "ó", "ỏ", "õ", "ọ"]), ("ô", ["ồ", "ố", "ổ", "ỗ", "ộ"]), ("ơ", ["ờ", "ớ", "ở", "ỡ", "ợ"]),
        ("u", ["ù", "ú", "ủ", "ũ", "ụ"]), ("ư", ["ừ", "ứ", "ử", "ữ", "ự"]), ("y", ["ỳ", "ý", "ỷ", "ỹ", "ỵ"]),
    ]

    /// Decodes UniKey's legacy VIQR text, e.g. `Tie^'ng` -> `Tiếng`, `dd` -> `đ`, `\'` -> `'`.
    /// Simplified like the Rust version: marks apply to the previous character only when it is a vowel.
    static func viqrDecode(_ text: String) -> String {
        let chars = Array(text)
        var out: [Character] = []
        var i = 0
        while i < chars.count {
            let c = chars[i]
            i += 1
            if c == "\\" {
                if i < chars.count {
                    out.append(chars[i])
                    i += 1
                }
                continue
            }
            if c == "d" || c == "D", i < chars.count, chars[i] == "d" || chars[i] == "D" {
                i += 1
                out.append(c == "d" ? "đ" : "Đ")
                continue
            }
            // A tone mark sits between the vowel and its coda (`Tie^'ng`); a trailing `.` or `?` is punctuation.
            let followedByLetter = i < chars.count && chars[i].isLetter
            var mapped: Character?
            if let base = out.last {
                switch c {
                case "^", "(", "+", "*":
                    mapped = modify(base, c)
                case "`", "'", "?", "~", ".":
                    if followedByLetter { mapped = tone(base, c) }
                default:
                    break
                }
            }
            if let mapped, !out.isEmpty {
                out[out.count - 1] = mapped
            } else {
                out.append(c)
            }
        }
        return String(out)
    }

    private static func modify(_ base: Character, _ mark: Character) -> Character? {
        let mark: Character = mark == "*" ? "+" : mark
        let lower: Character = base.isASCII ? Character(base.lowercased()) : base
        guard let entry = viqrMods.first(where: { $0.base == lower && $0.mark == mark }) else { return nil }
        return matchedCase(base, entry.result)
    }

    private static func tone(_ base: Character, _ tone: Character) -> Character? {
        let order: [Character] = ["`", "'", "?", "~", "."]
        guard let index = order.firstIndex(of: tone),
              let lower = base.lowercased().first,
              let entry = viqrTones.first(where: { $0.base == lower }) else { return nil }
        return matchedCase(base, entry.forms[index])
    }

    /// Returns `result` uppercased when `base` is uppercase.
    private static func matchedCase(_ base: Character, _ result: Character) -> Character? {
        base.isUppercase ? result.uppercased().first : result
    }
}
