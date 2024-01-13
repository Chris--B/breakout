// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "RooibosPlatformCode_Apple",
    platforms: [
        .macOS(.v13),
    ],
    products: [
        .library(name: "RooibosPlatform", type: .static, targets: ["RooibosPlatform"]),
    ],
    targets: [
        .target(name: "RooibosPlatform", path: "."),
    ]
)
