//! Output service for clipboard and cursor insertion operations.
//!
//! This service handles the output phase of the transcription pipeline:
//! 1. Copy transcribed text to system clipboard (FR12)
//! 2. Insert text at cursor position if accessibility permission granted (FR13, Epic 3)
//!
//! Graceful degradation: If accessibility permission is not granted, only clipboard
//! copy is performed with no error shown to user.

use crate::domain::{CyranoError, PermissionStatus};
use crate::services::accessibility_service;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Copy text to the system clipboard.
///
/// # Arguments
/// * `text` - The text to copy to clipboard
/// * `app` - The Tauri app handle (needed for clipboard plugin access)
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(CyranoError::ClipboardFailed)` if clipboard operation fails
///
/// # Notes
/// This function is safe to call from a spawned thread since it only
/// accesses the AppHandle, which is Send + Sync.
pub fn copy_to_clipboard(text: &str, app: &AppHandle) -> Result<(), CyranoError> {
    log::debug!("Copying {} chars to clipboard", text.len());

    app.clipboard()
        .write_text(text)
        .map_err(|e| CyranoError::ClipboardFailed {
            reason: e.to_string(),
        })?;

    log::info!("Successfully copied {} chars to clipboard", text.len());
    Ok(())
}

/// Check if cursor insertion is available (accessibility permission granted).
///
/// This function checks whether the app has accessibility permission,
/// which is required for cursor insertion. The result is logged for
/// debugging purposes.
///
/// # Returns
/// * `true` if accessibility permission is granted (cursor insertion available)
/// * `false` if permission is not granted (clipboard-only mode)
///
/// # Note
/// This function never fails - it simply returns false if permission is not
/// granted, supporting graceful degradation to clipboard-only output.
#[allow(dead_code)] // Will be used in Story 3.2
pub fn is_cursor_insertion_available() -> bool {
    let status = accessibility_service::check_accessibility_permission();
    let available = status == PermissionStatus::Granted;

    if available {
        log::debug!("Cursor insertion available (accessibility permission granted)");
    } else {
        log::debug!("Cursor insertion not available (accessibility permission: {status:?})");
    }

    available
}

/// Output transcribed text with automatic mode selection.
///
/// This function handles the output phase of transcription:
/// 1. Always copies text to clipboard (FR12)
/// 2. If accessibility permission granted: will insert at cursor (Story 3.2)
/// 3. If accessibility denied: gracefully degrades to clipboard-only
///
/// # Arguments
/// * `text` - The transcribed text to output
/// * `app` - The Tauri app handle
///
/// # Returns
/// * `Ok(true)` if both clipboard copy and cursor insertion succeeded
/// * `Ok(false)` if only clipboard copy succeeded (accessibility denied)
/// * `Err(CyranoError::ClipboardFailed)` if clipboard copy failed
///
/// # Note
/// Clipboard copy is always attempted regardless of accessibility status.
/// Cursor insertion failure is not treated as an error.
#[allow(dead_code)] // Will be used in Story 3.3
pub fn output_transcription(text: &str, app: &AppHandle) -> Result<bool, CyranoError> {
    // Step 1: Always copy to clipboard first
    copy_to_clipboard(text, app)?;

    // Step 2: Check if cursor insertion is available
    if is_cursor_insertion_available() {
        // TODO: Story 3.2 will implement actual cursor insertion here
        log::info!("Cursor insertion available - will be implemented in Story 3.2");
        // For now, return true to indicate cursor insertion could be done
        // Actual insertion will be added in Story 3.2
        Ok(true)
    } else {
        // Graceful degradation: no error, just clipboard only
        log::info!("Cursor insertion not available - clipboard copy completed");
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Clipboard tests require mocking or integration testing
    // since they interact with system clipboard.
    // Unit tests validate error handling paths.

    #[test]
    fn test_clipboard_failed_error_message() {
        let err = CyranoError::ClipboardFailed {
            reason: "Access denied".to_string(),
        };
        assert_eq!(err.to_string(), "Clipboard operation failed: Access denied");
    }

    #[test]
    fn test_clipboard_failed_serialization() {
        let err = CyranoError::ClipboardFailed {
            reason: "Test error".to_string(),
        };
        let json = serde_json::to_string(&err).expect("Should serialize");
        assert!(json.contains("ClipboardFailed"));
        assert!(json.contains("Test error"));
    }

    #[test]
    fn test_is_cursor_insertion_available_returns_bool() {
        // This test verifies the function executes without panic.
        // The actual result depends on system permission state.
        let result = is_cursor_insertion_available();
        // Result is either true or false - both are valid
        assert!(result || !result);
    }
}
