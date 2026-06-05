# FinderReroute

A macOS app that intercepts clicks on the **Finder** icon in the Dock and launches an alternative file manager (e.g., **Bloom**) instead.

![FinderReroute Icon](FinderReroute.app/Contents/Resources/AppIcon.icns)

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

## Features

- **Intercept Finder clicks** — Clicking the Finder icon in the Dock opens your chosen file manager
- **Menu bar UI** — Simple folder icon in the menu bar to toggle interception and select target app
- **Auto-start** — Optional LaunchAgent to start automatically on login
- **Shell override** — Make `open ~/folder` use your chosen file manager in Terminal
- **Configurable** — Choose any installed file manager (Bloom, Path Finder, ForkLift, etc.)

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Event interception | Rust + `core-graphics` 0.25 | `CGEventTap` for HID-level mouse event capture |
| Run loop | Rust + `core-foundation` 0.10 | `CFRunLoop` + `CFMachPort` for tap lifecycle |
| Dock detection | Rust + `axuielement` 0.9 | `AXUIElementCopyElementAtPosition` + attribute reading |
| App launching | Rust + `std::process::Command` | `open -a <app>` for launching macOS apps |
| UI | SwiftUI + `MenuBarExtra` | Menu bar app with app selector and toggle |
| Auto-start | `launchd` LaunchAgent | Runs automatically on login |

## Requirements

- macOS 13.0+
- **Accessibility** permission — Required for `AXUIElementCopyElementAtPosition`
- **Input Monitoring** permission — Required for `CGEventTap` (HID-level tap)

You must manually grant these permissions before first use:

> System Settings → Privacy & Security → Accessibility → + → Add `FinderReroute.app`
>
> System Settings → Privacy & Security → Input Monitoring → + → Add `FinderReroute.app`

## Installation

### Option 1: Download the .app bundle

1. Download `FinderReroute.app` from [Releases](../../releases)
2. Drag it to `/Applications/`
3. Launch it — the folder icon appears in your menu bar
4. Click the icon and toggle **"Intercept Finder clicks"**
5. Grant Accessibility permission when prompted

### Option 2: Build from source

```bash
# Clone the repo
git clone https://github.com/alexleekt/FinderReroute.git
cd FinderReroute

# Build everything (Rust + SwiftUI + icon)
./build.sh

# Copy to /Applications
cp -R FinderReroute.app /Applications/

# Launch
/Applications/FinderReroute.app/Contents/MacOS/FinderReroute
```

**Prerequisites for building:**
- [Rust](https://rustup.rs/) (latest stable)
- [Swift](https://developer.apple.com/swift/) (Xcode Command Line Tools)
- Python 3 with PIL (for icon generation)

## Usage

### Menu Bar

1. Look for the **folder icon** in your menu bar (top-right)
2. Click it to open the control panel
3. Select your preferred file manager from the dropdown
4. Toggle **"Intercept Finder clicks"** to start/stop interception

### CLI Commands

The bundled Rust binary supports CLI commands:

```bash
# Install auto-start LaunchAgent
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --install

# Check status
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --status

# Remove auto-start
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --uninstall

# Setup shell override
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --setup-shell

# Remove shell override
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --uninstall-shell
```

### Shell Override

The shell override makes `open ~/folder` in Terminal use your chosen file manager instead of Finder:

```bash
# Setup (supports Fish, Zsh, Bash)
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --setup-shell

# Examples after setup
open ~/Downloads        # Opens in Bloom (or your chosen app)
open file.txt           # Uses default app (unchanged)
open -a Finder ~/Downloads  # Force Finder (override bypassed)
```

## Auto-Start on Login

After installing the LaunchAgent:

```bash
/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --install
```

Grant Accessibility permission to the app:

> System Settings → Privacy & Security → Accessibility → + → Add `FinderReroute.app`

The app will now start automatically on every login.

## Project Structure

```
├── FinderReroute.app/          # macOS app bundle (SwiftUI + Rust)
│   └── Contents/
│       ├── Info.plist
│       ├── MacOS/
│       │   ├── FinderReroute       # SwiftUI menu bar app (main executable)
│       │   └── finder-reroute      # Rust interceptor (bundled)
│       └── Resources/
│           └── AppIcon.icns         # Custom folder icon
├── Cargo.toml                  # Dependencies + Clippy lint config
├── .gitignore
├── build.sh                      # Build script
├── create_icon.py                # Icon generation script
├── src/                          # Rust source
│   ├── main.rs                   # CLI entry point
│   ├── lib.rs                    # Library root + shared state
│   ├── tap.rs                    # CGEventTap installation
│   ├── detector.rs               # Dock/Finder detection
│   ├── launcher.rs               # App launching
│   ├── shell.rs                  # Shell override injection
│   ├── launchd.rs                # LaunchAgent management
│   └── bin/
│       └── test_detector.rs      # Diagnostic tool for Dock element scanning
├── ui/                           # SwiftUI source
│   ├── Package.swift
│   ├── .swiftlint.yml
│   └── Sources/FinderRerouteUI/
│       ├── FinderRerouteUI.swift  # App entry + AppState
│       └── ContentView.swift      # Menu bar UI
└── README.md
```

## Key Design Decisions

1. **Single .app bundle** — The SwiftUI app bundles the Rust binary internally. One app, one entry point.
2. **Menu bar only** — The app is a background agent. No dock icon, no window. `LSUIElement` is set.
3. **Title-based detection** (known limitation) — Checks `AXTitleAttribute == "Finder"`. Bundle ID detection is planned for a future release.
4. **`open -a` for launching** — Simple, reliable, no need for AppKit bridging.
5. **Event tap at HID level** — Intercepts clicks before the Dock processes them.
6. **Head insert placement** — Our tap runs before other taps, giving us first chance to consume the event.
7. **LaunchAgent for auto-start** — Uses `launchd`. `KeepAlive` with `SuccessfulExit: false` and `ThrottleInterval` (60s) restarts on crashes with a cooldown.
8. **Shell override** — Injects a function into shell config to redirect `open` on directories.

## Troubleshooting

### App doesn't appear in menu bar
- Check if the app is running: `ps aux | grep FinderReroute`
- Try killing and relaunching: `killall FinderReroute`
- Check if menu bar is too crowded — close other apps

### Interception not working
- Grant **Accessibility** permission: System Settings → Privacy & Security → Accessibility → Add `FinderReroute.app`
- Grant **Input Monitoring** permission: System Settings → Privacy & Security → Input Monitoring → Add `FinderReroute.app`
- Check logs: `tail -f /tmp/com.alexleekt.finder-reroute.err.log`

### Auto-start not working
- Check LaunchAgent status: `/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --status`
- Reinstall: `/Applications/FinderReroute.app/Contents/MacOS/finder-reroute --install`
- Check logs: `tail -f /tmp/com.alexleekt.finder-reroute.out.log`
- Note: Both Accessibility and Input Monitoring permissions are required for the auto-start LaunchAgent.

## Development

### Linting

The project enforces code quality via **Clippy** (Rust) and **SwiftLint** (Swift).

**Rust — Clippy:**

```bash
# Check all targets (lib, main binary, test_detector binary)
cargo clippy

# Auto-fix trivial issues
cargo clippy --fix --allow-dirty --allow-staged
```

Clippy is configured in `Cargo.toml` with the `all`, `pedantic`, `nursery`, and `cargo` lint groups at `warn` level.

**Swift — SwiftLint:**

```bash
cd ui
swiftlint lint

# Auto-fix (where supported)
swiftlint lint --fix
```

SwiftLint is configured in `ui/.swiftlint.yml` with standard opt-in rules and sensible thresholds.

### Running the Swift UI

```bash
cd ui
swift build
swift run
```

The Swift UI provides a menu-bar interface for starting/stopping the interceptor and selecting the target app.

## Future Work

- [ ] Bundle ID detection instead of title matching (robust across languages)
- [ ] Configurable target app per folder type
- [ ] Handle Dock in different positions (left, right, bottom)
- [ ] Multi-monitor support
- [ ] Hide Finder icon from Dock (requires SIP disable — not recommended)
- [ ] Code signing and notarization for distribution
- [ ] Add ad-hoc code signing step to `build.sh` for local testing

## License

MIT License — See [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [SwiftUI](https://developer.apple.com/xcode/swiftui/)
- Icon generated with Python PIL
