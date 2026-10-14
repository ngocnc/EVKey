import AppKit

/// Borderless panel that can still become key, so the popup's shortcuts and Escape work.
final class PopupPanel: NSPanel {
    override var canBecomeKey: Bool { true }
}
