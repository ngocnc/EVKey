import AppKit
import SwiftUI

/// Moves its window when dragged, by hand rather than with `isMovableByWindowBackground` / `performDrag`: AppKit's drag
/// loop blocks the main queue and, after a quick click, could miss the mouse-up and leave the window following the pointer.
/// Controls drawn above it still get their clicks.
/// A press released without dragging calls `onClick`, except the second press of a double-click.
struct WindowDragArea: NSViewRepresentable {
    var onClick: (() -> Void)?

    final class DragView: NSView {
        var onClick: (() -> Void)?
        private var start: (mouse: NSPoint, origin: NSPoint)?
        private var moved = false

        override func mouseDown(with event: NSEvent) {
            start = window.map { (NSEvent.mouseLocation, $0.frame.origin) }
            moved = false
        }

        override func mouseDragged(with event: NSEvent) {
            guard let window, let start else { return }
            let mouse = NSEvent.mouseLocation
            let dx = mouse.x - start.mouse.x, dy = mouse.y - start.mouse.y
            // A little jitter during a click is not a drag.
            if !moved && hypot(dx, dy) < 3 { return }
            moved = true
            window.setFrameOrigin(NSPoint(x: start.origin.x + dx, y: start.origin.y + dy))
        }

        override func mouseUp(with event: NSEvent) {
            if start != nil && !moved && event.clickCount < 2 { onClick?() }
            start = nil
        }

        override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
    }

    func makeNSView(context: Context) -> DragView { DragView() }
    func updateNSView(_ nsView: DragView, context: Context) { nsView.onClick = onClick }
}
