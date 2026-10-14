import Foundation
@testable import EVKeyKit

/// Self-checks for macro variables, the TOML backup format, macro file import and config backup. Called from main.swift.
func runMiscChecks() {
    // Macro variables (mirrors fill_vars in crates/app/src/hook.rs).
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = .current
    let moment = DateComponents(calendar: calendar, year: 2026, month: 3, day: 5, hour: 7, minute: 9).date ?? Date()
    var clipboardReads = 0
    let filled = MacroVars.fill("{date} {time}", now: moment) { clipboardReads += 1; return "X" }
    check(filled == "05/03/2026 07:09", "date is dd/MM/yyyy and time is HH:mm")
    check(clipboardReads == 0, "clipboard is not read unless {clipboard} appears")
    check(MacroVars.fill("a{clipboard}b{clipboard}", now: moment, clipboard: { "{date}" }) == "a{date}b{date}", "clipboard text is not expanded again")
    check(MacroVars.fill("{clipboard}", now: moment, clipboard: { nil }) == "", "empty clipboard gives empty text")
    check(MacroVars.expand(Array("xin chào".utf16)) == Array("xin chào".utf16), "text without { is untouched")

    // TOML subset: encode then decode, plus hand-written variants Windows may produce.
    let tree: [String: ConfigValue] = [
        "format_version": .int(1),
        "settings": .object([
            "toggle_hotkey": .string("alt+z"), "free_mark": .bool(true), "banner_opacity": .int(80),
            "custom_consonants": .array([.string("z"), .string("w")]), "excluded_apps": .array([]),
        ]),
        "macros": .object([
            "btw": .string("by the way"), "vn": .string("Việt \"Nam\"\nline2\t\\"), "a.b c": .string("x"),
        ]),
    ]
    check((try? TomlCodec.decode(TomlCodec.encode(tree))) == tree, "TOML encode/decode round trip")
    let hand = "# comment\nformat_version = 1 # trailing\n\n[settings]\nlist = [\n  \"a\", # first\n  'b',\n]\nmulti = \"\"\"\nline1\nline2\"\"\"\nn = 1_000\n[macros]\n\"k y\" = 'raw \\ text'\n"
    let parsed = try? TomlCodec.decode(hand)
    check(parsed?["format_version"] == .int(1), "TOML top-level integer with comment")
    if case .object(let settings)? = parsed?["settings"] {
        check(settings["list"] == .array([.string("a"), .string("b")]), "TOML multi-line array with comments")
        check(settings["multi"] == .string("line1\nline2"), "TOML multi-line string drops the first newline")
        check(settings["n"] == .int(1000), "TOML integer with underscore")
    } else {
        check(false, "TOML [settings] table parsed")
    }
    if case .object(let macros)? = parsed?["macros"] {
        check(macros["k y"] == .string("raw \\ text"), "TOML quoted key and literal string")
    } else {
        check(false, "TOML [macros] table parsed")
    }
    check((try? TomlCodec.decode("a = 1\na = 2")) == nil, "duplicate TOML key is rejected")
    check((try? TomlCodec.decode("a = {b = 1}")) == nil, "inline table is rejected")

    // UniKey / OpenKey macro files (mirrors the parse_macros tests' cases).
    let utf8File = Data(";[DO NOT DELETE THIS LINE]***version=1\nbtw:by the way\n\n;note\nvn:Việt Nam\n".utf8)
    let utf8Rows = (try? MacroFile.parse(utf8File)) ?? []
    check(utf8Rows.map { $0.key } == ["btw", "vn"] && utf8Rows.map { $0.text } == ["by the way", "Việt Nam"], "UniKey UTF-8 file")
    let openKey = Data(";Compatible OpenKey Macro Data file for UniKey*** version=1 ***\r\nbtw:by:the way\r\n".utf8)
    check((try? MacroFile.parse(openKey))?.first?.text == "by:the way", "OpenKey file splits at the first colon")
    check((try? MacroFile.parse(Data("add:ađ\n".utf8)))?.first?.text == "ađ", "headerless file is UTF-8, not VIQR")
    var utf16 = Data([0xFF, 0xFE])
    for unit in "k:v\n".utf16 { utf16.append(UInt8(unit & 0xFF)); utf16.append(UInt8(unit >> 8)) }
    check((try? MacroFile.parse(utf16))?.first?.text == "v", "UTF-16 LE with BOM")
    check((try? MacroFile.parse(Data("no colon here".utf8))) == nil, "line without colon is an error")
    check((try? MacroFile.parse(Data(":x\n".utf8))) == nil, "empty macro key is an error")
    let viqr = (try? MacroFile.parse(Data("***version=0\nvn:Vie^.t Nam, dd\n".utf8)))?.first?.text
    check(viqr == "Việt Nam, đ", "VIQR header decodes marks and dd")
    check(MacroFile.viqrDecode("Tie^'ng") == "Tiếng", "VIQR circumflex then acute")
    check(MacroFile.viqrDecode("end.") == "end.", "trailing dot stays punctuation")

    // Config backup through the real core.
    let source = FileManager.default.temporaryDirectory.appendingPathComponent("evkit-backup-\(UUID().uuidString)").path
    let target = FileManager.default.temporaryDirectory.appendingPathComponent("evkit-backup-\(UUID().uuidString)").path
    defer {
        try? FileManager.default.removeItem(atPath: source)
        try? FileManager.default.removeItem(atPath: target)
    }
    do {
        var settings = EVSettings.defaults
        settings.freeMark = true
        settings.bannerOpacity = 61
        try SettingsStore.save(directory: source, settings: settings, macros: ["btw": "by the way", "vn": "Việt Nam"])
        let text = try ConfigBackup.export(directory: source)
        check(text.hasPrefix("format_version = 1\n") && text.contains("\n[settings]\n") && text.contains("\n[macros]\n"), "export has the Windows layout")
        try ConfigBackup.importConfig(text, directory: target)
        let restored = try SettingsStore.load(directory: target)
        check(restored.settings == settings, "import restores settings")
        check(restored.macros == ["btw": "by the way", "vn": "Việt Nam"], "import restores macros")
        let bad = text.replacingOccurrences(of: "format_version = 1", with: "format_version = 2")
        check((try? ConfigBackup.importConfig(bad, directory: target)) == nil, "other format_version is rejected")
        try ConfigBackup.importMacros([(key: "new", text: "N"), (key: "btw", text: "changed")], replace: false, directory: target)
        let merged = try SettingsStore.load(directory: target)
        check(merged.macros == ["btw": "changed", "vn": "Việt Nam", "new": "N"], "macro import merges")
        try ConfigBackup.importMacros([(key: "only", text: "O")], replace: true, directory: target)
        let replaced = try SettingsStore.load(directory: target)
        check(replaced.macros == ["only": "O"], "macro import can replace all")
        check(replaced.settings == settings, "macro import keeps the settings")
    } catch {
        check(false, "config backup threw: \(error)")
    }
}
