import SwiftUI

/// Hàng popup: icon tuỳ chọn, tiêu đề, gợi ý phím tắt bên phải. Màu chữ lấy từ `PopupRowStyle` (trắng khi rê chuột).
struct PopupRow: View {
    let title: String
    var systemImage: String? = nil
    var hint: String? = nil
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 8) {
                if let systemImage {
                    Image(systemName: systemImage)
                        .font(.system(size: 13))
                        .opacity(0.7)
                        .frame(width: 16)
                }
                Text(title)
                    .font(.system(size: 13))
                Spacer(minLength: 8)
                if let hint {
                    Text(hint)
                        .font(.system(size: 12, weight: .medium))
                        .opacity(0.65)
                }
            }
        }
        .buttonStyle(PopupRowStyle())
        .pointerCursor()
    }
}
