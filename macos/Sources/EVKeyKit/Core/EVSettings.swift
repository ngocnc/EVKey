import Foundation

// Mirrors crates/engine/src/settings.rs. Raw values are the serde spellings; keep both in sync.

public enum InputKind: String, Codable, Equatable, CaseIterable {
    case telex
    case vni
    case simpleTelex1 = "simple_telex1"
    case simpleTelex2 = "simple_telex2"
}

public enum TonePlacement: String, Codable, Equatable, CaseIterable {
    case new
    case old
}

public enum CodeTable: String, Codable, Equatable, CaseIterable {
    case unicode
    case tcvn3
    case vniWindows = "vni_windows"
    case unicodeCompound = "unicode_compound"
}

public enum TempOffKey: String, Codable, Equatable, CaseIterable {
    case none
    case ctrl
    case alt
}

public enum Backend: String, Codable, Equatable, CaseIterable {
    case hook
    case tsf
}

/// Display language of every EVKey screen; not the V/E Chế độ.
public enum UILanguage: String, Codable, Equatable, CaseIterable {
    case vi
    case en
}

public struct EVSettings: Codable, Equatable {
    public var inputType: InputKind
    public var tonePlacement: TonePlacement
    public var codeTable: CodeTable
    public var freeMark: Bool
    public var checkSpelling: Bool
    public var restoreIfWrongSpelling: Bool
    public var quickTelex: Bool
    public var quickStartConsonant: Bool
    public var quickEndConsonant: Bool
    public var upperCaseFirstChar: Bool
    /// Which places capitalize the next letter when `upperCaseFirstChar` is on.
    public var upperCaseAfterPunct: Bool
    public var upperCaseAfterEnter: Bool
    public var upperCaseFirstLetter: Bool
    public var allowConsonantZfwj: Bool
    public var autoCapsMacro: Bool
    /// Esc while a word is composing restores the typed keys and stops conversion for that word.
    public var undoWord: Bool
    /// Each app keeps its own Chế độ V/E; the core stores the records in `app_modes.toml`.
    public var rememberAppMode: Bool
    /// Plays a short sound on user-initiated V/E switches.
    public var toggleSound: Bool
    /// Letters a-z accepted as syllable starts while `allowConsonantZfwj` is on (engine `custom_consonants`).
    public var customConsonants: [String]
    /// Windows executable names. Not edited on macOS, but kept so a save does not drop them.
    public var excludedApps: [String]
    /// macOS bundle identifiers where EVKey never converts keys.
    public var excludedAppsMacos: [String]
    public var tempOffKey: TempOffKey
    public var toggleHotkey: String
    public var backend: Backend
    public var bannerOpacity: Int
    public var bannerBlur: Int
    public var bannerBg: String
    public var bannerBorder: String
    public var bannerText: String
    /// Gradient end colours (`#rrggbb`); empty keeps the matching colour solid.
    public var bannerBgEnd: String
    public var bannerBorderEnd: String
    public var bannerTextEnd: String
    public var uiLanguage: UILanguage
    public var autoCheckUpdates: Bool
    /// Version the user chose to skip; its update popup is not shown again.
    public var skippedUpdate: String
    /// Key chord (`ctrl+alt+t`; must end in a key, `none` disables) that translates the selected text.
    public var translateHotkey: String
    /// What the translate flow does after the popup shows the result.
    public var translateAction: TranslateAction
    /// Milliseconds the translate popup stays before closing; 0 keeps it until clicked.
    public var translatePopupMs: Int
    /// Service that translates (`translate_provider`); Gemini uses the key saved in the Keychain.
    public var translateProvider: TranslateProvider
    /// Key chord (`ctrl+alt+a`; `none` disables) that opens the action panel on the selected text.
    public var actionHotkey: String

    /// Engine defaults, read from the core (the hotkey default depends on the platform).
    public static let defaults: EVSettings = SettingsStore.engineDefaults()

    enum CodingKeys: String, CodingKey {
        case inputType = "input_type"
        case tonePlacement = "tone_placement"
        case codeTable = "code_table"
        case freeMark = "free_mark"
        case checkSpelling = "check_spelling"
        case restoreIfWrongSpelling = "restore_if_wrong_spelling"
        case quickTelex = "quick_telex"
        case quickStartConsonant = "quick_start_consonant"
        case quickEndConsonant = "quick_end_consonant"
        case upperCaseFirstChar = "upper_case_first_char"
        case upperCaseAfterPunct = "upper_case_after_punct"
        case upperCaseAfterEnter = "upper_case_after_enter"
        case upperCaseFirstLetter = "upper_case_first_letter"
        case allowConsonantZfwj = "allow_consonant_zfwj"
        case autoCapsMacro = "auto_caps_macro"
        case undoWord = "undo_word"
        case rememberAppMode = "remember_app_mode"
        case toggleSound = "toggle_sound"
        case customConsonants = "custom_consonants"
        case excludedApps = "excluded_apps"
        case excludedAppsMacos = "excluded_apps_macos"
        case tempOffKey = "temp_off_key"
        case toggleHotkey = "toggle_hotkey"
        case backend
        case bannerOpacity = "banner_opacity"
        case bannerBlur = "banner_blur"
        case bannerBg = "banner_bg"
        case bannerBorder = "banner_border"
        case bannerText = "banner_text"
        case bannerBgEnd = "banner_bg_end"
        case bannerBorderEnd = "banner_border_end"
        case bannerTextEnd = "banner_text_end"
        case uiLanguage = "ui_language"
        case autoCheckUpdates = "auto_check_updates"
        case skippedUpdate = "skipped_update"
        case translateHotkey = "translate_hotkey"
        case translateAction = "translate_action"
        case translatePopupMs = "translate_popup_ms"
        case translateProvider = "translate_provider"
        case actionHotkey = "action_hotkey"
    }
}

/// What the translate flow does after the popup shows the result (`translate_action`).
public enum TranslateAction: String, Codable, CaseIterable {
    case show, copy, replace
}

/// Service that translates the selection (`translate_provider`).
public enum TranslateProvider: String, Codable, CaseIterable {
    case google, gemini
}
