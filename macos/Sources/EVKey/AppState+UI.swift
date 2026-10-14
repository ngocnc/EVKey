import AppKit
import EVKeyKit
import SwiftUI

extension AppState {
    /// Applies an edit and saves at once, like the Windows settings window does on each change.
    func updateSettings(_ change: (inout EVSettings) -> Void) {
        change(&settings)
        do {
            try saveSettings()
        } catch {
            showError(error)
        }
    }

    /// Two-way binding to one settings field; every write saves the config.
    func binding<Value>(_ keyPath: WritableKeyPath<EVSettings, Value>) -> Binding<Value> {
        Binding(
            get: { self.settings[keyPath: keyPath] },
            set: { value in self.updateSettings { $0[keyPath: keyPath] = value } }
        )
    }

    /// Adds one custom consonant (a-z, stored lowercase) and saves at once. Returns a Vietnamese message when the letter is rejected.
    func addCustomConsonant(_ text: String) -> String? {
        let letter = text.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard letter.count == 1, letter.allSatisfy({ $0.isASCII && $0.isLowercase }) else { return "Chỉ nhập đúng một chữ cái a-z." }
        guard !settings.customConsonants.contains(letter) else { return "Phụ âm \(letter.uppercased()) đã có trong danh sách." }
        updateSettings { $0.customConsonants.append(letter) }
        return nil
    }

    /// Removes one custom consonant and saves at once.
    func removeCustomConsonant(_ letter: String) {
        updateSettings { $0.customConsonants.removeAll { $0 == letter } }
    }

    /// Runs a throwing action and reports a failure in an alert.
    func perform(_ action: () throws -> Void) {
        do {
            try action()
        } catch {
            showError(error)
        }
    }

    private func showError(_ error: Error) {
        let alert = NSAlert()
        alert.messageText = "Không lưu được cài đặt"
        alert.informativeText = error.localizedDescription
        alert.alertStyle = .warning
        alert.runModal()
    }
}
