import AppKit
import SwiftUI

/// Pointing-hand cursor while the mouse is over a clickable control.
private struct PointerCursor: ViewModifier {
    @State private var inside = false

    func body(content: Content) -> some View {
        content
            .onHover { hovering in
                if hovering, !inside {
                    NSCursor.pointingHand.push()
                    inside = true
                } else if !hovering, inside {
                    NSCursor.pop()
                    inside = false
                }
            }
            // A control that disappears under the mouse never reports the exit, so release the cursor here.
            .onDisappear {
                if inside {
                    NSCursor.pop()
                    inside = false
                }
            }
    }
}

extension View {
    func pointerCursor() -> some View { modifier(PointerCursor()) }
}
