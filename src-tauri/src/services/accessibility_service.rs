//! Accessibility permission service.
//!
//! Provides business logic for checking and requesting macOS accessibility
//! permission, which is required for cursor insertion functionality.

use crate::domain::{CyranoError, PermissionStatus};

#[cfg(target_os = "macos")]
use crate::infrastructure::permissions::macos_accessibility;

/// Check the current accessibility permission status.
///
/// On macOS, this checks whether the app has been granted accessibility
/// permission in System Preferences > Privacy & Security > Accessibility.
///
/// # Returns
/// * `PermissionStatus::Granted` if permission is granted
/// * `PermissionStatus::NotDetermined` if permission is not granted
///
/// # Note
/// The macOS API cannot distinguish between "denied" and "not determined"
/// states - both return `false` from `AXIsProcessTrusted()`. We default
/// to `NotDetermined` for a safer UX (allows prompting).
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> PermissionStatus {
    if macos_accessibility::check_accessibility_trusted() {
        log::debug!("Accessibility permission granted");
        PermissionStatus::Granted
    } else {
        // Cannot distinguish Denied from NotDetermined with AXIsProcessTrusted
        // Default to NotDetermined for safer UX
        log::debug!("Accessibility permission not granted");
        PermissionStatus::NotDetermined
    }
}

/// Non-macOS stub: always returns Denied.
#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> PermissionStatus {
    log::warn!("Accessibility permission check is only supported on macOS");
    PermissionStatus::Denied
}

/// Request accessibility permission from the user.
///
/// On macOS, this triggers the system accessibility prompt if permission
/// has not yet been determined. If already granted or denied, no dialog
/// is shown.
///
/// # Returns
/// * `Ok(true)` if permission is granted
/// * `Ok(false)` if permission is not granted (graceful degradation)
///
/// # Note
/// Unlike microphone permission, we return `Ok(false)` instead of an error
/// when permission is denied. This supports graceful degradation - the app
/// continues to work with clipboard-only output.
#[cfg(target_os = "macos")]
pub fn request_accessibility_permission() -> Result<bool, CyranoError> {
    let granted = macos_accessibility::prompt_accessibility_permission();

    if granted {
        log::info!("Accessibility permission granted");
        Ok(true)
    } else {
        log::warn!("Accessibility permission not granted - falling back to clipboard only");
        // Return Ok(false) for graceful degradation, not an error
        Ok(false)
    }
}

/// Non-macOS stub: always returns Ok(false).
#[cfg(not(target_os = "macos"))]
pub fn request_accessibility_permission() -> Result<bool, CyranoError> {
    log::warn!("Accessibility permission request is only supported on macOS");
    Ok(false)
}

/// Open the Accessibility preferences pane in System Preferences.
///
/// This function opens System Preferences directly to the Privacy & Security >
/// Accessibility pane, making it easy for users to grant permission.
///
/// # Returns
/// * `Ok(())` if System Preferences was opened successfully
/// * `Err(CyranoError::OpenSettingsFailed)` if the command failed
#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> Result<(), CyranoError> {
    log::info!("Opening Accessibility settings in System Preferences");
    macos_accessibility::open_accessibility_preferences().map_err(|e| {
        log::error!("Failed to open Accessibility settings: {e}");
        CyranoError::OpenSettingsFailed {
            reason: format!("Failed to open Accessibility settings: {e}"),
        }
    })
}

/// Non-macOS stub: returns an error.
#[cfg(not(target_os = "macos"))]
pub fn open_accessibility_settings() -> Result<(), CyranoError> {
    log::warn!("Opening Accessibility settings is only supported on macOS");
    Err(CyranoError::OpenSettingsFailed {
        reason: "Accessibility settings are only available on macOS".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_accessibility_permission_returns_valid_status() {
        let status = check_accessibility_permission();
        assert!(matches!(
            status,
            PermissionStatus::Granted | PermissionStatus::Denied | PermissionStatus::NotDetermined
        ));
    }

    #[test]
    fn test_request_accessibility_permission_returns_result() {
        // This test verifies the function executes without panic.
        // The actual result depends on system state.
        let result = request_accessibility_permission();
        assert!(result.is_ok());
    }

    // Note: Cannot test open_accessibility_settings in unit tests
    // as it launches an external application.
}
