import EVKeyKit
import SwiftUI

/// "Gõ tắt": editable abbreviation table. Typing in the last row appends an empty one (no "add row" button), deleting a
/// stored row writes the file at once, and Save needs both fields of every row that has any text. Rows are validated and
/// saved by `AppState.saveMacros`.
struct SettingsMacroTab: View {
    @ObservedObject private var state = AppState.shared
    @State private var rows: [MacroRow] = []
    @State private var error: String?

    private struct MacroRow: Identifiable {
        let id = UUID()
        var key: String
        var value: String
        /// The abbreviation as stored on disk; nil for a row that was never saved.
        var saved: String?
    }

    /// Rows whose empty field blocked the last Save.
    @State private var incomplete: Set<UUID> = []

    private func isBlank(_ s: String) -> Bool { s.trimmingCharacters(in: .whitespaces).isEmpty }

    var body: some View {
        SettingsPage(
            title: UIStrings.tr("tab.macro"),
            note: UIStrings.tr("macro.note")
        ) {
            SettingsCard {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach($rows) { $row in
                        HStack(spacing: 8) {
                            TextField(UIStrings.tr("macro.key"), text: $row.key)
                                .textFieldStyle(.roundedBorder)
                                .overlay(invalidOutline(row, empty: isBlank(row.key)))
                                .frame(width: 180)
                            TextField(UIStrings.tr("macro.value"), text: $row.value)
                                .textFieldStyle(.roundedBorder)
                                .overlay(invalidOutline(row, empty: isBlank(row.value)))
                            Button(UIStrings.tr("macro.delete")) { delete(row) }
                        }
                        .transition(.opacity)
                    }
                    HStack {
                        Spacer()
                        Button(UIStrings.tr("macro.save"), action: save)
                            .keyboardShortcut(.defaultAction)
                    }
                    .padding(.top, 4)
                    if let error {
                        Text(error)
                            .font(.system(size: 12))
                            .foregroundStyle(.red)
                    }
                }
                .padding(14)
            }
            SettingsAdvanced(pane: "macro") {
                SettingsMacroImportRow(onImported: reload)
            }
        }
        .onAppear(perform: reload)
        // Typing in the last row appends a fresh empty one.
        .onChange(of: rows.last.map { !isBlank($0.key) || !isBlank($0.value) } ?? true) {
            withAnimation(.easeInOut(duration: 0.2)) { rows.append(MacroRow(key: "", value: "")) }
        }
    }

    @ViewBuilder
    private func invalidOutline(_ row: MacroRow, empty: Bool) -> some View {
        if empty && incomplete.contains(row.id) {
            RoundedRectangle(cornerRadius: 5).stroke(Color.red, lineWidth: 1.5)
        }
    }

    private func reload() {
        rows = state.macros.sorted { $0.key < $1.key }.map { MacroRow(key: $0.key, value: $0.value, saved: $0.key) }
        rows.append(MacroRow(key: "", value: ""))
        incomplete = []
    }

    /// Removing a stored row takes effect at once, without pressing Save.
    private func delete(_ row: MacroRow) {
        withAnimation(.easeInOut(duration: 0.2)) { rows.removeAll { $0.id == row.id } }
        if rows.isEmpty { rows.append(MacroRow(key: "", value: "")) }
        guard let saved = row.saved else { return }
        do {
            try state.saveMacros(state.macros.filter { $0.key != saved }.map { (key: $0.key, value: $0.value) })
            error = nil
        } catch {
            self.error = error.localizedDescription
            reload()
        }
    }

    private func save() {
        var filled: [(key: String, value: String)] = []
        var bad: Set<UUID> = []
        var firstBad = 0
        for (index, row) in rows.enumerated() {
            let (noKey, noValue) = (isBlank(row.key), isBlank(row.value))
            if noKey != noValue {
                bad.insert(row.id)
                if firstBad == 0 { firstBad = index + 1 }
            } else if !noKey {
                filled.append((key: row.key, value: row.value))
            }
        }
        incomplete = bad
        guard bad.isEmpty else {
            error = UIStrings.tr("macro.rowIncomplete", String(firstBad))
            return
        }
        do {
            try state.saveMacros(filled)
            error = nil
            reload()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
