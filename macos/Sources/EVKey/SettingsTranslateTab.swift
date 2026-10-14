import SwiftUI

/// "Dịch thuật" (Beta): placeholder until the feature is built.
struct SettingsTranslateTab: View {
    var body: some View {
        SettingsPage(title: "Dịch thuật") {
            SettingsCard {
                Text("Đang phát triển")
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(14)
            }
        }
    }
}
