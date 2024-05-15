// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Release",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "release", targets: ["Release"])
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.2.0"),
        .package(url: "https://github.com/element-hq/swift-command-line-tools.git", revision: "a6ad90808f4f6cac615ab8496c6ff1bc5f9fa192")
        // .package(path: "../../../../../swift-command-line-tools")
    ],
    targets: [
        .executableTarget(name: "Release",
                          dependencies: [
                            .product(name: "ArgumentParser", package: "swift-argument-parser"),
                            .product(name: "CommandLineTools", package: "swift-command-line-tools")
                          ]),
    ]
)
