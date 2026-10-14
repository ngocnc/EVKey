import SwiftUI

/// Outcome line under a backup or import row: green for success, red for a failure.
struct SettingsResultText: View {
    let text: String
    let isError: Bool

    var body: some View {
        Text(text)
            .font(.system(size: 12))
            .foregroundStyle(isError ? Color.red : Color.green)
            .fixedSize(horizontal: false, vertical: true)
    }
}
