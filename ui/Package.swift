// swift-tools-version:5.7
import PackageDescription

let package = Package(
    name: "FinderRerouteUI",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(
            name: "FinderRerouteUI",
            targets: ["FinderRerouteUI"]
        )
    ],
    targets: [
        .executableTarget(
            name: "FinderRerouteUI",
            swiftSettings: [
                .unsafeFlags(["-parse-as-library"])
            ]
        )
    ]
)
