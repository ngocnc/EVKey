import AppKit
import EVKeyKit
import SwiftUI

/// Owns the update window. It is created once and reused; closing only hides it.
@MainActor
final class UpdateWindowController {
    static let shared = UpdateWindowController()

    private let window: NSWindow

    private init() {
        let hosting = NSHostingController(rootView: UpdateView())
        window = NSWindow(contentViewController: hosting)
        window.styleMask = [.titled, .closable, .fullSizeContentView]
        window.titlebarAppearsTransparent = true
        window.isOpaque = false
        window.backgroundColor = .clear
        window.setContentSize(NSSize(width: 440, height: 400))
        window.isReleasedWhenClosed = false
        window.center()
        window.level = .floating
    }

    /// Shows the window above other apps. Called when a newer release is found or the user asks to check.
    func show() {
        window.title = UIStrings.tr("update.title")
        NSApp.activate(ignoringOtherApps: true)
        window.makeKeyAndOrderFront(nil)
    }

    func close() {
        window.orderOut(nil)
    }
}

/// Release notes, the install buttons and any install error for the pending release.
struct UpdateView: View {
    @ObservedObject private var updater = AppUpdater.shared
    @ObservedObject private var state = AppState.shared

    @Environment(\.colorScheme) private var scheme

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(spacing: 14) {
                LogoMark(letter: "V", size: 50)
                VStack(alignment: .leading, spacing: 3) {
                    Text(UIStrings.tr("update.title"))
                        .font(.system(size: 17, weight: .bold))
                    if let version = updater.pending?.version {
                        Text(UIStrings.tr("update.version", version) + " · " + UIStrings.tr("update.current", state.version))
                            .font(.system(size: 12))
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .padding(.top, 14)

            Text(UIStrings.tr("update.notes").uppercased())
                .font(.system(size: 11, weight: .semibold))
                .tracking(0.4)
                .foregroundStyle(.secondary)

            SettingsCard {
                ScrollView {
                    Text(notesText)
                        .font(.system(size: 12.5))
                        .lineSpacing(3)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .textSelection(.enabled)
                        .padding(14)
                }
                .frame(maxHeight: .infinity)
            }
            .frame(maxHeight: .infinity)

            if let error = updater.installError {
                Text(error)
                    .font(.system(size: 12))
                    .foregroundStyle(Tokens(scheme).danger)
                    .fixedSize(horizontal: false, vertical: true)
            }

            HStack {
                Button(UIStrings.tr("update.skip")) {
                    if let version = updater.pending?.version {
                        state.updateSettings { $0.skippedUpdate = version }
                    }
                    UpdateWindowController.shared.close()
                }
                .buttonStyle(PressButtonStyle())
                Spacer()
                Button(UIStrings.tr("update.later")) {
                    UpdateWindowController.shared.close()
                }
                .buttonStyle(PressButtonStyle())
                Button(installTitle) {
                    Task { await updater.install() }
                }
                .buttonStyle(PressButtonStyle(primary: true))
                .keyboardShortcut(.defaultAction)
                .disabled(updater.installing || updater.pending == nil)
            }
        }
        .padding(20)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .glassBackdrop()
        .noFocusRing()
    }

    private var notesText: String {
        guard let notes = updater.pending?.notes, !notes.isEmpty else {
            return UIStrings.tr("update.noNotes")
        }
        return notes
    }

    private var installTitle: String {
        updater.installing ? UIStrings.tr("update.downloading") : UIStrings.tr("update.install")
    }
}
