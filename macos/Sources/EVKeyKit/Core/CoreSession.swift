import CEVKeyCore
import Foundation

/// What the host does with one key event: swallow it, and type `edit` instead.
public struct KeyReaction: Equatable {
    public var swallow = false
    public var hotkeyToggled = false
    public var edit: TextEdit?
}

/// Typed in place of the swallowed key: optional EMPTY_CHAR, `backspaces` backspaces, `text`, then `resend`.
public struct TextEdit: Equatable {
    public var prefixEmptyChar: Bool
    public var backspaces: Int
    public var text: [UInt16]
    public var resend: Resend?
}

/// The trigger key typed again: a UTF-16 unit, or a Windows virtual-key code when there is no character.
public enum Resend: Equatable {
    case char(UInt16)
    case vk(UInt16)
}

/// Owns one core `EvSession`. The core is not thread-safe, so every call takes `lock`.
/// Keys arrive on the tap thread; everything else comes from the main thread.
public final class CoreSession {
    private let handle: OpaquePointer
    private let lock = NSLock()

    /// `directory` nil = the default settings directory.
    init?(directory: String?) {
        let session: OpaquePointer?
        if let directory {
            session = directory.withCString { evkey_session_new($0) }
        } else {
            session = evkey_session_new(nil)
        }
        guard let session else { return nil }
        handle = session
    }

    deinit { evkey_session_free(handle) }

    /// `vk` is a Windows virtual-key code; `mods` bits: 1 ctrl, 2 alt, 4 shift, 8 win (Command).
    func onKey(vk: UInt16, down: Bool, mods: UInt8, capsLock: Bool) -> KeyReaction {
        locked {
            var out = EvReaction()
            guard evkey_session_on_key(handle, vk, down, mods, capsLock, &out) else { return KeyReaction() }
            var reaction = KeyReaction(swallow: out.swallow, hotkeyToggled: out.hotkey_toggled)
            if out.has_edit {
                // The text buffer belongs to the session and is valid only until the next call: copy it now.
                var text: [UInt16] = []
                if let ptr = out.text_ptr, out.text_len > 0 {
                    text = Array(UnsafeBufferPointer(start: ptr, count: out.text_len))
                }
                reaction.edit = TextEdit(
                    prefixEmptyChar: out.prefix_empty_char,
                    backspaces: Int(out.backspaces),
                    text: text,
                    resend: Self.resend(out))
            }
            return reaction
        }
    }

    func mouseClick() {
        locked { evkey_session_mouse_click(handle) }
    }

    /// `bundleID` nil when the frontmost app is unknown.
    func setForeground(_ bundleID: String?) {
        locked {
            guard let bundleID else {
                evkey_session_set_foreground(handle, nil)
                return
            }
            bundleID.withCString { evkey_session_set_foreground(handle, $0) }
        }
    }

    func setSecure(_ on: Bool) {
        locked { evkey_session_set_secure(handle, on) }
    }

    var isEnglish: Bool {
        locked { evkey_session_is_english(handle) }
    }

    func setEnglish(_ on: Bool) {
        locked { evkey_session_set_english(handle, on) }
    }

    func toggle() {
        locked { evkey_session_toggle(handle) }
    }

    private func locked<T>(_ body: () -> T) -> T {
        lock.lock()
        defer { lock.unlock() }
        return body()
    }

    private static func resend(_ out: EvReaction) -> Resend? {
        switch Int(out.resend_kind) {
        case Int(EV_RESEND_CHAR): return .char(out.resend_value)
        case Int(EV_RESEND_VK): return .vk(out.resend_value)
        default: return nil
        }
    }
}
