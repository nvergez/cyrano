//! Output service for clipboard and cursor insertion operations.
//!
//! This service handles the output phase of the transcription pipeline:
//! 1. Copy transcribed text to system clipboard (FR12)
//! 2. Future: Insert text at cursor position (FR13, Epic 3)

use crate::domain::CyranoError;
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
}
