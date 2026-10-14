import SwiftUI

/// Plain popup row: optional icon, title, and a right-aligned shortcut hint.
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
                        .foregroundStyle(.secondary)
                        .frame(width: 16)
                }
                Text(title)
                    .font(.system(size: 13))
                Spacer(minLength: 8)
                if let hint {
                    Text(hint)
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                }
            }
        }
            .pointerCursor()
        .buttonStyle(PopupRowStyle())
    }
}
