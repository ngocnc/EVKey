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
            HStack {
                Text(UIStrings.tr("popup.header")).font(.system(size: 15, weight: .semibold))
                Spacer(minLength: 8)
                Toggle("", isOn: vietnameseBinding)
                    .labelsHidden()
                    .toggleStyle(SettingsSwitchStyle())
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)

            PopupChoiceSection(title: UIStrings.tr("popup.inputMethod"), choices: Self.inputChoices, selected: state.settings.inputType) { kind in
                state.updateSettings { $0.inputType = kind }
            }
            PopupChoiceSection(title: UIStrings.tr("popup.codeTable"), choices: Self.tableChoices, selected: state.settings.codeTable) { table in
                state.updateSettings { $0.codeTable = table }
            }

            Divider().padding(.vertical, 4)

            PopupRow(title: UIStrings.tr("popup.macros"), systemImage: "text.badge.plus") { openSettings(.macro) }
            PopupRow(title: UIStrings.tr("popup.controlPanel"), systemImage: "gearshape", hint: "⌘,") { openSettings(nil) }

            Divider().padding(.vertical, 4)

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
