//! Recording command handlers.
//!
//! Thin Tauri command wrappers that delegate to recording services.
//! These commands expose recording functionality to the frontend.

use tauri::AppHandle;

use crate::domain::{CyranoError, PermissionStatus};
use crate::services::permission_service;
use crate::services::recording_service::{self, RecordingStoppedPayload};
use crate::services::shortcut_service::{self, DEFAULT_RECORDING_SHORTCUT};

/// Returns the default recording shortcut constant for frontend use.
#[tauri::command]
#[specta::specta]
pub fn get_default_recording_shortcut() -> String {
    DEFAULT_RECORDING_SHORTCUT.to_string()
}

/// Updates the global shortcut for recording.
/// Pass None to reset to default.
///
/// # Arguments
/// * `app` - The Tauri application handle
/// * `shortcut` - The new shortcut string, or None to use default
///
/// # Returns
/// * `Ok(())` if the shortcut was updated successfully
/// * `Err(String)` if the update failed
#[tauri::command]
#[specta::specta]
pub fn update_recording_shortcut(
    app: AppHandle,
    shortcut: Option<String>,
) -> Result<(), CyranoError> {
    #[cfg(desktop)]
    {
        let new_shortcut = shortcut.as_deref().unwrap_or(DEFAULT_RECORDING_SHORTCUT);
        log::info!("Updating recording shortcut to: {new_shortcut}");

        shortcut_service::register_recording_shortcut(&app, new_shortcut)?;

        log::info!("Recording shortcut updated successfully");
    }

    #[cfg(not(desktop))]
    {
        let _ = (app, shortcut);
        log::warn!("Global shortcuts not supported on this platform");
    }

    Ok(())
}

/// Starts audio recording from the microphone.
///
/// # Arguments
/// * `app` - The Tauri application handle
///
/// # Returns
/// * `Ok(())` if recording started successfully
/// * `Err(CyranoError::MicAccessDenied)` if microphone permission is denied
/// * `Err(CyranoError::RecordingFailed)` for other errors
#[tauri::command]
#[specta::specta]
pub fn start_recording(app: AppHandle) -> Result<(), CyranoError> {
    log::info!("start_recording command called");
    recording_service::start_recording(&app)
}

/// Stops audio recording and returns the recording information.
///
/// # Arguments
/// * `app` - The Tauri application handle
///
/// # Returns
/// * `Ok(RecordingStoppedPayload)` with duration and sample count
/// * `Err(CyranoError::RecordingFailed)` if no recording was in progress
#[tauri::command]
#[specta::specta]
pub fn stop_recording(app: AppHandle) -> Result<RecordingStoppedPayload, CyranoError> {
    log::info!("stop_recording command called");
    recording_service::stop_recording(&app)
}

/// Checks the current microphone permission status.
///
/// # Returns
/// * `PermissionStatus::Granted` if permission is granted
/// * `PermissionStatus::Denied` if permission is denied
/// * `PermissionStatus::NotDetermined` if not yet requested
#[tauri::command]
#[specta::specta]
pub fn check_microphone_permission() -> PermissionStatus {
    log::info!("check_microphone_permission command called");
    permission_service::check_microphone_permission()
}

/// Requests microphone permission from the user.
///
/// On macOS, this triggers the system permission dialog if not previously requested.
///
/// # Returns
/// * `Ok(true)` if permission was granted
/// * `Err(CyranoError::MicAccessDenied)` if permission was denied
#[tauri::command]
#[specta::specta]
pub fn request_microphone_permission() -> Result<bool, CyranoError> {
    log::info!("request_microphone_permission command called");
    permission_service::request_microphone_permission()
}
