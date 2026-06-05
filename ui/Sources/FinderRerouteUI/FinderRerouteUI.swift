import SwiftUI

@main
struct FinderRerouteUIApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        MenuBarExtra {
            ContentView()
                .environmentObject(appState)
        } label: {
            Image(systemName: "folder")
                .accessibilityLabel("FinderReroute")
        }
        .menuBarExtraStyle(.window)
    }
}

class AppState: ObservableObject {
    @Published var isRunning = false
    @Published var selectedApp: String
    @Published var availableApps: [String] = ["Finder", "Bloom"]
    @Published var statusMessage = "Ready"
    private var process: Process?
    private let rustBinaryPath: String
    private let configPath: String

    init() {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        self.configPath = "\(home)/.config/finder-reroute/config.json"

        // Find the bundled Rust binary relative to the SwiftUI app executable
        let bundleDir = Bundle.main.executableURL?.deletingLastPathComponent()
        guard let executablePath = bundleDir?.appendingPathComponent("finder-reroute").path else {
            fatalError("finder-reroute binary not found in app bundle.")
        }
        self.rustBinaryPath = executablePath

        let saved = Self.loadConfig(configPath)
        self.selectedApp = saved?["app"] as? String ?? "Bloom"

        self.isRunning = false
    }

    private static func loadConfig(_ path: String) -> [String: Any]? {
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return nil
        }
        return json
    }

    private static func saveConfig(_ path: String, _ dict: [String: Any]) {
        let dir = (path as NSString).deletingLastPathComponent
        try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
        if let data = try? JSONSerialization.data(withJSONObject: dict, options: [.prettyPrinted]) {
            try? data.write(to: URL(fileURLWithPath: path))
        }
    }

    func toggleRunning() {
        if isRunning {
            stop()
        } else {
            start()
        }
    }

    func start() {
        Self.saveConfig(configPath, ["app": selectedApp])

        let task = Process()
        task.launchPath = rustBinaryPath
        task.environment = [
            "RUST_LOG": "info",
            "HOME": FileManager.default.homeDirectoryForCurrentUser.path
        ]
        let null = FileHandle.nullDevice
        task.standardOutput = null
        task.standardError = null

        do {
            try task.run()
            process = task
            isRunning = true
            statusMessage = "Running — intercepting Finder"
        } catch {
            statusMessage = "Failed to start: \(error.localizedDescription)"
        }
    }

    func stop() {
        process?.terminate()
        process?.waitUntilExit()
        process = nil
        isRunning = false
        statusMessage = "Stopped"
    }

    func selectApp(_ app: String) {
        selectedApp = app
        Self.saveConfig(configPath, ["app": app])
        if isRunning {
            statusMessage = "Restart to apply \(app)"
        } else {
            statusMessage = "Selected: \(app)"
        }
    }

    func scanApps() {
        statusMessage = "Scanning..."
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            let found = Self.discoverFileManagerApps()
            DispatchQueue.main.async {
                self?.availableApps = found
                self?.statusMessage = "Found \(found.count) apps"
            }
        }
    }

    func quit() {
        stop()
        NSApplication.shared.terminate(nil)
    }

    private static func discoverFileManagerApps() -> [String] {
        var apps = ["Finder"]
        let workspace = NSWorkspace.shared

        let knownBundleIDs = [
            "com.asiafu.Bloom",      // Bloom
            "com.cocoatech.PathFinder", // Path Finder
            "com.binarynights.ForkLift", // ForkLift
            "com.eltima.CommanderOne", // Commander One
            "com.panic.Transmit",      // Transmit
            "com.qiuyingzhe.Files"   // Files (if exists)
        ]

        for bundleID in knownBundleIDs {
            if let url = workspace.urlForApplication(withBundleIdentifier: bundleID) {
                let name = url.deletingPathExtension().lastPathComponent
                if !apps.contains(name) {
                    apps.append(name)
                }
            }
        }

        return apps
    }
}
