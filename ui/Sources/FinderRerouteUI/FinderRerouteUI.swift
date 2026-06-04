import SwiftUI

@main
struct FinderRerouteUIApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        MenuBarExtra("FinderReroute", systemImage: appState.iconName) {
            ContentView()
                .environmentObject(appState)
        }
        .menuBarExtraStyle(.window)
    }
}

class AppState: ObservableObject {
    @Published var isRunning = false
    @Published var selectedApp: String
    @Published var availableApps: [String] = ["Finder", "Bloom"]
    @Published var statusMessage = "Ready"
    @Published var iconName = "folder"

    private var process: Process?
    private let rustBinaryPath: String
    private let configPath: String

    init() {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        self.configPath = "\(home)/.config/finder-reroute/config.json"
        self.rustBinaryPath = "/Users/alexleekt/git/FinderReroute/target/release/finder-reroute"

        // Load saved selection
        let saved = Self.loadConfig(configPath)
        self.selectedApp = saved?["app"] as? String ?? "Bloom"

        // Check if already running
        self.isRunning = Self.isRustProcessRunning()
        self.updateIcon()
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

    private static func isRustProcessRunning() -> Bool {
        let task = Process()
        task.launchPath = "/usr/bin/pgrep"
        task.arguments = ["-f", "finder-reroute"]
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        try? task.run()
        task.waitUntilExit()
        return task.terminationStatus == 0
    }

    private func updateIcon() {
        iconName = isRunning ? "folder.badge.checkmark" : "folder"
    }

    func toggleRunning() {
        if isRunning {
            stop()
        } else {
            start()
        }
    }

    func start() {
        // Save config
        Self.saveConfig(configPath, ["app": selectedApp])

        // Start the Rust binary
        let task = Process()
        task.launchPath = rustBinaryPath
        task.environment = [
            "RUST_LOG": "info",
            "HOME": FileManager.default.homeDirectoryForCurrentUser.path
        ]
        // Don't capture output — let it run in background
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
        updateIcon()
    }

    func stop() {
        // Kill via pkill
        let task = Process()
        task.launchPath = "/usr/bin/pkill"
        task.arguments = ["-f", rustBinaryPath]
        try? task.run()
        task.waitUntilExit()

        process?.terminate()
        process = nil
        isRunning = false
        statusMessage = "Stopped"
        updateIcon()
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

    private static func discoverFileManagerApps() -> [String] {
        var apps = ["Finder"]
        let workspace = NSWorkspace.shared

        let knownBundleIDs = [
            "com.asiafu.Bloom",      // Bloom
            "com.cocoatech.PathFinder", // Path Finder
            "com.binarynights.ForkLift", // ForkLift
            "com.eltima.CommanderOne", // Commander One
            "com.panic.Transmit",      // Transmit
            "com.qiuyingzhe.Files",   // Files (if exists)
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

    func quit() {
        stop()
        NSApplication.shared.terminate(nil)
    }
}
