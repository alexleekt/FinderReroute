# FinderReroute

A macOS app that intercepts clicks on the **Finder** icon in the Dock and launches an alternative file manager (e.g., **Bloom**) instead.

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

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Event interception | Rust + `core-graphics` 0.25 | `CGEventTap` for HID-level mouse event capture |
| Run loop | Rust + `core-foundation` 0.10 | `CFRunLoop` + `CFMachPort` for tap lifecycle |
| Dock detection | Rust + `axuielement` 0.9 | `AXUIElementCopyElementAtPosition` + attribute reading |
| App launching | Rust + `std::process::Command` | `open -a <app>` for launching macOS apps |
| UI | SwiftUI + `MenuBarExtra` | Menu bar app with app selector and toggle |
| Auto-start | `launchd` LaunchAgent | Runs automatically on login |

## Required macOS Permissions

- **Accessibility** — Required for `AXUIElementCopyElementAtPosition`
- **Input Monitoring** — Required for `CGEventTap` (HID-level tap)

The app will exit on startup if Accessibility is not granted, prompting the user to enable it in:

> System Settings → Privacy & Security → Accessibility

## Installation

### Option 1: Download the .app bundle

1. Download `FinderReroute.app`
2. Drag it to `/Applications/`
3. Launch it — the folder icon appears in your menu bar
4. Click the icon and toggle **"Intercept Finder clicks"**
5. Grant Accessibility permission when prompted

### Option 2: Build from source

```bash
# Build everything
./build.sh

# Copy to /Applications
cp -R FinderReroute.app /Applications/

# Launch
/Applications/FinderReroute.app/Contents/MacOS/FinderReroute
```

## Auto-Start on Login

The app can install a **LaunchAgent** so it starts automatically on login:

```bash
# Install auto-start
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --install

# Check status
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --status

# Remove auto-start
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --uninstall
```

After `--install`, you must grant Accessibility permission to the app:

> System Settings → Privacy & Security → Accessibility → + → Add `FinderReroute.app`

## Shell Override (Optional)

You can also make the `open` command in Terminal open folders with Bloom instead of Finder:

```bash
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --setup-shell
```

This adds a shell function to your config (Fish/Zsh/Bash) so:
- `open ~/Downloads` → opens in Bloom
- `open file.txt` → uses default app (unchanged)

## Project Structure

```
├── FinderReroute.app/          # macOS app bundle (SwiftUI + Rust)
│   └── Contents/MacOS/
│       ├── FinderReroute       # SwiftUI menu bar app
│       └── finder-reroute      # Rust interceptor
├── src/                        # Rust source
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library root
│   ├── tap.rs                  # CGEventTap
│   ├── detector.rs             # Dock detection
│   ├── launcher.rs             # App launching
│   ├── launchd.rs              # LaunchAgent management
│   └── shell.rs                # Shell override
├── ui/                         # SwiftUI source
│   └── Sources/
│       ├── FinderRerouteUI.swift
│       └── ContentView.swift
├── build.sh                    # Build script
├── Cargo.toml
└── RESEARCH.md
```

## Key Design Decisions

1. **Single .app bundle** — The SwiftUI app bundles the Rust binary internally. One app, one entry point.
2. **Menu bar only** — The app is a background agent. No dock icon, no window.
3. **Title-based detection** — Checks `AXTitleAttribute == "Finder"`. Future versions will use bundle ID.
4. **`open -a` for launching** — Simple, reliable, no need for AppKit bridging.
5. **Event tap at HID level** — Intercepts clicks before the Dock processes them.
6. **LaunchAgent for auto-start** — Uses `launchd`. `KeepAlive` with `SuccessfulExit: false` restarts on crashes.

## Future Work

- [ ] Bundle ID detection instead of title matching
- [ ] Configurable target app per folder type
- [ ] Handle Dock in different positions (left, right, bottom)
- [ ] Multi-monitor support
- [ ] Hide Finder icon from Dock (requires SIP disable)
