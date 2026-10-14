import EVKeyKit
import SwiftUI

/// "Thông tin": app name and version, update check, GitHub link and credit.
struct SettingsInfoTab: View {
    @ObservedObject private var state = AppState.shared
    @ObservedObject private var updater = AppUpdater.shared

    /// Result of the last manual or automatic check; nil while idle.
    private var updateStatus: String? {
        switch updater.status {
        case .idle, .checking: return nil
        case .latest: return UIStrings.tr("update.latest")
        case .failed(let message): return UIStrings.tr("update.error", message)
        }
    }

    var body: some View {
        SettingsPage(title: UIStrings.tr("info.header")) {
            SettingsGroup {
                SettingsCard {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("EVKey")
                            .font(.system(size: 16, weight: .bold))
                        Text("v\(state.version)")
                            .font(.system(size: 12))
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 10)
                }
                SettingsActionRow(title: UIStrings.tr("update.checkNow"), subtitle: updateStatus) {
                    Button(UIStrings.tr("update.checkNow")) {
                        Task { await updater.check(manual: true) }
                    }
                    .disabled(updater.status == .checking)
                }
                SettingsToggleRow(
                    title: UIStrings.tr("update.autoTitle"),
                    subtitle: UIStrings.tr("update.autoDesc"),
                    isOn: state.binding(\.autoCheckUpdates))
                SettingsActionRow(title: "GitHub", subtitle: "github.com/ngocnc/EVKey") {
                    Button(UIStrings.tr("info.openGitHub"), action: state.openGitHub)
                }
            }
            Text(UIStrings.tr("info.developedBy"))
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .center)
                .padding(.top, 6)
        }
    }
}
