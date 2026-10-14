import EVKeyKit
import SwiftUI

/// "Gõ tắt": editable abbreviation table. Rows are validated and saved by `AppState.saveMacros`.
struct SettingsMacroTab: View {
    @ObservedObject private var state = AppState.shared
    @State private var rows: [MacroRow] = []
    @State private var error: String?

    private struct MacroRow: Identifiable {
        let id = UUID()
        var key: String
        var value: String
    }

    var body: some View {
        SettingsPage(
            title: "Gõ tắt",
            note: "Từ viết tắt được thay bằng nội dung khi gõ. Không để trống và không được trùng nhau."
        ) {
            SettingsCard {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach($rows) { $row in
                        HStack(spacing: 8) {
                            TextField("Từ viết tắt", text: $row.key)
                                .textFieldStyle(.roundedBorder)
                                .frame(width: 180)
                            TextField("Nội dung thay thế", text: $row.value)
                                .textFieldStyle(.roundedBorder)
                            Button("Xóa") {
                                let id = row.id
                                withAnimation(.easeInOut(duration: 0.2)) { rows.removeAll { $0.id == id } }
                            }
                                .pointerCursor()
                        }
                        .transition(.opacity)
                    }
                    HStack {
                        Button("Thêm dòng") {
                            withAnimation(.easeInOut(duration: 0.2)) { rows.append(MacroRow(key: "", value: "")) }
                        }
                            .pointerCursor()
                        Spacer()
                        Button("Lưu gõ tắt", action: save)
                            .pointerCursor()
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
        }
        .onAppear(perform: reload)
    }

    private func reload() {
        rows = state.macros.sorted { $0.key < $1.key }.map { MacroRow(key: $0.key, value: $0.value) }
        if rows.isEmpty { rows.append(MacroRow(key: "", value: "")) }
    }

    private func save() {
        let filled = rows
            .map { (key: $0.key, value: $0.value) }
            .filter { !$0.key.trimmingCharacters(in: .whitespaces).isEmpty || !$0.value.trimmingCharacters(in: .whitespaces).isEmpty }
        do {
            try state.saveMacros(filled)
            error = nil
            reload()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
