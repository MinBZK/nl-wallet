// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "bluetooth",
    platforms: [
        .iOS("13.0"),
    ],
    products: [
        .library(name: "bluetooth", targets: ["bluetooth"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
        
    ],
    targets: [
        .target(
            name: "bluetooth",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ],
            resources: [
                .process("PrivacyInfo.xcprivacy"),
            ]
        )
    ]
)

