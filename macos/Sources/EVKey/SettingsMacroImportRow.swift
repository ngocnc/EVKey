import AppKit
import EVKeyKit
import SwiftUI

/// Import of a UniKey / OpenKey macro file. "Replace all" drops the current macros first (after a confirmation);
/// otherwise imported abbreviations overwrite same-named ones and the rest stay.
struct SettingsMacroImportRow: View {
    @ObservedObject private var state = AppState.shared
    /// Called after a successful import so the macro table shows the new rows.
    let onImported: () -> Void
    @State private var replace = false
    @State private var message: String?
    @State private var failed = false

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            SettingsActionRow(title: UIStrings.misc("macroImport.title"), subtitle: UIStrings.misc("macroImport.desc")) {
                HStack(spacing: 12) {
                    Toggle(UIStrings.misc("macroImport.replace"), isOn: $replace)
                        .toggleStyle(.checkbox)
                    Button(UIStrings.misc("macroImport.button"), action: pick)
                        .buttonStyle(PressButtonStyle())
                }
            }
            if let message {
                SettingsResultText(text: message, isError: failed)
            }
        }
    }

    private func pick() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        guard panel.runModal() == .OK, let url = panel.url else { return }
        if replace {
            let alert = NSAlert()
            alert.messageText = UIStrings.misc("macroImport.confirm")
            alert.alertStyle = .warning
            alert.addButton(withTitle: UIStrings.misc("macroImport.replaceAction"))
            alert.addButton(withTitle: UIStrings.tr("common.cancel"))
            guard alert.runModal() == .alertFirstButtonReturn else { return }
        }
        do {
            let count = try state.importMacros(from: url, replace: replace)
            message = UIStrings.misc("macroImport.done", count)
            failed = false
            onImported()
        } catch {
            message = error.localizedDescription
            failed = true
        }
    }
}
