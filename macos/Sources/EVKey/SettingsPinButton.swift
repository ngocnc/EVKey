import EVKeyKit
import SwiftUI

/// Pin toggle for the Bảng điều khiển corner: keeps that window above others. Session only; the window level is the source of truth.
struct SettingsPinButton: View {
    // Starts pinned: the window controller sets `.floating` at creation. Touching `.shared` here would recurse, because
    // the controller builds this view while it is still initialising.
    @State private var pinned = true

    var body: some View {
        Button {
            pinned.toggle()
            SettingsWindowController.shared.setPinned(pinned)
        } label: {
            Image(systemName: pinned ? "pin.fill" : "pin")
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(pinned ? Color.accentColor : Color.primary.opacity(0.7))
                .frame(width: 26, height: 26)
                .background(pinned ? Color.accentColor.opacity(0.15) : Color.clear, in: Circle())
        }
        .buttonStyle(.plain)
        .frame(height: 28)
        .help(pinned ? UIStrings.tr("pin.unpin") : UIStrings.tr("pin.pin"))
        .animation(.easeInOut(duration: 0.15), value: pinned)
    }
}
