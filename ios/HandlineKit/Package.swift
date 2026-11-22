// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "HandlineKit",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "HandlineKit", targets: ["HandlineKit"])
    ],
    targets: [
        .target(name: "HandlineKit"),
        .executableTarget(name: "HandlineCheck", dependencies: ["HandlineKit"])
    ]
)
