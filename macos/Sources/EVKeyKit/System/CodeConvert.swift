import CEVKeyCore
import Foundation

/// Legacy Vietnamese code tables to Unicode, done by the Rust core (`crates/engine/src/convert.rs`).
public enum CodeConvert {
    /// Guesses the code table of `text` (TCVN3, VNI-Windows, Unicode compound) and returns precomposed Unicode.
    /// Plain ASCII and text that is already Unicode come back unchanged. Nil when the core rejects the input.
    public static func toUnicode(_ text: String) -> String? {
        guard let converted = text.withCString({ evkey_convert_to_unicode($0) }) else { return nil }
        defer { evkey_string_free(converted) }
        return String(cString: converted)
    }
}
