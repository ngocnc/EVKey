import EVKeyKit
import SwiftUI

/// "Ứng dụng loại trừ": bundle IDs where EVKey never converts keys. Saved when the editor loses focus.
struct SettingsAppsTab: View {
    @ObservedObject private var state = AppState.shared
    @State private var text = ""
    @State private var notice: String?
    @FocusState private var focused: Bool

    var body: some View {
        SettingsPage(
            title: UIStrings.tr("tab.apps"),
            note: UIStrings.tr("apps.note")
        ) {
            SettingsGroup {
                ZStack(alignment: .topLeading) {
                    TextEditor(text: $text)
                        .font(.system(size: 12, design: .monospaced))
                        .focused($focused)
                        .scrollContentBackground(.hidden)
                        .padding(6)
                        .background(Color.black.opacity(0.2), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                        .frame(height: 220)
                    if text.isEmpty {
                        Text("com.apple.Safari")
                            .font(.system(size: 12, design: .monospaced))
                            .foregroundStyle(.secondary)
                            .padding(.top, 12)
                            .padding(.leading, 11)
                            .allowsHitTesting(false)
                    }
                }
                .padding(12)
                .settingsBlock()
                HStack(spacing: 10) {
                    Menu(UIStrings.tr("apps.addRunning")) {
                        ForEach(state.runningApps()) { app in
                            Button(app.name) { add(app.bundleID) }
                        }
                    }
                    .fixedSize()
                    Button(UIStrings.tr("apps.addRecent"), action: addLastApp)
                    Spacer()
                }
                .padding(12)
                .settingsBlock()
            }
            if let notice {
                Text(notice)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
        }
        .onAppear { text = state.settings.excludedAppsMacos.joined(separator: "\n") }
        .onChange(of: focused) { isFocused in
            if !isFocused { commit() }
        }
    }

    private var entries: [String] {
        text.split(separator: "\n").map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
    }

    private func commit() {
        let list = entries
        state.updateSettings { $0.excludedAppsMacos = list }
    }

    private func add(_ bundleID: String) {
        var list = entries
        if !list.contains(where: { $0.caseInsensitiveCompare(bundleID) == .orderedSame }) {
            list.append(bundleID)
        }
        text = list.joined(separator: "\n")
        state.updateSettings { $0.excludedAppsMacos = list }
    }

    private func addLastApp() {
        guard let app = state.lastApp else {
            notice = UIStrings.tr("apps.empty")
            return
        }
        notice = nil
        add(app.bundleID)
    }
}
