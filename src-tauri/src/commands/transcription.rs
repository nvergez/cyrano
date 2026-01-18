//! Tauri commands for model management.
//!
//! Thin command handlers that delegate to transcription_service.

use crate::domain::CyranoError;
use crate::services::transcription_service::{self, ModelStatus};

/// Check the current model status.
///
/// Returns whether the model is loaded and its path if available.
#[tauri::command]
#[specta::specta]
pub fn check_model_status() -> ModelStatus {
    transcription_service::get_model_status()
}

/// Get the expected model directory path.
///
/// Returns the path where the model should be located (~/.cyrano/models/).
#[tauri::command]
#[specta::specta]
pub fn get_model_directory() -> Result<String, CyranoError> {
    transcription_service::get_models_directory().map(|p| p.display().to_string())
}

/// Open the model directory in Finder.
#[tauri::command]
#[specta::specta]
pub fn open_model_directory() -> Result<(), CyranoError> {
    let models_dir = transcription_service::get_models_directory()?;

    // Create directory if it doesn't exist
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir).map_err(|e| CyranoError::ModelNotFound {
            path: format!("Failed to create directory: {e}"),
        })?;
    }

    // Open in Finder (macOS specific)
    std::process::Command::new("open")
        .arg(&models_dir)
        .spawn()
        .map_err(|e| CyranoError::ModelNotFound {
            path: format!("Failed to open Finder: {e}"),
        })?;

    Ok(())
}

/// Request cancellation of ongoing transcription.
///
/// This sets a flag that will abort transcription before it starts or
/// at the earliest opportunity. Note that once whisper processing begins,
/// it will complete before checking the flag.
///
/// # Returns
/// * Always returns Ok(()) - cancellation request is always accepted
#[tauri::command]
#[specta::specta]
pub fn cancel_transcription() {
    log::info!("cancel_transcription command called");
    transcription_service::request_cancellation();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_model_status_returns_valid_struct() {
        let status = check_model_status();
        // Should return a valid struct (may or may not have model loaded)
        // Just verify it doesn't panic
        let _ = status.loaded;
        let _ = status.path;
    }

    #[test]
    fn test_get_model_directory_returns_path() {
        let result = get_model_directory();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.contains(".cyrano"));
        assert!(path.contains("models"));
    }
}
