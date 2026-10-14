import Foundation

/// The small part of TOML that EVKey's backup file uses: `key = value` pairs, one level of `[table]` headers, strings
/// (basic, literal, and their multi-line forms), integers, floats, booleans and arrays. Anything else (inline tables,
/// dotted keys, dates, arrays of tables) is reported as an error rather than guessed at.
enum TomlCodec {
    // MARK: Encode

    /// `root` holds scalars and arrays at the top and tables (objects) below; objects nest one level only.
    static func encode(_ root: [String: ConfigValue]) -> String {
        var out = ""
        for key in root.keys.sorted() {
            guard let value = root[key] else { continue }
            if case .object = value { continue }
            out += "\(keyText(key)) = \(valueText(value))\n"
        }
        for key in root.keys.sorted() {
            guard case .object(let table)? = root[key] else { continue }
            out += "\n[\(keyText(key))]\n"
            for inner in table.keys.sorted() {
                guard let value = table[inner] else { continue }
                out += "\(keyText(inner)) = \(valueText(value))\n"
            }
        }
        return out
    }

    private static func keyText(_ key: String) -> String {
        let bare = !key.isEmpty && key.unicodeScalars.allSatisfy {
            ($0.value >= 0x30 && $0.value <= 0x39) || ($0.value >= 0x41 && $0.value <= 0x5A)
                || ($0.value >= 0x61 && $0.value <= 0x7A) || $0 == "_" || $0 == "-"
        }
        return bare ? key : quoted(key)
    }

    private static func valueText(_ value: ConfigValue) -> String {
        switch value {
        case .string(let text): return quoted(text)
        case .int(let number): return String(number)
        case .double(let number): return String(number)
        case .bool(let flag): return flag ? "true" : "false"
        case .array(let items): return "[" + items.map(valueText).joined(separator: ", ") + "]"
        case .object: return "{}"  // never reached: callers write objects as tables
        }
    }

    private static func quoted(_ text: String) -> String {
        var out = "\""
        for scalar in text.unicodeScalars {
            switch scalar {
            case "\"": out += "\\\""
            case "\\": out += "\\\\"
            case "\n": out += "\\n"
            case "\r": out += "\\r"
            case "\t": out += "\\t"
            case "\u{08}": out += "\\b"
            case "\u{0C}": out += "\\f"
            default:
                if scalar.value < 0x20 || scalar.value == 0x7F {
                    out += String(format: "\\u%04X", scalar.value)
                } else {
                    out.unicodeScalars.append(scalar)
                }
            }
        }
        return out + "\""
    }

    // MARK: Decode

    /// Parses `text` into a root object. Throws `CoreError` naming the line.
    static func decode(_ text: String) throws -> [String: ConfigValue] {
        var parser = Parser(text)
        return try parser.document()
    }

    private struct Parser {
        private let s: [Unicode.Scalar]
        private var i = 0

        init(_ text: String) {
            var scalars = Array(text.unicodeScalars)
            if scalars.first == "\u{FEFF}" { scalars.removeFirst() }
            s = scalars
        }

        private var line: Int { s[..<min(i, s.count)].reduce(1) { $1 == "\n" ? $0 + 1 : $0 } }

        private func fail(_ message: String) -> CoreError { CoreError("TOML line \(line): \(message)") }

        private var atEnd: Bool { i >= s.count }
        private var current: Unicode.Scalar? { i < s.count ? s[i] : nil }

        private func at(_ offset: Int) -> Unicode.Scalar? { i + offset < s.count ? s[i + offset] : nil }

        private func hasPrefix(_ text: String) -> Bool {
            let scalars = Array(text.unicodeScalars)
            return i + scalars.count <= s.count && Array(s[i..<(i + scalars.count)]) == scalars
        }

        private mutating func skipBlanks() {
            while let c = current, c == " " || c == "\t" { i += 1 }
        }

        private mutating func skipComment() {
            if current == "#" { while let c = current, c != "\n" { i += 1 } }
        }

        /// Blanks, comments and line breaks, as allowed between statements and inside arrays.
        private mutating func skipSpace() {
            while let c = current {
                if c == " " || c == "\t" || c == "\n" || c == "\r" {
                    i += 1
                } else if c == "#" {
                    skipComment()
                } else {
                    break
                }
            }
        }

        /// The end of a statement: optional blanks and comment, then a line break or the end of the text.
        private mutating func endOfLine() throws {
            skipBlanks()
            skipComment()
            if atEnd { return }
            if current == "\r" { i += 1 }
            guard current == "\n" else { throw fail("unexpected text after the value") }
            i += 1
        }

        mutating func document() throws -> [String: ConfigValue] {
            var root: [String: ConfigValue] = [:]
            var table: String?
            skipSpace()
            while !atEnd {
                if current == "[" {
                    if at(1) == "[" { throw fail("arrays of tables are not supported") }
                    i += 1
                    skipBlanks()
                    let name = try key()
                    skipBlanks()
                    guard current == "]" else { throw fail("expected ']' after the table name") }
                    i += 1
                    try endOfLine()
                    if root[name] != nil { throw fail("duplicate table [\(name)]") }
                    root[name] = .object([:])
                    table = name
                } else {
                    let name = try key()
                    skipBlanks()
                    guard current == "=" else { throw fail("expected '=' after the key \"\(name)\"") }
                    i += 1
                    skipBlanks()
                    let value = try self.value()
                    try endOfLine()
                    if let table {
                        guard case .object(var entries)? = root[table] else { throw fail("internal table error") }
                        if entries[name] != nil { throw fail("duplicate key \"\(name)\"") }
                        entries[name] = value
                        root[table] = .object(entries)
                    } else {
                        if root[name] != nil { throw fail("duplicate key \"\(name)\"") }
                        root[name] = value
                    }
                }
                skipSpace()
            }
            return root
        }

        private mutating func key() throws -> String {
            if current == "\"" { return try basicString() }
            if current == "'" { return try literalString() }
            let start = i
            while let c = current, isBare(c) { i += 1 }
            guard i > start else { throw fail("expected a key") }
            if current == "." { throw fail("dotted keys are not supported") }
            var name = ""
            name.unicodeScalars.append(contentsOf: s[start..<i])
            return name
        }

        private func isBare(_ c: Unicode.Scalar) -> Bool {
            (c.value >= 0x30 && c.value <= 0x39) || (c.value >= 0x41 && c.value <= 0x5A)
                || (c.value >= 0x61 && c.value <= 0x7A) || c == "_" || c == "-"
        }

        private mutating func value() throws -> ConfigValue {
            guard let c = current else { throw fail("missing value") }
            switch c {
            case "\"":
                return .string(try hasPrefix("\"\"\"") ? multilineBasic() : basicString())
            case "'":
                return .string(try hasPrefix("'''") ? multilineLiteral() : literalString())
            case "[":
                return try array()
            case "{":
                throw fail("inline tables are not supported")
            default:
                return try scalar()
            }
        }

        private mutating func array() throws -> ConfigValue {
            i += 1
            var items: [ConfigValue] = []
            while true {
                skipSpace()
                if current == "]" { i += 1; return .array(items) }
                items.append(try value())
                skipSpace()
                if current == "," {
                    i += 1
                } else if current != "]" {
                    throw fail("expected ',' or ']' in the array")
                }
            }
        }

        private mutating func scalar() throws -> ConfigValue {
            let start = i
            while let c = current, c != " ", c != "\t", c != "\n", c != "\r", c != "#", c != ",", c != "]" { i += 1 }
            var word = ""
            word.unicodeScalars.append(contentsOf: s[start..<i])
            if word == "true" { return .bool(true) }
            if word == "false" { return .bool(false) }
            let digits = word.replacingOccurrences(of: "_", with: "")
            if let number = Int(digits) { return .int(number) }
            if let number = Double(digits), !digits.lowercased().contains("n") { return .double(number) }
            throw fail("unsupported value \"\(word)\"")
        }

        private mutating func basicString() throws -> String {
            i += 1
            var out = ""
            while true {
                guard let c = current, c != "\n" else { throw fail("unterminated string") }
                i += 1
                if c == "\"" { return out }
                if c == "\\" { try escape(into: &out) } else { out.unicodeScalars.append(c) }
            }
        }

        private mutating func literalString() throws -> String {
            i += 1
            var out = ""
            while true {
                guard let c = current, c != "\n" else { throw fail("unterminated string") }
                i += 1
                if c == "'" { return out }
                out.unicodeScalars.append(c)
            }
        }

        private mutating func multilineBasic() throws -> String {
            i += 3
            if current == "\r" { i += 1 }
            if current == "\n" { i += 1 }
            var out = ""
            while true {
                guard let c = current else { throw fail("unterminated multi-line string") }
                if hasPrefix("\"\"\"") {
                    i += 3
                    // Up to two more quotes right before the closing delimiter belong to the string.
                    while current == "\"" { out += "\""; i += 1 }
                    return out
                }
                i += 1
                if c == "\\" {
                    // A backslash at the end of a line trims the line break and the blanks that follow.
                    var look = i
                    while look < s.count, s[look] == " " || s[look] == "\t" || s[look] == "\r" { look += 1 }
                    if look < s.count, s[look] == "\n" {
                        i = look
                        while let w = current, w == " " || w == "\t" || w == "\r" || w == "\n" { i += 1 }
                    } else {
                        try escape(into: &out)
                    }
                } else if c == "\r", current == "\n" {
                    continue
                } else {
                    out.unicodeScalars.append(c)
                }
            }
        }

        private mutating func multilineLiteral() throws -> String {
            i += 3
            if current == "\r" { i += 1 }
            if current == "\n" { i += 1 }
            var out = ""
            while true {
                guard let c = current else { throw fail("unterminated multi-line string") }
                if hasPrefix("'''") {
                    i += 3
                    while current == "'" { out += "'"; i += 1 }
                    return out
                }
                i += 1
                if c == "\r", current == "\n" { continue }
                out.unicodeScalars.append(c)
            }
        }

        /// The character after a backslash.
        private mutating func escape(into out: inout String) throws {
            guard let c = current else { throw fail("unterminated string") }
            i += 1
            switch c {
            case "b": out += "\u{08}"
            case "t": out += "\t"
            case "n": out += "\n"
            case "f": out += "\u{0C}"
            case "r": out += "\r"
            case "\"": out += "\""
            case "\\": out += "\\"
            case "u", "U":
                let length = c == "u" ? 4 : 8
                guard i + length <= s.count else { throw fail("bad \\\(c) escape") }
                var hex = ""
                hex.unicodeScalars.append(contentsOf: s[i..<(i + length)])
                guard let code = UInt32(hex, radix: 16), let scalar = Unicode.Scalar(code) else {
                    throw fail("bad \\\(c) escape")
                }
                i += length
                out.unicodeScalars.append(scalar)
            default:
                throw fail("unknown escape \\\(c)")
            }
        }
    }
}
