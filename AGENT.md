# Agent Guidelines

## Communication Style
- Be direct and concise. Prioritize actionable guidance over verbose narration.
- When requirements are unclear, ask 1-2 clarifying questions rather than guessing.
- Show code diffs when relevant; summarize when not.

## Code Conventions

### Rust
- Follow existing conventions in the project.
- All code must pass `cargo clippy` at `warn` level for `all`, `pedantic`, `nursery`, and `cargo` groups (configured in `Cargo.toml`).
- Prefer explicit error handling with `thiserror` over `unwrap`/`expect` in production code.
- Use `log` + `env_logger` for logging; avoid `println!` in library code.

### SwiftUI
- Follow existing conventions in `ui/Sources/FinderRerouteUI/`.
- All code must pass `swiftlint` rules as configured in `ui/.swiftlint.yml`.
- Prefer `MenuBarExtra` and background-agent patterns (`LSUIElement`); no dock icons or main windows.

## Workflow Rules
- Run linting before considering any change complete:
  1. `cargo clippy` (repo root)
  2. `cd ui && swiftlint lint` (if Swift files changed)
- Always run `./build.sh` after changes that affect the app bundle, binary, or UI.
- Ask before installing new Rust crates or Swift packages.
- Never commit changes unless explicitly asked.
- Do not modify `FinderReroute.app/Contents/Info.plist` or the app bundle structure without explicit instruction.

## Tool Usage
- Use `edit_file` for precise, targeted changes.
- Use `grep` for searching symbols across Rust and Swift sources.
- Use `read_file` for understanding specific files before editing.
- Use `terminal` for running `cargo`, `swift`, and `swiftlint` commands.

## macOS-Specific Considerations
- This is a system-level utility using `CGEventTap` and `AXUIElement`.
- Never suggest disabling SIP (System Integrity Protection) as a solution.
- Code changes affecting accessibility or input monitoring permissions must be called out explicitly.
- The bundled binary structure (`FinderReroute.app/Contents/MacOS/`) is load-bearing; preserve it.

## Testing Requirements
- Run `cargo clippy` for all Rust changes.
- Run `./build.sh` to verify the full build pipeline (Rust + SwiftUI + icon) when build-related files change.
- Use `src/bin/test_detector.rs` to validate Dock detection changes when relevant.

## Approval Workflows
- Ask before adding new dependencies to `Cargo.toml` or `ui/Package.swift`.
- Ask before changing lint levels in `Cargo.toml` or `ui/.swiftlint.yml`.
- Ask before modifying `build.sh` or the app bundle structure.
