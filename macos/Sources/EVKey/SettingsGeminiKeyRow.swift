import EVKeyKit
import SwiftUI

/// The Gemini API key: paste it, "Kiểm tra & lưu" tests it with Gemini and stores it in the Keychain, "Xóa key" removes it.
/// The saved key is never shown again.
struct SettingsGeminiKeyRow: View {
    @ObservedObject private var state = AppState.shared
    @State private var key = ""
    @State private var busy = false
    /// The last outcome (checking, saved, deleted or an error); nil falls back to whether a key is saved.
    @State private var outcome: (text: String, isError: Bool)?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SettingsRowLabel(title: UIStrings.tr("gemini.key"), subtitle: UIStrings.tr("gemini.key.detail"))
            HStack(spacing: 8) {
                SecureField(
                    state.geminiKeySaved ? UIStrings.tr("gemini.key.placeholderSet") : UIStrings.tr("gemini.key.placeholderUnset"),
                    text: $key)
                    .textFieldStyle(.roundedBorder)
                    .disableAutocorrection(true)
                    .onSubmit(save)
                Button(UIStrings.tr("gemini.key.save"), action: save)
                    .buttonStyle(.borderedProminent)
                    .disabled(busy || key.trimmingCharacters(in: .whitespaces).isEmpty)
                Button(UIStrings.tr("gemini.key.delete"), role: .destructive, action: delete)
                    .disabled(busy || !state.geminiKeySaved)
            }
            Text(statusText)
                .font(.system(size: 11))
                .foregroundStyle(outcome?.isError == true ? Color.red : Color.primary.opacity(0.65))
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .settingsBlock()
    }

    private var statusText: String {
        if let outcome { return outcome.text }
        return UIStrings.tr(state.geminiKeySaved ? "gemini.key.statusSet" : "gemini.key.statusUnset")
    }

    private func save() {
        guard !busy else { return }
        busy = true
        outcome = (UIStrings.tr("gemini.key.checking"), false)
        Task { @MainActor in
            do {
                try await state.saveGeminiKey(key)
                key = ""
                outcome = (UIStrings.tr("gemini.key.saved"), false)
            } catch {
                outcome = (error.localizedDescription, true)
            }
            busy = false
        }
    }

    private func delete() {
        do {
            try state.deleteGeminiKey()
            outcome = (UIStrings.tr("gemini.key.deleted"), false)
        } catch {
            outcome = (error.localizedDescription, true)
        }
    }
}
