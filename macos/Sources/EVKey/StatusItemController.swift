import AppKit
import Combine
import EVKeyKit

/// Menu bar item: the V/E template icon follows the mode; a click opens the popup under it.
@MainActor
final class StatusItemController {
    private let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    private var modeCancellable: AnyCancellable?

    init() {
        guard let button = item.button else { return }
        button.target = self
        button.action = #selector(togglePopup)
        button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        modeCancellable = AppState.shared.$isEnglish.sink { [weak self] english in
            guard let button = self?.item.button else { return }
            // Cross-fade the V/E icon.
            let fade = CATransition()
            fade.type = .fade
            fade.duration = 0.18
            button.wantsLayer = true
            button.layer?.add(fade, forKey: "mode")
            button.image = MenuBarIcons.image(english: english)
        }
    }

    @objc private func togglePopup() {
        PopupController.shared.toggle(below: item.button)
    }
}
