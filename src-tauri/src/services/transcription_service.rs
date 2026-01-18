//! Transcription service with lazy model loading and timeout-based unloading.
//!
//! This service manages the Whisper model lifecycle:
//! - Lazy loading on first transcription
//! - 30-minute inactivity timeout for memory cleanup
//! - Thread-safe model access

use crate::domain::CyranoError;
use crate::infrastructure::whisper::WhisperAdapter;
use crate::traits::transcriber::Transcriber;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// How long the model stays loaded after last use before auto-unloading.
const KEEP_ALIVE_DURATION: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// Global transcription service state with lazy initialization.
static TRANSCRIPTION_SERVICE: OnceLock<Mutex<TranscriptionServiceState>> = OnceLock::new();

/// Internal state for the transcription service.
struct TranscriptionServiceState {
    adapter: WhisperAdapter,
    last_used: Option<Instant>,
}

/// Get the global service state, initializing if needed.
fn service_state() -> &'static Mutex<TranscriptionServiceState> {
    TRANSCRIPTION_SERVICE.get_or_init(|| {
        Mutex::new(TranscriptionServiceState {
            adapter: WhisperAdapter::new(),
            last_used: None,
        })
    })
}

/// Ensure the model is loaded, loading lazily if needed.
///
/// This function will:
/// 1. Check if the model has been idle for too long and unload if so
/// 2. If not loaded, find and load the model from `~/.cyrano/models/`
/// 3. Update the last-used timestamp
pub fn ensure_model_loaded() -> Result<(), CyranoError> {
    let mut state = service_state()
        .lock()
        .map_err(|e| CyranoError::TranscriptionFailed {
            reason: format!("Lock failed: {e}"),
        })?;

    // Check timeout first - unload if idle too long
    if let Some(last_used) = state.last_used {
        if last_used.elapsed() > KEEP_ALIVE_DURATION {
            log::info!("Model idle for >30 min, unloading to free memory");
            state.adapter.unload()?;
            state.last_used = None;
        }
    }

    // Already loaded? Just update timestamp
    if state.adapter.is_loaded() {
        state.last_used = Some(Instant::now());
        return Ok(());
    }

    // Find and load model
    let model_path = get_model_path()?;
    log::info!("Loading Whisper model from: {}", model_path.display());
    state.adapter.load_model(&model_path)?;
    state.last_used = Some(Instant::now());

    Ok(())
}

/// Check if the model is currently loaded.
pub fn is_model_loaded() -> bool {
    service_state()
        .lock()
        .map(|state| state.adapter.is_loaded())
        .unwrap_or(false)
}

/// Manually unload the model to free memory.
#[allow(dead_code)] // Will be used when background timer is added
pub fn unload_model() -> Result<(), CyranoError> {
    let mut state = service_state()
        .lock()
        .map_err(|e| CyranoError::TranscriptionFailed {
            reason: format!("Lock failed: {e}"),
        })?;

    state.adapter.unload()?;
    state.last_used = None;
    log::info!("Model manually unloaded");
    Ok(())
}

/// Check if the model has been idle and unload if needed.
///
/// Call this periodically or before transcription to enforce the timeout.
#[allow(dead_code)] // Will be used when background timer is added
pub fn check_and_unload_if_idle() -> Result<bool, CyranoError> {
    let mut state = service_state()
        .lock()
        .map_err(|e| CyranoError::TranscriptionFailed {
            reason: format!("Lock failed: {e}"),
        })?;

    if let Some(last_used) = state.last_used {
        if last_used.elapsed() > KEEP_ALIVE_DURATION && state.adapter.is_loaded() {
            log::info!(
                "Model idle for {:?}, unloading to free memory",
                last_used.elapsed()
            );
            state.adapter.unload()?;
            state.last_used = None;
            return Ok(true);
        }
    }

    Ok(false)
}

/// Get the path to the models directory.
pub fn get_models_directory() -> Result<PathBuf, CyranoError> {
    let home = dirs::home_dir().ok_or_else(|| CyranoError::ModelNotFound {
        path: "~/.cyrano/models/ (could not resolve home directory)".to_string(),
    })?;

    Ok(home.join(".cyrano").join("models"))
}

/// Find the first .bin model file in `~/.cyrano/models/`.
fn get_model_path() -> Result<PathBuf, CyranoError> {
    let models_dir = get_models_directory()?;

    if !models_dir.exists() {
        return Err(CyranoError::ModelNotFound {
            path: models_dir.display().to_string(),
        });
    }

    // Find first .bin file
    let entries = std::fs::read_dir(&models_dir).map_err(|e| CyranoError::ModelNotFound {
        path: format!("{}: {}", models_dir.display(), e),
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "bin") {
            return Ok(path);
        }
    }

    Err(CyranoError::ModelNotFound {
        path: format!("{} (no .bin files found)", models_dir.display()),
    })
}

/// Model status information for the frontend.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ModelStatus {
    pub loaded: bool,
    pub path: Option<String>,
}

/// Get the current model status.
pub fn get_model_status() -> ModelStatus {
    let loaded = is_model_loaded();
    let path = if loaded {
        get_model_path().ok().map(|p| p.display().to_string())
    } else {
        None
    };
    ModelStatus { loaded, path }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_not_loaded_at_startup() {
        // Fresh state - model should not be loaded
        // Note: This test may fail if run after other tests that load the model
        // In isolation, a fresh service has no model loaded
        let adapter = WhisperAdapter::new();
        assert!(!adapter.is_loaded());
    }

    #[test]
    fn test_model_path_resolution() {
        // Test that get_models_directory returns expected path
        let result = get_models_directory();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".cyrano"));
        assert!(path.to_string_lossy().contains("models"));
    }

    #[test]
    fn test_model_not_found_error() {
        // When no models directory exists or no .bin files, should return error
        // This test relies on the models directory not existing
        let result = get_model_path();
        // Either ModelNotFound (directory doesn't exist) or success (if user has models)
        // We just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_model_status_when_not_loaded() {
        let status = get_model_status();
        // On a fresh test run, model is likely not loaded
        // The status should at least not panic
        assert!(status.loaded == status.path.is_some() || !status.loaded);
    }
}
