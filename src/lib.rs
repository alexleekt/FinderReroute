pub mod detector;
pub mod launcher;
pub mod launchd;
pub mod shell;
pub mod tap;

use log::info;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Check if the process has been granted Accessibility permission.
#[must_use] 
pub fn has_accessibility_permission() -> bool {
    axuielement::is_process_trusted()
}

/// Prompt the user to grant Accessibility permission.
pub fn prompt_accessibility_permission() {
    // Calling is_process_trusted_with_prompt would show the system dialog
    // For now, we just log and exit; the user must grant it manually.
    info!("Accessibility permission required. Please grant it in System Settings > Privacy & Security > Accessibility.");
}

/// Shared state for the event tap callback.
#[derive(Debug)]
pub struct RewireState {
    /// Whether the tap is currently enabled.
    pub enabled: AtomicBool,
    /// The target app to launch instead of Finder.
    pub target_app: String,
    /// The bundle ID to intercept (default: com.apple.finder).
    pub target_bundle_id: String,
}

impl RewireState {
    pub fn new(target_app: impl Into<String>, target_bundle_id: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            enabled: AtomicBool::new(true),
            target_app: target_app.into(),
            target_bundle_id: target_bundle_id.into(),
        })
    }
}
