// swift-tools-version:5.9
import PackageDescription
import Foundation

// Repo root (this file lives in <repo>/macos/). The Rust core is linked from target/release.
let repoRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().path

let package = Package(
    name: "EVKey",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "EVKey", targets: ["EVKey"]),
        .library(name: "EVKeyKit", targets: ["EVKeyKit"]),
    ],
    targets: [
        // C module over crates/core/include/evkey_core.h (symlinked into the target's include dir).
        .target(
            name: "CEVKeyCore",
            path: "Sources/CEVKeyCore"
        ),
        .target(
            name: "EVKeyKit",
            dependencies: ["CEVKeyCore"],
            path: "Sources/EVKeyKit",
            linkerSettings: [
                .unsafeFlags(["-L\(repoRoot)/target/release"]),
                .linkedLibrary("evkey_core"),
                .linkedLibrary("resolv"),
                .linkedFramework("Carbon"),
                .linkedFramework("ApplicationServices"),
                .linkedFramework("AppKit"),
                .linkedFramework("IOKit"),
            ]
        ),
        .executableTarget(
            name: "EVKey",
            dependencies: ["EVKeyKit"],
            path: "Sources/EVKey",
            resources: [.process("Resources")]
        ),
        // XCTest and swift-testing macros are missing under Command Line Tools; this is a plain self-check.
        // Run with: swift run EVKeyKitSelfCheck
        .executableTarget(
            name: "EVKeyKitSelfCheck",
            dependencies: ["EVKeyKit"],
            path: "Tests/EVKeyKitTests"
        ),
    ]
)
