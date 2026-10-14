import EVKeyKit
import SwiftUI

/// "Thông tin": app name and version, update check (not yet available), GitHub link and credit.
struct SettingsInfoTab: View {
    @ObservedObject private var state = AppState.shared

    var body: some View {
        SettingsPage(title: "Thông tin") {
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
                SettingsActionRow(title: "Kiểm tra cập nhật", subtitle: "Đang phát triển") {
                    Button("Sắp có") {}
                        .pointerCursor()
                        .disabled(true)
                }
                SettingsActionRow(title: "GitHub", subtitle: "github.com/ngocnc/EVKey") {
                    Button("Mở trang GitHub", action: state.openGitHub)
                        .pointerCursor()
                }
            }
            Text("Phát triển bởi ChrisNguyen")
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .center)
                .padding(.top, 6)
        }
    }
}
