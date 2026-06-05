pub mod detector;
pub mod launchd;
pub mod launcher;
pub mod shell;
pub mod tap;

use log::info;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

/// Check if the process has been granted Accessibility permission.
#[must_use]
pub fn has_accessibility_permission() -> bool {
    axuielement::is_process_trusted()
}

/// Log instructions for granting Accessibility permission.
///
/// Note: The app does not show a system dialog automatically.
/// The user must manually grant the permission in System Settings.
pub fn prompt_accessibility_permission() {
    info!("Accessibility permission required. Please grant it in System Settings > Privacy & Security > Accessibility.");
}

/// Read the target app name from the shared config file.
///
/// Returns the app name from `~/.config/finder-reroute/config.json`
/// (written by the `SwiftUI` menu bar app), or `"Bloom"` as a fallback.
#[must_use]
pub fn read_config_app() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = PathBuf::from(home).join(".config/finder-reroute/config.json");

    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(app) = json.get("app").and_then(|v| v.as_str()) {
                return app.to_string();
            }
        }
    }

    "Bloom".to_string()
}

/// Shared state for the event tap callback.
#[derive(Debug)]
pub struct RerouteState {
    /// Whether the tap is currently enabled.
    pub enabled: AtomicBool,
    /// The app to launch instead of Finder (e.g., "Bloom").
    pub replacement_app: String,
    /// The bundle ID of the app to intercept (default: com.apple.finder).
    pub intercepted_bundle_id: String,
}

impl RerouteState {
    pub fn new(
        replacement_app: impl Into<String>,
        intercepted_bundle_id: impl Into<String>,
    ) -> Self {
        Self {
            enabled: AtomicBool::new(true),
            replacement_app: replacement_app.into(),
            intercepted_bundle_id: intercepted_bundle_id.into(),
        }
    }
}
