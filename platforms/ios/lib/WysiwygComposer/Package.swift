// swift-tools-version: 5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "WysiwygComposer",
    platforms: [
        .iOS(.v14),
    ],
    products: [
        .library(
            name: "WysiwygComposer",
            targets: ["WysiwygComposer"]
        ),
    ],
    dependencies: [
        .package(
            url: "https://github.com/pointfreeco/swift-snapshot-testing",
            from: "1.10.0"
        ),
        .package(
            url: "https://github.com/Cocoanetics/DTCoreText",
            from: "1.6.27"
        ),
    ],
    targets: [
        .target(
            name: "DTCoreTextExtended",
            dependencies: [
                .product(name: "DTCoreText", package: "DTCoreText"),
            ]
        ),
        .target(
            name: "HTMLParser",
            dependencies: [
                .product(name: "DTCoreText", package: "DTCoreText"),
                .target(name: "DTCoreTextExtended"),
            ]
        ),
        .binaryTarget(
            name: "WysiwygComposerFFI",
            path: "WysiwygComposerFFI.xcframework"
        ),
        .target(
            name: "WysiwygComposer",
            dependencies: [
                .target(name: "WysiwygComposerFFI"),
                .target(name: "HTMLParser"),
            ]
        ),
        .testTarget(
            name: "HTMLParserTests",
            dependencies: [
                "HTMLParser",
            ]
        ),
        .testTarget(
            name: "WysiwygComposerTests",
            dependencies: [
                "WysiwygComposer",
            ]
        ),
        .testTarget(
            name: "WysiwygComposerSnapshotTests",
            dependencies: [
                "WysiwygComposer",
                .product(name: "SnapshotTesting", package: "swift-snapshot-testing"),
            ]
        ),
    ]
)
