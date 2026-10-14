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
check(KeyMap.vk(forKeycode: 122) == nil, "F1 is unmodeled")
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

if failures > 0 {
    print("\(failures) check(s) failed")
    exit(1)
}
print("EVKeyKit self-check passed")
