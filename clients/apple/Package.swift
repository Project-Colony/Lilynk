// swift-tools-version: 5.9
// LilynkKit — Swift wrapper around the lilynk-core C ABI (lilynk/1 client connector) plus the
// SwiftUI/VideoToolbox presentation layer. Build LilynkCore.xcframework first:
//   bash ../../scripts/build-xcframework.sh   (on a Mac; see README.md)
import PackageDescription

let package = Package(
    name: "LilynkKit",
    defaultLocalization: "en",
    platforms: [.macOS(.v14), .iOS(.v17), .tvOS(.v17), .visionOS("26.0")],
    products: [
        .library(name: "LilynkKit", targets: ["LilynkKit"]),
        // Dependency-free foundation (stored-host model + JSON codec, settings keys, App-Group
        // constant, deep-link grammar, Live Activity attributes). A separate PRODUCT so the widget
        // extension — which must never link LilynkKit (Rust staticlib + presentation layer) —
        // can link this and nothing else. LilynkKit re-exports it (see SharedReexport.swift).
        .library(name: "LilynkShared", targets: ["LilynkShared"]),
        .executable(name: "LilynkClient", targets: ["LilynkClient"]),
    ],
    dependencies: [
        // Progressive backdrop blur with no material tint; `ly-console-ui`'s `blur.rs` follows it.
        // GlurBackdrop first ships in 2.0.
        .package(url: "https://github.com/joogps/Glur.git", from: "2.0.0"),
    ],
    targets: [
        .binaryTarget(name: "LilynkCore", path: "LilynkCore.xcframework"),
        // No dependencies by design — an extension process links this alone. `Generated/` is
        // the String Catalog and `L10n` from l10n/messages: scripts/gen-l10n-apple.sh writes it.
        // The two other catalogs there belong to the apps, which compile them.
        .target(
            name: "LilynkShared",
            exclude: ["Generated/AppShortcuts.xcstrings", "Generated/InfoPlist.xcstrings"],
            resources: [.process("Generated/Localizable.xcstrings")]),
        .target(
            name: "LilynkKit",
            dependencies: ["LilynkCore", "LilynkShared"],
            // OSS attribution shown by the app's Acknowledgements screen. Bundled here (not in the
            // app target) so it rides along via Bundle.module in both `swift build` and the Xcode
            // app, which links the LilynkKit product. Refresh with
            // scripts/gen-third-party-notices.sh (it copies the generated file into Resources/).
            resources: [
                .copy("Resources/THIRD-PARTY-NOTICES.txt"),
                .copy("Resources/LICENSE-GPL.txt"),
                .copy("Resources/LICENSE-MIT.txt"),
                .copy("Resources/LICENSE-APACHE.txt"),
                // Geist (SIL OFL 1.1) — the brand typeface, shared with punktfunk-website.
                // Registered with Core Text at first use; see BrandFont.swift.
                .copy("Resources/Fonts"),
                // The host cards' OS marks (template vector imagesets generated from the
                // assets/os-icons masters by scripts/gen-os-icons.sh — per-mark provenance and
                // licensing in that README). `.process` compiles the catalog; loaded via
                // OsIcon.swift.
                .process("Resources/OsIcons.xcassets"),
                // The launcher tiles' brand marks (template vector imagesets generated from the
                // assets/launcher-icons masters by scripts/gen-launcher-icons.sh — per-mark
                // provenance and licensing in that README). Loaded via LauncherIcon.swift.
                .process("Resources/LauncherIcons.xcassets"),
                // The Controller type quick action's pad outlines (scripts/gen-pad-marks.py, from
                // the console's Kenney outlines). Loaded via PadType.swift.
                .process("Resources/PadMarks.xcassets"),
            ],
            linkerSettings: [
                // Rust staticlib system deps.
                .linkedFramework("Security"),
                .linkedFramework("SystemConfiguration"),
                .linkedLibrary("resolv"),
                // Skia, for the console (clients/apple/native).
                .linkedLibrary("c++"),
                .linkedFramework("Metal"),
                .linkedFramework("CoreText"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("ImageIO"),
                .linkedFramework("ApplicationServices", .when(platforms: [.macOS])),
                .linkedFramework("UIKit", .when(platforms: [.iOS, .tvOS, .visionOS])),
                .linkedFramework("MobileCoreServices", .when(platforms: [.iOS, .tvOS, .visionOS])),
            ]
        ),
        // Development app shell (swift run LilynkClient): connect form → stream + input.
        // (The tvOS slide-transition package is referenced by the Xcode PROJECT only —
        // its manifest breaks SwiftPM whole-graph validation on macOS, and only the
        // Lilynk-tvOS target links it; the #if os(tvOS) import never compiles here.)
        .executableTarget(
            name: "LilynkClient",
            dependencies: [
                "LilynkKit",
                .product(name: "GlurBackdrop", package: "Glur"),
            ]),
        // LilynkCore is a direct dep too so the wire tests can name the C ABI's
        // `LilynkInputEvent` / `LILYNK_INPUT_KIND_*` when asserting the gamepad byte layout.
        // Pins the tests to English; see TestLanguage.m.
        .target(name: "LilynkTestLanguage", path: "Tests/TestLanguage"),
        .testTarget(
            name: "LilynkKitTests",
            dependencies: ["LilynkKit", "LilynkShared", "LilynkCore", "LilynkTestLanguage"],
            resources: [
                // PyroWave golden fixtures: host-encoded AUs + upstream-decoded reference
                // planes (regenerate with lilynk-host's `pyrowave_dump_golden` on a
                // Vulkan box — see PyroWaveDecoderTests.swift).
                .copy("PyroWaveFixtures")
            ]),
    ]
)
