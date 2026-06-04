# FinderReroute

A Rust core library that intercepts clicks on the **Finder** icon in the macOS Dock and launches an alternative file manager (e.g., **Bloom**) instead.

## How It Works

```
User clicks Finder icon in Dock
        ↓
CGEventTap (HID level) intercepts leftMouseDown
        ↓
AXUIElement hit-test identifies the clicked Dock item
        ↓
If title == "Finder" and subrole == "AXApplicationDockItem"
        ↓
Consume the event (Dock never sees it)
        ↓
Launch target app via `open -a`
```

## Tech Stack

| Layer | Crate | Purpose |
|-------|-------|---------|
| Event interception | `core-graphics` 0.25 | `CGEventTap` for HID-level mouse event capture |
| Run loop | `core-foundation` 0.10 | `CFRunLoop` + `CFMachPort` for tap lifecycle |
| Dock detection | `axuielement` 0.9 | `AXUIElementCopyElementAtPosition` + attribute reading |
| App launching | `std::process::Command` | `open -a <app>` for launching macOS apps |
| Logging | `log` + `env_logger` | Structured logging with `RUST_LOG` env var |

## Required macOS Permissions

- **Accessibility** — Required for `AXUIElementCopyElementAtPosition`
- **Input Monitoring** — Required for `CGEventTap` (HID-level tap)

The binary will exit on startup if Accessibility is not granted, prompting the user to enable it in:

> System Settings → Privacy & Security → Accessibility

## Building

```bash
cargo build --release
```

## Running

```bash
# Development (verbose logging)
RUST_LOG=debug cargo run

# Production (minimal logging)
RUST_LOG=info cargo run --release
```

## Auto-Start on Login

The app can install a **LaunchAgent** so it starts automatically on login:

```bash
# Install auto-start
./target/release/finder-reroute --install

# Check status
./target/release/finder-reroute --status

# Remove auto-start
./target/release/finder-reroute --uninstall
```

After `--install`, you must also grant Accessibility permission to the release binary:

> System Settings → Privacy & Security → Accessibility → + → Add the binary

## Project Structure

```
├── Cargo.toml           # Dependencies
├── src/
│   ├── main.rs          # Binary entry point (CLI + permission check + event loop)
│   ├── lib.rs           # Library root + shared state
│   ├── tap.rs           # CGEventTap installation + callback
│   ├── detector.rs      # Dock/Finder detection via AXUIElement
│   ├── launcher.rs      # App launching via `open -a`
│   └── launchd.rs       # LaunchAgent install/uninstall/status
└── RESEARCH.md          # Research notes on prior art
```

## Key Design Decisions

1. **No UI** — The binary is a background process. A UI for configuration will be added later.
2. **Title-based detection** — For the MVP, we check `AXTitleAttribute == "Finder"`. Future versions will use `AXURLAttribute` → bundle ID for robustness across languages.
3. **`open -a` for launching** — Simple, reliable, no need for AppKit/NSWorkspace bridging.
4. **Event tap at HID level** — Ensures we intercept the click before the Dock processes it.
5. **Head insert placement** — Our tap runs before other taps, giving us first chance to consume the event.
6. **LaunchAgent for auto-start** — Uses `launchd` (no `SMAppService` needed for a CLI binary). `KeepAlive` with `SuccessfulExit: false` ensures the app restarts on crashes but not on graceful shutdown.

## Future Work

- [ ] Bundle ID detection instead of title matching
- [ ] Configurable target app (currently hardcoded to "Bloom")
- [ ] Per-app overrides (e.g., different app for different folders)
- [ ] Settings UI (Swift or Tauri)
- [ ] Handle Dock in different positions (left, right, bottom)
- [ ] Multi-monitor support
- [ ] Hide Finder icon from Dock (requires SIP disable — not recommended)
