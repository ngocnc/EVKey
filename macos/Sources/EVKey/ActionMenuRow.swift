import SwiftUI

/// Một hàng của bảng hành động: ô icon màu, nhãn, badge AI tuỳ chọn. Rê chuột thì nền `brand-fill` chữ trắng; mờ và không
/// bấm được khi tắt.
struct ActionMenuRow: View {
    let symbol: String
    let tile: [Color]
    let title: String
    var ai = false
    var enabled = true
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                ActionTile(symbol: symbol, colors: tile)
                Text(title).font(.system(size: 13.5))
                Spacer(minLength: 0)
                if ai { AIBadge() }
            }
        }
        .buttonStyle(PopupRowStyle())
        .disabled(!enabled)
        .opacity(enabled ? 1 : 0.4)
        .pointerCursor()
    }
}

/// Ô vuông 22pt bo 6, nền gradient màu riêng của từng hành động, icon trắng.
struct ActionTile: View {
    let symbol: String
    let colors: [Color]

    var body: some View {
        RoundedRectangle(cornerRadius: 6, style: .continuous)
            .fill(LinearGradient(colors: colors, startPoint: .topLeading, endPoint: .bottomTrailing))
            .overlay {
                Image(systemName: symbol)
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(.white)
            }
            .frame(width: 22, height: 22)
            .shadow(color: .black.opacity(0.15), radius: 1, y: 1)
    }
}

/// Badge "AI" nền `brand-fill`.
struct AIBadge: View {
    var body: some View {
        Text("AI")
            .font(.system(size: 10, weight: .bold))
            .foregroundStyle(.white)
            .padding(.horizontal, 6)
            .padding(.vertical, 1)
            .background { BrandFill().clipShape(Capsule()) }
    }
}
