//! Recording overlay window management commands.
//!
//! The recording overlay is a floating panel (NSPanel on macOS, standard window elsewhere)
//! that displays the current recording state. It appears when the user triggers recording
//! via the global shortcut and provides visual feedback for the recording workflow.

use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl};

use crate::domain::RecordingState;
use crate::services::{recording_service, recording_state};

// ============================================================================
// Constants
// ============================================================================

/// Window label for the recording overlay
const RECORDING_OVERLAY_LABEL: &str = "recording-overlay";

/// Recording overlay window dimensions (compact status indicator)
const RECORDING_OVERLAY_WIDTH: f64 = 200.0;
const RECORDING_OVERLAY_HEIGHT: f64 = 80.0;

static LAST_SHOW_INSTANT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn last_show_instant() -> &'static Mutex<Option<Instant>> {
    LAST_SHOW_INSTANT.get_or_init(|| Mutex::new(None))
}

// ============================================================================
// macOS-specific: NSPanel support
// ============================================================================

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask,
};

// Define custom panel class for recording overlay (macOS only)
#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(RecordingOverlayPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

// ============================================================================
// Window Initialization
// ============================================================================

/// Creates the recording overlay window at app startup.
/// Must be called from the main thread (e.g., in setup()).
/// The window starts hidden and is shown via show_recording_overlay command.
pub fn init_recording_overlay(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        init_recording_overlay_macos(app)
    }

    #[cfg(not(target_os = "macos"))]
    {
        init_recording_overlay_standard(app)
    }
}

/// Creates the recording overlay as an NSPanel on macOS (hidden).
#[cfg(target_os = "macos")]
fn init_recording_overlay_macos(app: &AppHandle) -> Result<(), String> {
    use tauri::{LogicalSize, Size};

    log::debug!("Creating recording overlay as NSPanel (macOS)");

    let panel = PanelBuilder::<_, RecordingOverlayPanel>::new(app, RECORDING_OVERLAY_LABEL)
        .url(WebviewUrl::App("recording-overlay.html".into()))
        .title("Recording")
        .size(Size::Logical(LogicalSize::new(
            RECORDING_OVERLAY_WIDTH,
            RECORDING_OVERLAY_HEIGHT,
        )))
        .level(PanelLevel::Status) // Status level to appear above fullscreen apps
        .transparent(true)
        .has_shadow(true)
        .collection_behavior(
            CollectionBehavior::new()
                .full_screen_auxiliary()
                .can_join_all_spaces(),
        )
        .style_mask(StyleMask::empty().nonactivating_panel())
        .hides_on_deactivate(false) // Stay visible when clicking other apps
        .works_when_modal(true)
        .with_window(|w| {
            w.decorations(false)
                .transparent(true)
                .skip_taskbar(true)
                .resizable(false)
                .center()
        })
        .build()
        .map_err(|e| format!("Failed to create recording overlay panel: {e}"))?;

    // Start hidden - will be shown via show_recording_overlay command
    panel.hide();
    log::info!("Recording overlay NSPanel created (hidden)");
    Ok(())
}

/// Creates the recording overlay as a standard Tauri window (hidden) on non-macOS platforms.
#[cfg(not(target_os = "macos"))]
fn init_recording_overlay_standard(app: &AppHandle) -> Result<(), String> {
    use tauri::webview::WebviewWindowBuilder;

    log::debug!("Creating recording overlay as standard window");

    WebviewWindowBuilder::new(
        app,
        RECORDING_OVERLAY_LABEL,
        WebviewUrl::App("recording-overlay.html".into()),
    )
    .title("Recording")
    .inner_size(RECORDING_OVERLAY_WIDTH, RECORDING_OVERLAY_HEIGHT)
    .always_on_top(true)
    .skip_taskbar(true)
    .decorations(false)
    .transparent(true)
    .visible(false) // Start hidden
    .resizable(false)
    .center()
    .build()
    .map_err(|e| format!("Failed to create recording overlay window: {e}"))?;

    log::info!("Recording overlay window created (hidden)");
    Ok(())
}

// ============================================================================
// Window Positioning
// ============================================================================

/// Gets the monitor containing the given cursor position, falling back to primary monitor.
fn get_monitor_for_cursor(
    app: &AppHandle,
    cursor_pos: tauri::PhysicalPosition<f64>,
) -> Option<tauri::Monitor> {
    match app.monitor_from_point(cursor_pos.x, cursor_pos.y) {
        Ok(Some(m)) => Some(m),
        Ok(None) => {
            log::warn!("No monitor found at cursor position, trying primary monitor");
            app.primary_monitor().ok().flatten()
        }
        Err(e) => {
            log::warn!("Failed to get monitor from point: {e}");
            app.primary_monitor().ok().flatten()
        }
    }
}

/// Calculates the position to center a window on the monitor containing the cursor.
/// Falls back to primary monitor if cursor monitor cannot be determined.
fn get_centered_position_on_cursor_monitor(
    app: &AppHandle,
) -> Option<tauri::PhysicalPosition<i32>> {
    // Get cursor position
    let cursor_pos = match app.cursor_position() {
        Ok(pos) => pos,
        Err(e) => {
            log::warn!("Failed to get cursor position: {e}");
            return None;
        }
    };

    log::debug!("Cursor position: ({}, {})", cursor_pos.x, cursor_pos.y);

    // Get the monitor containing the cursor
    let monitor = get_monitor_for_cursor(app, cursor_pos)?;

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    let scale_factor = monitor.scale_factor();

    log::debug!(
        "Monitor: pos=({}, {}), size={}x{}, scale={}",
        monitor_pos.x,
        monitor_pos.y,
        monitor_size.width,
        monitor_size.height,
        scale_factor
    );

    // Calculate centered position on this monitor
    // Window size needs to be scaled by the monitor's scale factor
    let scaled_width = (RECORDING_OVERLAY_WIDTH * scale_factor) as i32;
    let scaled_height = (RECORDING_OVERLAY_HEIGHT * scale_factor) as i32;

    let x = monitor_pos.x + (monitor_size.width as i32 - scaled_width) / 2;
    let y = monitor_pos.y + (monitor_size.height as i32 - scaled_height) / 2;

    log::debug!("Calculated position: ({x}, {y})");

    Some(tauri::PhysicalPosition::new(x, y))
}

/// Positions the recording overlay window centered on the monitor containing the cursor.
fn position_recording_overlay_on_cursor_monitor(app: &AppHandle) {
    if let Some(position) = get_centered_position_on_cursor_monitor(app) {
        if let Some(window) = app.get_webview_window(RECORDING_OVERLAY_LABEL) {
            if let Err(e) = window.set_position(position) {
                log::warn!("Failed to set window position: {e}");
            }
        }
    }
}

// ============================================================================
// Window Visibility
// ============================================================================

/// Returns whether the recording overlay is currently visible.
fn is_recording_overlay_visible(app: &AppHandle) -> bool {
    #[cfg(target_os = "macos")]
    {
        app.get_webview_panel(RECORDING_OVERLAY_LABEL)
            .map(|panel| panel.is_visible())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "macos"))]
    {
        app.get_webview_window(RECORDING_OVERLAY_LABEL)
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false)
    }
}

/// Payload emitted when the recording overlay is shown.
#[derive(Clone, serde::Serialize)]
pub struct RecordingOverlayShownPayload {
    /// Time in milliseconds for the show command to return
    pub show_call_ms: u64,
}

/// Payload emitted when the recording state changes.
#[derive(Clone, serde::Serialize)]
pub struct RecordingStateChangedPayload {
    pub state: RecordingState,
}

/// Shows the recording overlay window without stealing focus.
#[tauri::command]
#[specta::specta]
pub fn show_recording_overlay(app: AppHandle) -> Result<(), String> {
    let start = Instant::now();
    log::info!("Showing recording overlay window");

    if let Ok(mut guard) = last_show_instant().lock() {
        *guard = Some(start);
    }

    position_recording_overlay_on_cursor_monitor(&app);

    #[cfg(target_os = "macos")]
    {
        let panel = app
            .get_webview_panel(RECORDING_OVERLAY_LABEL)
            .map_err(|e| format!("Recording overlay panel not found: {e:?}"))?;
        panel.show();
        log::debug!("Recording overlay panel shown (macOS)");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let window = app
            .get_webview_window(RECORDING_OVERLAY_LABEL)
            .ok_or_else(|| {
                "Recording overlay window not found - was init_recording_overlay called at startup?"
                    .to_string()
            })?;
        window
            .show()
            .map_err(|e| format!("Failed to show window: {e}"))?;
        log::debug!("Recording overlay window shown");
    }

    let elapsed_ms = start.elapsed().as_millis() as u64;
    log::info!("Recording overlay show call completed in {elapsed_ms}ms");

    // Emit event for frontend to update state
    if let Err(e) = app.emit(
        "recording-overlay-shown",
        RecordingOverlayShownPayload {
            show_call_ms: elapsed_ms,
        },
    ) {
        log::error!("Failed to emit recording-overlay-shown event: {e}");
    }

    // Update state for listeners
    recording_state::set_recording_state(RecordingState::Recording);
    if let Err(e) = app.emit(
        "recording-state-changed",
        RecordingStateChangedPayload {
            state: RecordingState::Recording,
        },
    ) {
        log::error!("Failed to emit recording-state-changed event: {e}");
    }

    Ok(())
}

/// Called by the frontend when the overlay has rendered.
/// This measures actual UI render time against the show timestamp.
#[tauri::command]
#[specta::specta]
pub fn report_recording_overlay_rendered(_app: AppHandle) -> Result<(), String> {
    let elapsed_ms = match last_show_instant().lock() {
        Ok(mut guard) => guard.take().map(|start| start.elapsed().as_millis() as u64),
        Err(err) => {
            log::error!("Failed to lock render timing mutex: {err}");
            None
        }
    };

    if let Some(render_ms) = elapsed_ms {
        log::info!("Recording overlay rendered in {render_ms}ms");
        if render_ms > 50 {
            log::warn!("Recording overlay render exceeded 50ms target: {render_ms}ms");
        }
    } else {
        log::warn!("Recording overlay render reported but no show timestamp recorded");
    }

    Ok(())
}

/// Dismisses the recording overlay window.
/// On macOS, resigns key window status before hiding to avoid activating main window.
#[tauri::command]
#[specta::specta]
pub fn dismiss_recording_overlay(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(RECORDING_OVERLAY_LABEL) {
            // Guard: resign_key_window triggers blur event which could call dismiss again
            if !panel.is_visible() {
                return Ok(());
            }
            log::info!("Dismissing recording overlay window");
            // Resign key window BEFORE hiding to prevent macOS from
            // activating our main window (which would cause space switching)
            panel.resign_key_window();
            panel.hide();
            log::debug!("Recording overlay panel dismissed (macOS)");
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window(RECORDING_OVERLAY_LABEL) {
            let is_visible = window.is_visible().unwrap_or(false);
            if !is_visible {
                log::debug!("Recording overlay already hidden, skipping");
                return Ok(());
            }
            log::info!("Dismissing recording overlay window");
            window
                .hide()
                .map_err(|e| format!("Failed to hide window: {e}"))?;
            log::debug!("Recording overlay window hidden");
        }
    }

    recording_state::set_recording_state(RecordingState::Idle);
    if let Err(e) = app.emit(
        "recording-state-changed",
        RecordingStateChangedPayload {
            state: RecordingState::Idle,
        },
    ) {
        log::error!("Failed to emit recording-state-changed event: {e}");
    }

    if let Err(e) = app.emit("recording-overlay-dismissed", ()) {
        log::error!("Failed to emit recording-overlay-dismissed event: {e}");
    }

    Ok(())
}

/// Toggles the recording overlay window visibility.
#[tauri::command]
#[specta::specta]
pub fn toggle_recording_overlay(app: AppHandle) -> Result<(), String> {
    log::info!("Toggling recording overlay window");

    if is_recording_overlay_visible(&app) {
        dismiss_recording_overlay(app)
    } else {
        show_recording_overlay(app)
    }
}

/// Cancels the current recording, dismisses the overlay, and returns to idle state.
/// This is called when the user clicks on the overlay during recording.
#[tauri::command]
#[specta::specta]
pub fn cancel_recording(app: AppHandle) -> Result<(), String> {
    log::info!("Cancelling recording via overlay click");

    // Dismiss the overlay first
    dismiss_recording_overlay(app.clone())?;

    let cleared_samples = recording_service::cancel_recording();
    log::info!("Cancelled recording, discarded {cleared_samples} audio samples");

    // Emit recording-cancelled event for state management
    if let Err(e) = app.emit("recording-cancelled", ()) {
        log::error!("Failed to emit recording-cancelled event: {e}");
    }

    log::info!("Recording cancelled, state returned to idle");
    Ok(())
}

/// Opens the macOS System Preferences to the Privacy > Microphone settings.
/// This is useful when the user denies microphone permission and needs to grant it.
#[tauri::command]
#[specta::specta]
pub fn open_microphone_settings(_app: AppHandle) -> Result<(), String> {
    log::info!("Opening microphone settings");

    // macOS deep link to Privacy > Microphone
    #[cfg(target_os = "macos")]
    {
        let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone";
        tauri_plugin_opener::open_url(url, None::<&str>)
            .map_err(|e| format!("Failed to open microphone settings: {e}"))?;
        log::info!("Opened microphone settings");
    }

    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Opening microphone settings is only supported on macOS");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_overlay_label_is_valid() {
        assert!(!RECORDING_OVERLAY_LABEL.is_empty());
        assert_eq!(RECORDING_OVERLAY_LABEL, "recording-overlay");
    }

    #[test]
    fn test_recording_overlay_dimensions_are_reasonable() {
        // Overlay should be compact (status indicator)
        assert!(RECORDING_OVERLAY_WIDTH >= 100.0);
        assert!(RECORDING_OVERLAY_WIDTH <= 400.0);
        assert!(RECORDING_OVERLAY_HEIGHT >= 50.0);
        assert!(RECORDING_OVERLAY_HEIGHT <= 200.0);
    }

    #[test]
    fn test_recording_overlay_shown_payload_serializes() {
        let payload = RecordingOverlayShownPayload { show_call_ms: 42 };
        let json = serde_json::to_string(&payload).expect("Should serialize");
        assert!(json.contains("42"));
        assert!(json.contains("show_call_ms"));
    }
}
