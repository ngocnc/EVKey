import EVKeyKit
import SwiftUI

/// "Chính tả & gõ nhanh": spelling check, quick consonants and capitalisation.
struct SettingsSmartTab: View {
    @ObservedObject private var state = AppState.shared

    var body: some View {
        SettingsPage(title: "Chính tả & gõ nhanh") {
            SettingsGroup {
                SettingsCard {
                    SettingsToggleLine(
                        title: "Kiểm tra chính tả", subtitle: "Chỉ biến đổi khi âm tiết hợp lệ.",
                        isOn: state.binding(\.checkSpelling))
                    SettingsDivider()
                    SettingsToggleLine(
                        title: "Khôi phục khi sai chính tả", subtitle: "Gõ window thì giữ nguyên, không biến thành chữ lạ.",
                        isOn: state.binding(\.restoreIfWrongSpelling))
                }
            }

            SettingsHeading("Gõ nhanh")
            SettingsGroup {
                SettingsToggleRow(
                    title: "Telex nhanh", subtitle: "cc→ch, gg→gi, kk→kh, nn→ng, qq→qu, pp→ph, tt→th",
                    isOn: state.binding(\.quickTelex))
                SettingsToggleRow(
                    title: "Phụ âm đầu nhanh", subtitle: "f→ph, j→gi, w→qu",
                    isOn: state.binding(\.quickStartConsonant))
                SettingsToggleRow(
                    title: "Phụ âm cuối nhanh", subtitle: "g→ng, h→nh, k→ch",
                    isOn: state.binding(\.quickEndConsonant))
                SettingsCard {
                    SettingsToggleLine(
                        title: "Cho phép phụ âm tùy chỉnh (mặc định: Z,F,W,J)",
                        isOn: state.binding(\.allowConsonantZfwj))
                    if state.settings.allowConsonantZfwj {
                        SettingsDivider()
                        SettingsConsonantChips()
                    }
                }
            }

            SettingsGroup {
                SettingsToggleRow(
                    title: "Chế độ theo ứng dụng",
                    subtitle: "Nhớ chế độ Việt/Anh của từng ứng dụng và khôi phục khi quay lại ứng dụng đó.",
                    isOn: state.binding(\.rememberAppMode))
                SettingsToggleRow(
                    title: "Hoàn tác từ bằng Esc",
                    subtitle: "Khi đang gõ dở một từ, Esc trả về các phím đã gõ và tắt biến đổi cho từ đó.",
                    isOn: state.binding(\.undoWord))
            }

            SettingsHeading("Viết hoa")
            SettingsGroup {
                SettingsToggleRow(
                    title: "Tự viết hoa đầu câu", subtitle: "Viết hoa chữ cái đầu sau dấu chấm.",
                    isOn: state.binding(\.upperCaseFirstChar))
                SettingsToggleRow(
                    title: "Gõ tắt theo kiểu chữ", subtitle: "Btw thành By the way, BTW thành BY THE WAY.",
                    isOn: state.binding(\.autoCapsMacro))
            }
        }
    }
}
