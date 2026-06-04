use axuielement::prelude::*;
use log::{debug, trace};
use std::sync::atomic::{AtomicBool, Ordering};

/// Detects whether a mouse click is on the Finder icon in the Dock.
#[derive(Debug)]
pub struct DockDetector {
    /// Whether detection is enabled.
    enabled: AtomicBool,
    /// The Finder title to match (localized, but "Finder" works for English).
    finder_title: String,
    /// The Dock subrole for app icons.
    app_dock_subrole: String,
}

impl DockDetector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            finder_title: "Finder".to_string(),
            app_dock_subrole: "AXApplicationDockItem".to_string(),
        }
    }

    /// Check if the click at (x, y) is on the Finder Dock icon.
    pub fn is_finder_click(&self, x: f64, y: f64) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }

        // Hit-test the element at the click position.
        let Some(system) = system_wide() else {
            trace!("system_wide accessibility element not available");
            return false;
        };

        #[allow(clippy::cast_possible_truncation)]
        let element = match system.element_at_position(x as f32, y as f32) {
            Ok(Some(e)) => e,
            Ok(None) => {
                trace!("no element at ({x}, {y})");
                return false;
            }
            Err(e) => {
                trace!("AX error at ({x}, {y}): {e:?}");
                return false;
            }
        };

        // Check subrole — must be an app dock item.
        let Ok(Some(subrole)) =
            element.string_attribute(axuielement::ax_attribute::AX_SUBROLE_ATTRIBUTE)
        else {
            trace!("element has no subrole — not a Dock item");
            return false;
        };

        if subrole != self.app_dock_subrole {
            trace!("subrole='{subrole}' — not an app dock item");
            return false;
        }

        // Check title — must be "Finder".
        let Ok(Some(title)) =
            element.string_attribute(axuielement::ax_attribute::AX_TITLE_ATTRIBUTE)
        else {
            trace!("dock app item has no title");
            return false;
        };

        debug!("Dock app item at ({x}, {y}): title='{title}' subrole='{subrole}'");

        if title == self.finder_title {
            return true;
        }

        trace!("title='{title}' — not Finder");
        false
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
}

impl Default for DockDetector {
    fn default() -> Self {
        Self::new()
    }
}
