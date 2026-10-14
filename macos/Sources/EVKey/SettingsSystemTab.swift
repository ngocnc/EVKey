import AppKit
import EVKeyKit
import SwiftUI

/// "Hệ thống": launch at login, config folder and the about section.
struct SettingsSystemTab: View {
    @ObservedObject private var state = AppState.shared

    var body: some View {
        SettingsPage(title: "Hệ thống") {
            SettingsGroup {
                SettingsToggleRow(
                    title: "Tự chạy khi đăng nhập",
                    subtitle: "EVKey khởi động ẩn trên thanh menu.",
                    isOn: Binding(
                        get: { state.autostart },
                        set: { value in state.perform { try state.setAutostart(value) } }))
                SettingsToggleRow(
                    title: "Phát âm khi chuyển chế độ",
                    subtitle: "Âm khác nhau cho Tiếng Việt và Tiếng Anh, chỉ khi bạn tự chuyển.",
                    isOn: state.binding(\.toggleSound))
            }

            SettingsHeading("Tệp cấu hình")
            SettingsCard {
                VStack(alignment: .leading, spacing: 10) {
                    Text(state.configDir)
                        .font(.system(size: 12, design: .monospaced))
                        .textSelection(.enabled)
                        .foregroundStyle(.secondary)
                    HStack {
                        Button("Mở thư mục", action: state.openConfigDir)
                            .pointerCursor()
                        Spacer()
                        Button("Đặt lại về mặc định", role: .destructive) {
                            if confirmResetConfig() { state.perform { try state.resetConfig() } }
                        }
                            .pointerCursor()
                    }
                }
                .padding(14)
            }
        }
    }
}

/// Asks before the config is replaced by defaults. Returns true when the user confirms.
@MainActor
func confirmResetConfig() -> Bool {
    let alert = NSAlert()
    alert.messageText = "Đặt lại cài đặt về mặc định?"
    alert.informativeText = "Mọi tùy chỉnh hiện tại sẽ bị thay thế bằng giá trị mặc định."
    alert.alertStyle = .warning
    alert.addButton(withTitle: "Đặt lại")
    alert.addButton(withTitle: "Hủy")
    return alert.runModal() == .alertFirstButtonReturn
}
