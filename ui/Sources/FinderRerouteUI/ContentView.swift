import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header
            HStack {
                Image(systemName: "folder")
                    .foregroundColor(appState.isRunning ? .green : .secondary)
                Text("FinderReroute")
                    .font(.headline)
                Spacer()
            }

            Divider()

            // App selection
            Picker("Open folders with:", selection: Binding(
                get: { appState.selectedApp },
                set: { appState.selectApp($0) }
            )) {
                ForEach(appState.availableApps, id: \.self) { app in
                    Text(app).tag(app)
                }
            }
            .pickerStyle(.menu)
            .accessibilityLabel("Open folders with")

            Button("Scan for Apps") {
                appState.scanApps()
            }
            .font(.caption)
            .buttonStyle(.plain)
            .foregroundColor(.accentColor)

            Divider()

            // Toggle
            Toggle("Intercept Finder clicks", isOn: Binding(
                get: { appState.isRunning },
                set: { _ in appState.toggleRunning() }
            ))
            .toggleStyle(.switch)
            .accessibilityHint("When on, clicking the Finder icon in the Dock opens your chosen file manager instead.")

            // Status
            Text(appState.statusMessage)
                .font(.caption)
                .foregroundColor(.secondary)
                .lineLimit(1)

            Divider()

            // Quit
            Button("Quit") {
                appState.quit()
            }
            .buttonStyle(.plain)
            .foregroundColor(.secondary)
        }
        .padding()
        .frame(width: 220)
    }
}
