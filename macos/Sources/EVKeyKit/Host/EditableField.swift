import AppKit
import ApplicationServices

/// Whether keyboard focus is in a place where text can be typed. The action panel rewrites the selection, so it opens
/// only there (the Windows build does the same in `secure_input_win::editable_focused`).
public enum EditableField {
    private static let textRoles: Set<String> = ["AXTextField", "AXTextArea", "AXComboBox"]
    private static let secureSubrole = "AXSecureTextField"

    /// What the focused element reports through Accessibility.
    struct Facts {
        var role: String?
        var subrole: String?
        /// `AXValue` can be written.
        var valueSettable = false
        /// Web content flags: `AXEditable`, and whether the element sits inside an editable ancestor.
        var editableFlag: Bool?
        var hasEditableAncestor = false
    }

    /// Password fields never count. A text field, text area or combo box counts unless it says it is not editable. Anything else
    /// (web contenteditable, custom editors) counts when its value is writable or it is flagged editable.
    static func isEditable(_ facts: Facts) -> Bool {
        if facts.subrole == secureSubrole { return false }
        if let role = facts.role, textRoles.contains(role) { return facts.editableFlag != false }
        return facts.valueSettable || facts.editableFlag == true || facts.hasEditableAncestor
    }

    /// Reads the focused element of the frontmost app. False when Accessibility is not granted; nil when the app reports
    /// no focused element, as Chromium apps that keep their web content out of Accessibility do (Lark).
    public static func isFocused() -> Bool? {
        guard AXIsProcessTrusted() else { return false }
        guard let element = focusedElement() else { return nil }
        // A hung app must not freeze the hotkey.
        AXUIElementSetMessagingTimeout(element, 0.5)
        var facts = Facts()
        facts.role = string(element, kAXRoleAttribute)
        facts.subrole = string(element, kAXSubroleAttribute)
        var settable: DarwinBoolean = false
        if AXUIElementIsAttributeSettable(element, kAXValueAttribute as CFString, &settable) == .success {
            facts.valueSettable = settable.boolValue
        }
        facts.editableFlag = bool(element, "AXEditable")
        facts.hasEditableAncestor = value(element, "AXEditableAncestor") != nil
            || value(element, "AXHighestEditableAncestor") != nil
        return isEditable(facts)
    }

    /// The system-wide focus first; some apps only answer when asked through their own application element.
    private static func focusedElement() -> AXUIElement? {
        let systemWide = AXUIElementCreateSystemWide()
        AXUIElementSetMessagingTimeout(systemWide, 0.5)
        if let element = element(systemWide, kAXFocusedUIElementAttribute) { return element }
        guard let app = NSWorkspace.shared.frontmostApplication else { return nil }
        let application = AXUIElementCreateApplication(app.processIdentifier)
        AXUIElementSetMessagingTimeout(application, 0.5)
        return element(application, kAXFocusedUIElementAttribute)
    }

    private static func element(_ parent: AXUIElement, _ attribute: String) -> AXUIElement? {
        guard let raw = value(parent, attribute), CFGetTypeID(raw) == AXUIElementGetTypeID() else { return nil }
        return (raw as! AXUIElement)
    }

    private static func value(_ element: AXUIElement, _ attribute: String) -> CFTypeRef? {
        var out: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, attribute as CFString, &out) == .success else { return nil }
        return out
    }

    private static func string(_ element: AXUIElement, _ attribute: String) -> String? {
        value(element, attribute) as? String
    }

    private static func bool(_ element: AXUIElement, _ attribute: String) -> Bool? {
        (value(element, attribute) as? NSNumber)?.boolValue
    }
}
