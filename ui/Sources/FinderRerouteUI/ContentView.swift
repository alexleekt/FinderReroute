import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header
            HStack {
                Image(systemName: appState.iconName)
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
            .labelsHidden()

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

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
            .environmentObject(AppState())
    }
}
