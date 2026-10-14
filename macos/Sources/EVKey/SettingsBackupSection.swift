import AppKit
import EVKeyKit
import SwiftUI
import UniformTypeIdentifiers

/// Export and import of settings plus macros as one TOML file (same format as the Windows export).
struct SettingsBackupSection: View {
    @ObservedObject private var state = AppState.shared
    @State private var message: String?
    @State private var failed = false

    private static var tomlTypes: [UTType] { [UTType(filenameExtension: "toml") ?? .plainText] }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            SettingsHeading(UIStrings.misc("backup.heading"))
            SettingsGroup {
                SettingsActionRow(
                    title: UIStrings.misc("backup.exportTitle"), subtitle: UIStrings.misc("backup.exportDesc")
                ) {
                    Button(UIStrings.misc("backup.exportButton"), action: export)
                }
                SettingsActionRow(
                    title: UIStrings.misc("backup.importTitle"), subtitle: UIStrings.misc("backup.importDesc")
                ) {
                    Button(UIStrings.misc("backup.importButton"), action: importConfig)
                }
            }
            if let message {
                SettingsResultText(text: message, isError: failed)
            }
        }
    }

    private func export() {
        let panel = NSSavePanel()
        panel.nameFieldStringValue = "EVKey-config.toml"
        panel.allowedContentTypes = Self.tomlTypes
        panel.canCreateDirectories = true
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try state.exportConfig(to: url)
            show(UIStrings.misc("backup.exportDone", url.path))
        } catch {
            show(error.localizedDescription, failed: true)
        }
    }

    private func importConfig() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = Self.tomlTypes
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        guard panel.runModal() == .OK, let url = panel.url else { return }
        let alert = NSAlert()
        alert.messageText = UIStrings.misc("backup.importConfirm")
        alert.alertStyle = .warning
        alert.addButton(withTitle: UIStrings.misc("backup.importAction"))
        alert.addButton(withTitle: UIStrings.tr("common.cancel"))
        guard alert.runModal() == .alertFirstButtonReturn else { return }
        do {
            try state.importConfig(from: url)
            show(UIStrings.misc("backup.importDone"))
        } catch {
            show(error.localizedDescription, failed: true)
        }
    }

    private func show(_ text: String, failed: Bool = false) {
        message = text
        self.failed = failed
    }
}
