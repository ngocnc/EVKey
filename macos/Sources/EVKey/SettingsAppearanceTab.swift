import AppKit
import EVKeyKit
import SwiftUI

/// "Giao diện": Liquid Glass of the popup and Bảng điều khiển, and the Secure Input banner look.
struct SettingsAppearanceTab: View {
    @ObservedObject private var state = AppState.shared

    var body: some View {
        SettingsPage(title: UIStrings.tr("appearance.header")) {
            SettingsHeading(UIStrings.tr("appearance.liquidGlass"))
            SettingsGroup { SettingsGlassSection() }
            SettingsAdvanced(pane: "appearance") {
                SettingsGroup { SettingsGlassSection(advanced: true) }
                SettingsHeading(UIStrings.tr("appearance.secureBanner"))
                SettingsGroup { SettingsBannerSection() }
                HStack {
                    Spacer()
                    Button(UIStrings.tr("appearance.reset"), role: .destructive) {
                        let alert = NSAlert()
                        alert.messageText = UIStrings.tr("appearance.reset.title")
                        alert.informativeText = UIStrings.tr("appearance.reset.message")
                        alert.alertStyle = .warning
                        alert.addButton(withTitle: UIStrings.tr("appearance.reset.confirm"))
                        alert.addButton(withTitle: UIStrings.tr("common.cancel"))
                        guard alert.runModal() == .alertFirstButtonReturn else { return }
                        GlassAppearance.shared.reset()
                        SettingsBannerSection.reset()
                    }
                }
            }
        }
    }
}
