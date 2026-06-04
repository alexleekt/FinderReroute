# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-04

### Added
- Initial release of FinderReroute
- Rust core with CGEventTap for HID-level mouse event interception
- AXUIElement detection for Finder icon in Dock
- App launching via `open -a` to alternative file managers
- SwiftUI menu bar app with folder icon
- App selector to choose target file manager (Bloom, Path Finder, ForkLift, etc.)
- Toggle to start/stop interception
- Auto-start via LaunchAgent (`--install`, `--uninstall`, `--status`)
- Shell override for `open` command (`--setup-shell`, `--uninstall-shell`)
- Custom folder icon generated with Python PIL
- Single `.app` bundle in `/Applications`
- Build script (`build.sh`) for one-command rebuild
- Icon generation script (`create_icon.py`)

## [Unreleased]

### Planned
- Bundle ID detection for robust cross-language support
- Per-folder-type app selection
- Multi-monitor support
- Dock position awareness (left, right, bottom)
