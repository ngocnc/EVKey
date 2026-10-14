import EVKeyKit
import SwiftUI

/// Title, subtitle and a one-line field of space- or comma-separated words (`restore_english` lists).
/// Saved on Return or when the field loses focus, so typing a space does not rewrite the text.
struct SettingsWordListLine: View {
    let title: String
    let subtitle: String
    let placeholder: String
    let keyPath: WritableKeyPath<EVSettings, [String]>
    @ObservedObject private var state = AppState.shared
    @State private var text = ""
    @FocusState private var focused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsRowLabel(title: title, subtitle: subtitle)
            TextField(placeholder, text: $text)
                .textFieldStyle(.roundedBorder)
                .focused($focused)
                .onSubmit(commit)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .onAppear { text = state.settings[keyPath: keyPath].joined(separator: " ") }
        .onChange(of: focused) { isFocused in
            if !isFocused { commit() }
        }
    }

    private func commit() {
        let words = text.split(whereSeparator: { $0 == " " || $0 == "," }).map(String.init)
        state.updateSettings { $0[keyPath: keyPath] = words }
    }
}
