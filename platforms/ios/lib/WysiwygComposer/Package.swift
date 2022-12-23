// swift-tools-version: 5.6
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
    ],
    targets: [
        .binaryTarget(
            name: "WysiwygComposerFFI",
            path: "WysiwygComposerFFI.xcframework"
        ),
        .target(
            name: "WysiwygComposer",
            dependencies: [
                .target(name: "WysiwygComposerFFI"),
            ]
        ),
        .testTarget(
            name: "WysiwygComposerTests",
            dependencies: [
                "WysiwygComposer",
                .product(name: "SnapshotTesting", package: "swift-snapshot-testing"),
            ]
        ),
    ]
)
