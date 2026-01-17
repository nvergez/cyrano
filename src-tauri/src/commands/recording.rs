//! Recording shortcut command handlers.
//!
//! Thin Tauri command wrappers that delegate to the shortcut service.
//! These commands expose recording shortcut functionality to the frontend.

use tauri::AppHandle;

use crate::domain::CyranoError;
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
