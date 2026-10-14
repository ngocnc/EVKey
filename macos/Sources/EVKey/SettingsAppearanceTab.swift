import AppKit
import SwiftUI

/// "Giao diện": Liquid Glass of the popup and Bảng điều khiển, and the Secure Input banner look.
struct SettingsAppearanceTab: View {
    var body: some View {
        SettingsPage(title: "Giao diện") {
            SettingsHeading("Liquid Glass")
            SettingsGroup { SettingsGlassSection() }
            SettingsHeading("Cảnh báo Secure Input")
            SettingsGroup { SettingsBannerSection() }
            HStack {
                Spacer()
                Button("Đặt lại giao diện", role: .destructive) {
                    let alert = NSAlert()
                    alert.messageText = "Đặt lại giao diện về mặc định?"
                    alert.informativeText = "Kiểu kính, màu phủ và giao diện banner Secure Input sẽ về giá trị ban đầu."
                    alert.alertStyle = .warning
                    alert.addButton(withTitle: "Đặt lại")
                    alert.addButton(withTitle: "Hủy")
                    guard alert.runModal() == .alertFirstButtonReturn else { return }
                    GlassAppearance.shared.reset()
                    SettingsBannerSection.reset()
                }
            }
        }
    }
}
