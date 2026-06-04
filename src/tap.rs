use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult,
};
use log::{debug, error, info, warn};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::detector::DockDetector;
use crate::launcher::AppLauncher;
use crate::RewireState;

/// Manages the `CGEventTap` lifecycle.
pub struct EventTap {
    #[allow(dead_code)]
    tap: CGEventTap<'static>,
}

impl EventTap {
    /// Create and install a new event tap that intercepts left-mouse-down events.
    ///
    /// # Errors
    ///
    /// Returns [`EventTapError::CreateFailed`] if the `CGEventTap` cannot be created,
    /// or [`EventTapError::RunLoopSourceFailed`] if the run-loop source fails.
    pub fn new(
        state: Arc<RewireState>,
        detector: DockDetector,
        launcher: AppLauncher,
    ) -> Result<Self, EventTapError> {
        let events = vec![CGEventType::LeftMouseDown];

        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            events,
            move |_proxy, event_type, event| {
                handle_event(event_type, event, &state, &detector, &launcher)
            },
        )
        .map_err(|()| EventTapError::CreateFailed)?;

        let run_loop_source = tap
            .mach_port()
            .create_runloop_source(0)
            .map_err(|()| EventTapError::RunLoopSourceFailed)?;

        let run_loop = CFRunLoop::get_current();
        run_loop.add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });
        tap.enable();

        info!("Event tap installed for LeftMouseDown");
        Ok(Self { tap })
    }

    /// Run the current thread's `CFRunLoop`. This blocks until the process exits.
    pub fn run(&self) {
        info!("Running CFRunLoop — click Finder to launch target app.");
        CFRunLoop::run_current();
    }
}

/// Errors that can occur when creating the event tap.
#[derive(Debug, thiserror::Error)]
pub enum EventTapError {
    #[error("Failed to create CGEventTap — check Accessibility permission")]
    CreateFailed,
    #[error("Failed to create CFRunLoopSource")]
    RunLoopSourceFailed,
}

/// Per-event callback invoked by the `CGEventTap`.
fn handle_event(
    event_type: CGEventType,
    event: &CGEvent,
    state: &Arc<RewireState>,
    detector: &DockDetector,
    launcher: &AppLauncher,
) -> CallbackResult {
    // Handle special out-of-band events that disable the tap.
    match event_type as u32 {
        0xFFFF_FFFE => {
            // TapDisabledByTimeout
            warn!("Event tap disabled by timeout.");
            return CallbackResult::Keep;
        }
        0xFFFF_FFFF => {
            // TapDisabledByUserInput
            warn!("Event tap disabled by user input.");
            return CallbackResult::Keep;
        }
        _ => {}
    }

    // Quick exit if globally disabled.
    if !state.enabled.load(Ordering::Relaxed) {
        return CallbackResult::Keep;
    }

    // Only handle left-mouse-down.
    if (event_type as u32) != (CGEventType::LeftMouseDown as u32) {
        return CallbackResult::Keep;
    }

    let location = event.location();
    debug!("LeftMouseDown at ({}, {})", location.x, location.y);

    // Check if the click is on the Finder Dock icon.
    if detector.is_finder_click(location.x, location.y) {
        info!("Finder click detected — consuming event and launching target app.");

        if let Err(e) = launcher.launch() {
            error!("Failed to launch target app: {e}");
        }

        // Consume the event so the Dock never sees it.
        return CallbackResult::Drop;
    }

    // Pass the event through to the Dock.
    CallbackResult::Keep
}
