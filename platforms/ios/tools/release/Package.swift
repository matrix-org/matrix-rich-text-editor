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
        .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.5.0"),
        .package(url: "https://github.com/element-hq/swift-command-line-tools.git", revision: "483396af716a59eb45379126389a063f7f9bee80")
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
