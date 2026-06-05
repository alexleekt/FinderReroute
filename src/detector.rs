use axuielement::prelude::*;
use log::{debug, trace};

/// The Finder title to match (localized, but "Finder" works for English).
const FINDER_TITLE: &str = "Finder";
/// The Dock subrole for app icons.
const APP_DOCK_SUBROLE: &str = "AXApplicationDockItem";

/// Detects whether a mouse click is on the Finder icon in the Dock.
#[derive(Debug, Default)]
pub struct DockDetector;

impl DockDetector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Check if the click at (x, y) is on the Finder Dock icon.
    #[must_use]
    pub fn is_finder_click(&self, x: f64, y: f64) -> bool {
        // Hit-test the element at the click position.
        let Some(system) = system_wide() else {
            trace!("system_wide accessibility element not available");
            return false;
        };

        // SAFETY: macOS screen coordinates are well within f32 exact-integer range
        // (< 16M pixels) even for multi-monitor setups. The axuielement API expects
        // f32, so this cast is necessary and safe.
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

        if subrole != APP_DOCK_SUBROLE {
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

        if title == FINDER_TITLE {
            return true;
        }

        trace!("title='{title}' — not Finder");
        false
    }
}
