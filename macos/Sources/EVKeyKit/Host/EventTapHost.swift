import os
import ApplicationServices
import Carbon.HIToolbox
import CoreGraphics
import Foundation

/// The keyboard tap: a session-level CGEventTap on its own thread, so a busy main thread never stalls typing.
/// Keys go through the core; the returned edit is injected. Events EVKey injects carry `KeyInjector.marker`
/// and pass through untouched.
final class EventTapHost {
    private let core: CoreSession
    private let secure: SecureInputMonitor
    private let onHotkeyToggle: () -> Void
    private let onTranslate: () -> Void
    private let onActions: () -> Void
    private let onAccessibility: (Bool) -> Void
    private let injector = KeyInjector()
    // Written once on the tap thread before the run loop starts; the watchdog captures its own copy.
    private var tapPort: CFMachPort?
    private var watchdog: DispatchSourceTimer?

    init(
        core: CoreSession,
        secure: SecureInputMonitor,
        onHotkeyToggle: @escaping () -> Void,
        onTranslate: @escaping () -> Void,
        onActions: @escaping () -> Void,
        onAccessibility: @escaping (Bool) -> Void
    ) {
        self.core = core
        self.secure = secure
        self.onHotkeyToggle = onHotkeyToggle
        self.onTranslate = onTranslate
        self.onActions = onActions
        self.onAccessibility = onAccessibility
    }

    /// Starts the tap thread. It waits, polling, until Accessibility is granted, then installs the tap.
    func start() {
        let thread = Thread { [self] in run() }
        thread.name = "EVKey.eventtap"
        thread.start()
    }

    private func run() {
        if !Accessibility.isTrusted { Accessibility.prompt() }
        var reported: Bool?
        func report(_ trusted: Bool) {
            if reported != trusted {
                reported = trusted
                NSLog("EVKey: Accessibility trusted = \(trusted)")
                onAccessibility(trusted)
            }
        }
        while !Accessibility.isTrusted {
            report(false)
            Thread.sleep(forTimeInterval: 1)
        }
        report(true)
        NSLog("EVKey: Accessibility granted, installing the keyboard tap")

        let types: [CGEventType] = [.keyDown, .keyUp, .flagsChanged, .leftMouseDown, .rightMouseDown, .otherMouseDown]
        let mask = types.reduce(CGEventMask(0)) { $0 | (CGEventMask(1) << CGEventMask($1.rawValue)) }
        let refcon = Unmanaged.passUnretained(self).toOpaque()
        // Keyboard taps also need Input Monitoring; ask once, then keep trying until the system lets us create the tap.
        if !CGPreflightListenEventAccess() { _ = CGRequestListenEventAccess() }
        var created: CFMachPort?
        while created == nil {
            created = CGEvent.tapCreate(
                tap: .cgSessionEventTap,
                place: .headInsertEventTap,
                options: .defaultTap,
                eventsOfInterest: mask,
                callback: eventTapCallback,
                userInfo: refcon)
            if created == nil {
                NSLog("EVKey: keyboard tap not created yet (Input Monitoring?); retrying")
                Thread.sleep(forTimeInterval: 2)
            }
        }
        guard let tap = created else { return }
        NSLog("EVKey: keyboard tap installed")
        tapPort = tap
        CFRunLoopAddSource(CFRunLoopGetCurrent(), CFMachPortCreateRunLoopSource(nil, tap, 0), .commonModes)
        CGEvent.tapEnable(tap: tap, enable: true)
        startWatchdog(tap)
        CFRunLoopRun()
    }

    /// The system disables a tap that is slow or after user input; the callback re-enables it, and this covers the rest.
    private func startWatchdog(_ tap: CFMachPort) {
        let timer = DispatchSource.makeTimerSource(queue: DispatchQueue.global(qos: .utility))
        timer.schedule(deadline: .now() + 5, repeating: 5)
        timer.setEventHandler { [weak self] in
            if !CGEvent.tapIsEnabled(tap: tap) { CGEvent.tapEnable(tap: tap, enable: true) }
            self?.onAccessibility(Accessibility.isTrusted)
        }
        timer.resume()
        watchdog = timer
    }

    fileprivate func handle(type: CGEventType, event: CGEvent) -> Unmanaged<CGEvent>? {
        switch type {
        case .tapDisabledByTimeout, .tapDisabledByUserInput:
            if let tapPort { CGEvent.tapEnable(tap: tapPort, enable: true) }
            return Unmanaged.passUnretained(event)
        case .leftMouseDown, .rightMouseDown, .otherMouseDown:
            // A click can move the caret, so the word being typed is forgotten.
            core.mouseClick()
            return Unmanaged.passUnretained(event)
        case .keyDown, .keyUp, .flagsChanged:
            if event.getIntegerValueField(.eventSourceUserData) == KeyInjector.marker || HotkeyRecording.isActive {
                return Unmanaged.passUnretained(event)
            }
            return keyEvent(type: type, event: event)
        default:
            return Unmanaged.passUnretained(event)
        }
    }

    private func keyEvent(type: CGEventType, event: CGEvent) -> Unmanaged<CGEvent>? {
        let keycode = UInt16(truncatingIfNeeded: event.getIntegerValueField(.keyboardEventKeycode))
        let flags = event.flags
        let mods = KeyMap.modBits(flags)
        let capsLock = flags.contains(.maskAlphaShift)
        let vk: UInt16
        let down: Bool
        if type == .flagsChanged {
            guard let modifier = KeyMap.modifier(forKeycode: keycode) else {
                return Unmanaged.passUnretained(event)
            }
            vk = modifier.vk
            down = flags.rawValue & modifier.deviceMask != 0
        } else {
            down = type == .keyDown
            guard let mapped = KeyMap.vk(forKeycode: keycode) else {
                // Unmodeled keys end the word, as in the reference hook: vk 0 is a word break for the engine.
                if down { _ = core.onKey(vk: 0, down: true, mods: mods, capsLock: capsLock) }
                return Unmanaged.passUnretained(event)
            }
            vk = mapped
        }
        if down { secure.checkOnKeyDown() }

        let reaction = core.onKey(vk: vk, down: down, mods: mods, capsLock: capsLock)
        if reaction.hotkeyToggled { onHotkeyToggle() }
        if reaction.translateRequested {
            // Secure Input: the hotkey does nothing and the key goes through untouched.
            if IsSecureEventInputEnabled() { return Unmanaged.passUnretained(event) }
            onTranslate()
        }
        if reaction.actionsRequested {
            if IsSecureEventInputEnabled() { return Unmanaged.passUnretained(event) }
            onActions()
        }
        if let edit = reaction.edit { injector.apply(edit) }
        return reaction.swallow || reaction.edit != nil ? nil : Unmanaged.passUnretained(event)
    }
}

/// C callback: must not capture. `refcon` is the `EventTapHost`, which lives as long as the app.
private func eventTapCallback(
    proxy: CGEventTapProxy,
    type: CGEventType,
    event: CGEvent,
    refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {
    guard let refcon else { return Unmanaged.passUnretained(event) }
    return Unmanaged<EventTapHost>.fromOpaque(refcon).takeUnretainedValue().handle(type: type, event: event)
}

/// Ô ghi phím tắt trong Bảng điều khiển đang ghi: event tap cho mọi phím đi thẳng, để tổ hợp đã gán (ví dụ phím dịch)
/// không bị nuốt mà tới được ô ghi, và gõ tiếng Việt không chen vào.
public enum HotkeyRecording {
    private static let flag = OSAllocatedUnfairLock(initialState: false)
    public static var isActive: Bool { flag.withLock { $0 } }
    public static func setActive(_ on: Bool) { flag.withLock { $0 = on } }
}
