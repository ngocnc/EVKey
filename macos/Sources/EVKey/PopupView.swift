import AppKit
import EVKeyKit
import SwiftUI

/// Menu bar popup: mode switch, input method and code table pickers, and entry points to the control panel.
struct PopupView: View {
    @ObservedObject private var state = AppState.shared

    private static var inputChoices: [ChoiceOption<InputKind>] {
        [
            ChoiceOption(value: .telex, label: "Telex"),
            ChoiceOption(value: .vni, label: "VNI"),
            ChoiceOption(value: .simpleTelex1, label: "Simple Telex 1"),
            ChoiceOption(value: .simpleTelex2, label: "Simple Telex 2"),
        ]
    }

    private static var tableChoices: [ChoiceOption<CodeTable>] {
        [
            ChoiceOption(value: .unicode, label: "Unicode"),
            ChoiceOption(value: .tcvn3, label: "TCVN3 (ABC)"),
            ChoiceOption(value: .vniWindows, label: "VNI Windows"),
            ChoiceOption(value: .unicodeCompound, label: UIStrings.tr("codeTable.unicodeCompound")),
        ]
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 10) {
                LogoMark(letter: state.isEnglish ? "E" : "V", size: 30)
                VStack(alignment: .leading, spacing: 1) {
                    Text(UIStrings.tr("popup.header")).font(.system(size: 13, weight: .semibold))
                    Text("\(currentLabel(Self.inputChoices, state.settings.inputType)) · \(currentLabel(Self.tableChoices, state.settings.codeTable))")
                        .font(.system(size: 11.5))
                        .foregroundStyle(.secondary)
                }
                Spacer(minLength: 8)
                Toggle("", isOn: vietnameseBinding)
                    .labelsHidden()
                    .toggleStyle(SettingsSwitchStyle())
            }
            .padding(.horizontal, 10)
            .padding(.top, 8)
            .padding(.bottom, 10)

            if let app = excludedApp {
                HStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundStyle(.orange)
                    Text(UIStrings.tr("popup.excluded", app))
                        .font(.system(size: 11.5))
                        .fixedSize(horizontal: false, vertical: true)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 7)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.orange.opacity(0.14), in: RoundedRectangle(cornerRadius: 9, style: .continuous))
                .padding(.horizontal, 4)
                .padding(.bottom, 6)
            }

            Divider().padding(.horizontal, 8).padding(.bottom, 2)

            PopupChoiceSection(
                title: UIStrings.tr("popup.inputMethod"), choices: Self.inputChoices,
                selected: state.settings.inputType, collapsible: false
            ) { kind in
                state.updateSettings { $0.inputType = kind }
            }

            Divider().padding(.horizontal, 8).padding(.vertical, 4)

            PopupChoiceSection(title: UIStrings.tr("popup.codeTable"), choices: Self.tableChoices, selected: state.settings.codeTable) { table in
                state.updateSettings { $0.codeTable = table }
            }

            Divider().padding(.horizontal, 8).padding(.vertical, 4)

            PopupRow(title: UIStrings.tr("popup.macros"), systemImage: "text.badge.plus") { openSettings(.macro) }
            PopupRow(title: UIStrings.tr("popup.controlPanel"), systemImage: "gearshape", hint: "⌘,") { openSettings(nil) }
            PopupRow(title: UIStrings.tr("popup.quit"), systemImage: "power", hint: "⌘Q") { NSApp.terminate(nil) }
        }
        .padding(6)
        .frame(width: 308)
        .glassSurface(cornerRadius: 14)
        // Taps on the glass that miss a button are swallowed; taps on the transparent area below it close the popup.
        .onTapGesture {}
        .padding(6)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .contentShape(Rectangle())
        .onTapGesture { PopupController.shared.hide() }

        .background(shortcuts)
        .noFocusRing()
    }

    private var vietnameseBinding: Binding<Bool> {
        Binding(get: { !state.isEnglish }, set: { state.setEnglish(!$0) })
    }

    /// Tên ứng dụng phía trước khi nó nằm trong danh sách loại trừ.
    private var excludedApp: String? {
        guard let app = state.lastApp, state.settings.excludedAppsMacos.contains(app.bundleID) else { return nil }
        return app.name
    }

    private func currentLabel<V: Hashable>(_ choices: [ChoiceOption<V>], _ value: V) -> String {
        choices.first { $0.value == value }?.label ?? ""
    }

    private func openSettings(_ tab: SettingsTab?) {
        PopupController.shared.hide()
        SettingsWindowController.shared.show(tab: tab)
    }

    /// Shortcuts only fire while the popup is key. Hidden zero-size buttons carry them.
    private var shortcuts: some View {
        VStack {
            Button("") { openSettings(nil) }
                .keyboardShortcut(",", modifiers: .command)
            Button("") { NSApp.terminate(nil) }
                .keyboardShortcut("q", modifiers: .command)
            Button("") { PopupController.shared.hide() }
                .keyboardShortcut(.cancelAction)
        }
        .frame(width: 0, height: 0)
        .clipped()
        .opacity(0)
    }
}
