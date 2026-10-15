import AppKit
import SwiftUI

/// Background that moves its window when dragged. SwiftUI content swallows mouse-down, so
/// `isMovableByWindowBackground` alone does nothing; controls drawn above it still get their clicks.
struct WindowDragArea: NSViewRepresentable {
    final class DragView: NSView {
        override func mouseDown(with event: NSEvent) { window?.performDrag(with: event) }
        override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
    }

    func makeNSView(context: Context) -> DragView { DragView() }
    func updateNSView(_ nsView: DragView, context: Context) {}
}
