import Foundation

/// A JSON/TOML value without a fixed schema. Config backup moves the core's settings JSON to and from TOML with it,
/// so fields that `EVSettings` does not model survive an export and an import.
enum ConfigValue: Equatable {
    case string(String)
    case int(Int)
    case double(Double)
    case bool(Bool)
    case array([ConfigValue])
    case object([String: ConfigValue])
}

extension ConfigValue: Codable {
    init(from decoder: Decoder) throws {
        let single = try decoder.singleValueContainer()
        // Bool first: a JSON number never decodes as Bool, and Int before Double keeps whole numbers integral.
        if let value = try? single.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? single.decode(Int.self) {
            self = .int(value)
        } else if let value = try? single.decode(Double.self) {
            self = .double(value)
        } else if let value = try? single.decode(String.self) {
            self = .string(value)
        } else if let value = try? single.decode([ConfigValue].self) {
            self = .array(value)
        } else {
            self = .object(try single.decode([String: ConfigValue].self))
        }
    }

    func encode(to encoder: Encoder) throws {
        var single = encoder.singleValueContainer()
        switch self {
        case .string(let value): try single.encode(value)
        case .int(let value): try single.encode(value)
        case .double(let value): try single.encode(value)
        case .bool(let value): try single.encode(value)
        case .array(let value): try single.encode(value)
        case .object(let value): try single.encode(value)
        }
    }
}
