import CoreGraphics
import Foundation
@testable import EVKeyKit

// Plain self-check: the toolchain has no XCTest or swift-testing macros. Run: swift run EVKeyKitSelfCheck
var failures = 0

func check(_ ok: @autoclosure () -> Bool, _ what: String) {
    if !ok() {
        failures += 1
        print("FAIL: \(what)")
    }
}

// Keycode <-> VK map.
check(KeyMap.vk(forKeycode: 0) == 0x41, "A keycode maps to VK_A")
check(KeyMap.vk(forKeycode: 18) == 0x31, "1 keycode maps to VK_1")
check(KeyMap.vk(forKeycode: 29) == 0x30, "0 keycode maps to VK_0")
check(KeyMap.vk(forKeycode: 122) == 0x70, "F1 is VK_F1")
for (keycode, vk) in KeyMap.vkByKeycode {
    check(KeyMap.keycode(forVK: vk) == keycode, "round trip keycode \(keycode) via VK \(vk)")
}
check(KeyMap.modifier(forKeycode: 59)?.vk == 0xA2, "left Control is VK_LCONTROL")
check(KeyMap.modifier(forKeycode: 62)?.vk == 0xA3, "right Control is VK_RCONTROL")
check(KeyMap.modifier(forKeycode: 55)?.vk == 0x5B, "left Command is VK_LWIN")
check(KeyMap.modBits([.maskControl, .maskShift]) == 1 | 4, "ctrl+shift mod bits")
check(KeyMap.modBits([.maskCommand]) == 8, "command mod bit")

// EVSettings through the real core: save, then load back.
let dir = FileManager.default.temporaryDirectory
    .appendingPathComponent("evkit-check-\(UUID().uuidString)").path
do {
    try FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
    var settings = EVSettings.defaults
    settings.bannerOpacity = 55
    settings.bannerBg = "#123456"
    try SettingsStore.save(directory: dir, settings: settings, macros: ["btw": "by the way"])
    let loaded = try SettingsStore.load(directory: dir)
    check(loaded.settings == settings, "EVSettings round-trip through core")
    check(loaded.macros == ["btw": "by the way"], "macros round-trip through core")
} catch {
    check(false, "core settings round-trip threw: \(error)")
}
try? FileManager.default.removeItem(atPath: dir)

// Update check: version compare, notes split, release parsing (mirrors crates/app/src/updater.rs tests).
check(UpdateCheck.isNewer("v2.3.0", than: "2.2.9"), "newer minor compares numerically")
check(UpdateCheck.isNewer("2.10.0", than: "2.9.0"), "2.10 is newer than 2.9")
check(!UpdateCheck.isNewer("v2.2.2", than: "2.2.2"), "same version is not newer")
check(!UpdateCheck.isNewer("2.2.1", than: "2.2.2"), "older version is not newer")
check(!UpdateCheck.isNewer("nightly", than: "2.2.2"), "unparsable version is not newer")
check(UpdateCheck.notes(fromBody: "- Fix A\n- Fix B\n\nSHA256: abc123\n") == "- Fix A\n- Fix B", "SHA256 line removed from notes")
let releaseJSON = #"{"tag_name":"v9.0.0","body":"Notes\nSHA256: aa","assets":[{"name":"EVKey-Setup-9.0.0.exe","browser_download_url":"https://x/a.exe"},{"name":"EVKey-9.0.0-universal.dmg","browser_download_url":"https://x/a.dmg"}]}"#
do {
    let release = try UpdateCheck.latest(fromJSON: Data(releaseJSON.utf8), current: "2.2.2")
    check(release?.version == "9.0.0", "release version strips v")
    check(release?.notes == "Notes", "release notes drop SHA256 line")
    check(release?.downloadURL.absoluteString == "https://x/a.dmg", "dmg asset picked")
    let noDmg = #"{"tag_name":"v9.0.0","body":"","assets":[{"name":"EVKey-Setup-9.0.0.exe","browser_download_url":"https://x/a.exe"}]}"#
    let noDmgRelease = try UpdateCheck.latest(fromJSON: Data(noDmg.utf8), current: "2.2.2")
    check(noDmgRelease == nil, "release without dmg is not offered")
    let same = #"{"tag_name":"v2.2.2","body":"","assets":[]}"#
    let sameRelease = try UpdateCheck.latest(fromJSON: Data(same.utf8), current: "2.2.2")
    check(sameRelease == nil, "current release is not offered")
} catch {
    check(false, "release JSON parse threw: \(error)")
}
// Translate: response parsing, target choice, query encoding (mirrors crates/app/src/translate.rs tests).
let gtxJSON = #"[[["Xin chào","Hello",null,null,1],["  thế giới","world",null,null,1]],null,"en"]"#
let parsed = Translate.parse(Data(gtxJSON.utf8))
check(parsed == Translate.Translation(text: "Xin chào  thế giới", detected: "en"), "gtx segments join and detected language is read")
check(Translate.parse(Data("[[]]".utf8)) == nil, "empty translation is rejected")
check(Translate.parse(Data("not json".utf8)) == nil, "invalid JSON is rejected")
check(Translate.target(detected: "vi") == "en", "Vietnamese source goes to English")
check(Translate.target(detected: "vi-VN") == "en", "vi-VN counts as Vietnamese")
check(Translate.target(detected: "en") == "vi", "English source goes to Vietnamese")
check(Translate.target(detected: "") == "vi", "unknown source goes to Vietnamese")
let queryURL = Translate.url(text: "a+b & c", target: "vi")?.absoluteString ?? ""
check(queryURL.contains("q=a%2Bb%20%26%20c"), "plus and ampersand are percent-encoded in q")
check(queryURL.contains("tl=vi") && queryURL.contains("sl=auto"), "URL carries target and auto source")

// Translate hotkey: chords as the Rust core reads them.
check(TranslateHotkey.capture(keycode: 0, control: true, option: false, shift: false, command: false) == .chord("ctrl+a"), "Control+A builds ctrl+a")
check(TranslateHotkey.capture(keycode: 0, control: false, option: false, shift: false, command: true) == .chord("win+a"), "Command is win")
check(TranslateHotkey.capture(keycode: 0, control: false, option: false, shift: false, command: false) == .needsModifier, "bare letter needs a modifier")
check(TranslateHotkey.capture(keycode: 122, control: false, option: false, shift: false, command: false) == .chord("f1"), "F1 alone is allowed")
check(TranslateHotkey.capture(keycode: 49, control: true, option: true, shift: false, command: false) == .chord("ctrl+alt+space"), "Space with two modifiers")
check(TranslateHotkey.capture(keycode: 36, control: true, option: false, shift: false, command: false) == .badKey, "Return is not bindable")
check(TranslateHotkey.capture(keycode: 18, control: false, option: true, shift: true, command: false) == .chord("alt+shift+1"), "modifier order is ctrl alt shift win")
check(TranslateHotkey.labels("ctrl+alt+t") == ["⌃", "⌥", "T"], "labels use symbols")
check(TranslateDisplayTime.unit(forMs: 0) == .never && TranslateDisplayTime.unit(forMs: 5000) == .s && TranslateDisplayTime.unit(forMs: 250) == .ms, "unit from stored milliseconds")
check(TranslateDisplayTime.shownValue(ms: 5000, unit: .s) == 5 && TranslateDisplayTime.shownValue(ms: 250, unit: .s) == 1, "seconds round, minimum 1")
check(TranslateDisplayTime.storedMs(value: 0, unit: .s) == 1000 && TranslateDisplayTime.storedMs(value: 0, unit: .ms) == 100, "unit minimums")
check(TranslateDisplayTime.storedMs(value: 999_999, unit: .ms) == 600_000 && TranslateDisplayTime.storedMs(value: 9_999, unit: .s) == 600_000, "maximum is 600000 ms")
check(TranslateDisplayTime.storedMs(value: 7, unit: .never) == 0, "never stores 0")

// Gemini: action ids, prompts, request body, response parsing, error mapping (mirrors crates/app/src/ai.rs tests).
UIStrings.language = .vi
check(GeminiAction(rawValue: "add_diacritics") == .addDiacritics, "add_diacritics id maps to its action")
check(GeminiAction(rawValue: "rewrite_shorter") == .rewriteShorter, "rewrite_shorter id maps to its action")
check(GeminiAction(rawValue: "translate") == .translate, "translate id maps to its action")
check(GeminiAction(rawValue: "nope") == nil, "unknown action id is rejected")
let prompts = GeminiAction.allCases.map(\.systemPrompt)
check(prompts.count == 6, "six Gemini actions")
check(prompts.allSatisfy { $0.contains("ONLY") }, "every prompt asks for only the result")
check(Set(prompts).count == prompts.count, "every action has its own prompt")
check(GeminiAction.rewriteFriendly.loadingKey == "actions.loading.rewrite", "rewrite tones share one loading text")
check(GeminiAction.rewriteFormal.loadingKey == GeminiAction.rewriteShorter.loadingKey, "all tones share the loading key")
check(GeminiAction.addDiacritics.loadingKey == "actions.loading.add_diacritics", "add diacritics has its own loading text")
check(GeminiAction.allCases.allSatisfy { UIStrings.tr($0.loadingKey) != $0.loadingKey }, "every loading text exists in UIStrings")
if let body = try? JSONSerialization.data(withJSONObject: Gemini.requestBody(action: .addDiacritics, text: "tieng viet")),
   let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any] {
    let system = ((json["systemInstruction"] as? [String: Any])?["parts"] as? [[String: Any]])?.first?["text"] as? String
    check(system?.contains("tone and vowel marks") == true, "request carries the system prompt")
    let content = (json["contents"] as? [[String: Any]])?.first
    check(content?["role"] as? String == "user", "request content role is user")
    check((content?["parts"] as? [[String: Any]])?.first?["text"] as? String == "tieng viet", "request carries the text")
    check((json["generationConfig"] as? [String: Any])?["temperature"] as? Double == 0.2, "request temperature is 0.2")
} else {
    check(false, "request body is valid JSON")
}
func geminiParse(_ json: String) -> (text: String?, error: String?) {
    do { return (try Gemini.parse(Data(json.utf8)), nil) } catch { return (nil, error.localizedDescription) }
}
check(geminiParse(#"{"candidates":[{"content":{"parts":[{"text":"thinking","thought":true},{"text":"  Xin chào\nbạn  "}]}}]}"#).text == "Xin chào\nbạn", "response skips thought parts and trims")
check(geminiParse(#"{"candidates":[]}"#).error == UIStrings.tr("gemini.error.noContent"), "no candidates is an error")
check(geminiParse(#"{"promptFeedback":{"blockReason":"SAFETY"}}"#).error == UIStrings.tr("gemini.error.refused"), "blocked prompt is refused")
check(geminiParse("not json").error == UIStrings.tr("gemini.error.invalidResponse"), "invalid JSON is an error")
check(Gemini.httpError(status: 429, body: Data()).message == UIStrings.tr("gemini.error.quota"), "429 is a quota error")
check(Gemini.httpError(status: 403, body: Data()).message == UIStrings.tr("gemini.error.invalidKey"), "403 is an invalid key error")
check(Gemini.httpError(status: 500, body: Data(#"{"error":{"message":"boom"}}"#.utf8)).message.hasSuffix(": boom"), "other statuses show Gemini's message")
check(Gemini.httpError(status: 502, body: Data()).message.hasSuffix(": HTTP 502"), "other statuses without a body show the status")

// Editable focus decision (the Accessibility reads themselves need a Mac with a focused app).
check(EditableField.isEditable(.init(role: "AXTextField")), "a text field is editable")
check(EditableField.isEditable(.init(role: "AXTextArea", editableFlag: true)), "a text area is editable")
check(EditableField.isEditable(.init(role: "AXComboBox")), "a combo box is editable")
check(!EditableField.isEditable(.init(role: "AXTextField", subrole: "AXSecureTextField", valueSettable: true)), "a password field is never editable")
check(!EditableField.isEditable(.init(role: "AXTextArea", editableFlag: false)), "a read-only text area is not editable")
check(EditableField.isEditable(.init(role: "AXGroup", valueSettable: true)), "a writable value is editable")
check(EditableField.isEditable(.init(role: "AXGroup", editableFlag: true)), "a contenteditable group is editable")
check(EditableField.isEditable(.init(role: "AXWebArea", hasEditableAncestor: true)), "web content inside an editable ancestor is editable")
check(!EditableField.isEditable(.init(role: "AXStaticText")), "page text is not editable")
check(!EditableField.isEditable(.init()), "no focused role is not editable")

// Convert to Unicode through the core (pairs from crates/engine/tests/convert.rs). Scalars are compared because
// Swift's == treats "e" + U+0301 and "é" as equal.
let composedPair = CodeConvert.toUnicode("Tie\u{0301}ng")
check(composedPair.map { Array($0.unicodeScalars) } == Array("Ti\u{00E9}ng".unicodeScalars), "compound text becomes precomposed")
check(CodeConvert.toUnicode("Hello 123!") == "Hello 123!", "ASCII is untouched by the conversion")
check(UIStrings.tr("actions.loading.convert") != "actions.loading.convert", "convert loading text exists in UIStrings")
check(UIStrings.tr("actions.convert") == "Chuyển mã sang Unicode", "convert menu text")

// New settings: defaults come from the core, and both fields survive a save.
check(EVSettings.defaults.translateProvider == .google, "translate provider defaults to Google")
check(EVSettings.defaults.actionHotkey == "ctrl+alt+a", "action hotkey defaults to ctrl+alt+a")
let actionDir = FileManager.default.temporaryDirectory
    .appendingPathComponent("evkit-check-actions-\(UUID().uuidString)").path
do {
    try FileManager.default.createDirectory(atPath: actionDir, withIntermediateDirectories: true)
    var settings = EVSettings.defaults
    settings.translateProvider = .gemini
    settings.actionHotkey = "ctrl+alt+g"
    try SettingsStore.save(directory: actionDir, settings: settings, macros: [:])
    let loaded = try SettingsStore.load(directory: actionDir)
    check(loaded.settings.translateProvider == .gemini, "translate provider round-trips through core")
    check(loaded.settings.actionHotkey == "ctrl+alt+g", "action hotkey round-trips through core")
} catch {
    check(false, "action settings round-trip threw: \(error)")
}
try? FileManager.default.removeItem(atPath: actionDir)

runMiscChecks()

if failures > 0 {
    print("\(failures) check(s) failed")
    exit(1)
}
print("EVKeyKit self-check passed")
