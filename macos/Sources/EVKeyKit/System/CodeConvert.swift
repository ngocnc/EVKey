import CEVKeyCore
import Foundation

/// The action panel's local text tools, done by the Rust core (`convert::transform` in `crates/engine/src/convert.rs`).
public enum CodeConvert {
    /// Applies tool `id` (`convert`, `convert_tcvn3`, `convert_vni`, `strip_marks`, `case_upper`, `case_lower`,
    /// `case_title`) to `text`. `convert` guesses the source code table and returns precomposed Unicode.
    /// Nil when the core rejects the input or the id.
    public static func transform(_ text: String, id: String) -> String? {
        guard let converted = text.withCString({ t in id.withCString { evkey_text_transform(t, $0) } }) else { return nil }
        defer { evkey_string_free(converted) }
        return String(cString: converted)
    }
}
