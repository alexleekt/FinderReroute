use log::{debug, info};
use std::process::Command;

/// Launches a target macOS application by name.
#[derive(Debug, Clone)]
pub struct AppLauncher {
    app_name: String,
}

impl AppLauncher {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    /// Launch the configured app using `open -a`.
    ///
    /// Uses `spawn()` instead of `output()` so the event-tap callback
    /// returns immediately and does not trigger a tap timeout.
    ///
    /// # Errors
    ///
    /// Returns [`LauncherError::SpawnFailed`] if the `open` command fails to spawn.
    pub fn launch(&self) -> Result<(), LauncherError> {
        info!("Launching '{}' via open -a", self.app_name);

        let _child = Command::new("open")
            .args(["-a", &self.app_name])
            .spawn()
            .map_err(|e| LauncherError::SpawnFailed(e.to_string()))?;

        // Child runs independently; we don't wait so the tap callback
        // returns immediately.
        debug!("Successfully launched '{}'", self.app_name);
        Ok(())
    }

    /// Launch a specific file or folder with the target app.
    ///
    /// Uses `spawn()` instead of `output()` so the event-tap callback
    /// returns immediately and does not trigger a tap timeout.
    ///
    /// # Errors
    ///
    /// Returns [`LauncherError::SpawnFailed`] if the `open` command fails to spawn.
    pub fn open_with(&self, path: &str) -> Result<(), LauncherError> {
        info!("Opening '{}' with '{}'", path, self.app_name);

        let _child = Command::new("open")
            .args(["-a", &self.app_name, path])
            .spawn()
            .map_err(|e| LauncherError::SpawnFailed(e.to_string()))?;

        // Child runs independently; we don't wait so the tap callback
        // returns immediately.
        debug!("Successfully opened '{}' with '{}'", path, self.app_name);
        Ok(())
    }
}

impl Default for AppLauncher {
    fn default() -> Self {
        Self::new("Bloom")
    }
}

/// Errors that can occur when launching an app.
#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error("Failed to spawn `open` command: {0}")]
    SpawnFailed(String),
    #[error("App launch failed: {0}")]
    LaunchFailed(String),
}
