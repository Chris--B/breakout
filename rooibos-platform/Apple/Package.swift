// swift-tools-version:5.1
import PackageDescription

let package = Package(
    name: "RooibosPlatformCode_Apple",
    platforms: [
        .macOS(.v10_15),
    ],
    products: [
        .library(name: "RooibosPlatform", type: .static, targets: ["RooibosPlatform"]),
    ],
    targets: [
        .target(name: "RooibosPlatform", path: "."),
    ]
)
