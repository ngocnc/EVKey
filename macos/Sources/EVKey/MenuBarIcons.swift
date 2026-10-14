import AppKit

/// Template V/E icon: a rounded outline with the bold letter inside. Drawn, so it stays sharp at any scale.
enum MenuBarIcons {
    static func image(english: Bool) -> NSImage {
        let letter = english ? "E" : "V"
        let image = NSImage(size: NSSize(width: 18, height: 18), flipped: false) { rect in
            let box = rect.insetBy(dx: 1.1, dy: 1.1)
            let outline = NSBezierPath(roundedRect: box, xRadius: 5, yRadius: 5)
            outline.lineWidth = 1.6
            NSColor.black.setStroke()
            outline.stroke()

            let text = NSAttributedString(
                string: letter,
                attributes: [.font: NSFont.systemFont(ofSize: 12.5, weight: .heavy), .foregroundColor: NSColor.black])
            let size = text.size()
            text.draw(at: NSPoint(x: rect.midX - size.width / 2, y: rect.midY - size.height / 2))
            return true
        }
        image.isTemplate = true
        return image
    }
}
