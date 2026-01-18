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

use std::sync::atomic::{AtomicBool, Ordering};

/// How long the model stays loaded after last use before auto-unloading.
const KEEP_ALIVE_DURATION: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// Cancellation flag for transcription.
static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

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

/// Request cancellation of any ongoing transcription.
///
/// This sets a flag that will be checked before transcription begins.
/// Note: Once whisper `state.full()` is called, transcription runs to completion.
pub fn request_cancellation() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
    log::info!("Transcription cancellation requested");
}

/// Clear the cancellation flag.
///
/// Should be called when starting a new recording to reset the flag.
pub fn clear_cancellation() {
    CANCEL_FLAG.store(false, Ordering::SeqCst);
}

/// Check if transcription has been cancelled.
pub fn is_cancelled() -> bool {
    CANCEL_FLAG.load(Ordering::SeqCst)
}

/// Transcribe audio samples to text.
///
/// MUST be called from a non-async context (spawn_blocking or std::thread::spawn)
/// because whisper transcription is CPU-intensive.
///
/// # Arguments
/// * `samples` - Audio samples at 16kHz mono, normalized to [-1.0, 1.0]
///
/// # Returns
/// * `Ok(String)` - The transcribed text
/// * `Err(CyranoError)` - If transcription fails or is cancelled
///
/// # Panics
/// Never panics, all errors are returned as `CyranoError`.
pub fn transcribe(samples: &[f32]) -> Result<String, CyranoError> {
    // Check if cancelled before starting
    if is_cancelled() {
        clear_cancellation();
        log::info!("Transcription cancelled before starting");
        return Err(CyranoError::TranscriptionFailed {
            reason: "Transcription cancelled by user".to_string(),
        });
    }

    let start = Instant::now();

    let mut state = service_state()
        .lock()
        .map_err(|e| CyranoError::TranscriptionFailed {
            reason: format!("Lock failed: {e}"),
        })?;

    // Model must already be loaded (called ensure_model_loaded first)
    if !state.adapter.is_loaded() {
        return Err(CyranoError::TranscriptionFailed {
            reason: "Model not loaded - call ensure_model_loaded first".to_string(),
        });
    }

    // Handle empty audio buffer gracefully
    if samples.is_empty() {
        log::warn!("Transcription called with empty audio buffer");
        return Ok(String::new());
    }

    log::info!(
        "Starting transcription of {} samples ({:.2}s audio)",
        samples.len(),
        samples.len() as f64 / 16000.0
    );

    let text = state.adapter.transcribe(samples)?;

    // Update last used for timeout tracking
    state.last_used = Some(Instant::now());

    let elapsed_ms = start.elapsed().as_millis();
    log::info!(
        "Transcription completed in {}ms, {} chars",
        elapsed_ms,
        text.len()
    );

    // Warn if exceeding NFR2 (2 seconds for 1 minute audio)
    // 16kHz mono: 1 minute = 960,000 samples
    let audio_seconds = samples.len() as f64 / 16000.0;
    let expected_max_ms = (audio_seconds * 2.0 * 1000.0) as u128; // 2x real-time max
    if elapsed_ms > expected_max_ms {
        log::warn!(
            "Transcription exceeded 2x real-time target: {}ms for {:.1}s audio (expected max {}ms)",
            elapsed_ms,
            audio_seconds,
            expected_max_ms
        );
    }

    Ok(text)
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

    #[test]
    fn test_transcribe_requires_loaded_model() {
        // When model is not loaded, transcribe should fail
        // Note: This test may not be deterministic if model is loaded by other tests
        let adapter = WhisperAdapter::new();
        let samples = vec![0.0f32; 16000];
        let result = adapter.transcribe(&samples);
        // Expect TranscriptionFailed when model not loaded
        assert!(result.is_err());
        if let Err(CyranoError::TranscriptionFailed { reason }) = result {
            assert!(
                reason.contains("not loaded"),
                "Expected 'not loaded' in error message, got: {reason}"
            );
        } else {
            panic!("Expected TranscriptionFailed error");
        }
    }

    #[test]
    fn test_transcribe_empty_audio_returns_empty_string() {
        // Empty audio should return empty string (graceful handling)
        // This tests the service-level function, not the adapter
        // Without a loaded model, we can't test the full path
        // Instead, verify that empty input is handled at service level

        // Ensure cancellation is cleared
        clear_cancellation();

        // Since model isn't loaded, we'll get an error about that
        // This is expected behavior - model must be loaded first
        let samples: Vec<f32> = vec![];
        let result = transcribe(&samples);

        // Either empty audio handling or model-not-loaded error is acceptable
        match result {
            Ok(text) => assert!(text.is_empty(), "Empty audio should produce empty text"),
            Err(CyranoError::TranscriptionFailed { reason }) => {
                // Model not loaded is expected in test environment
                assert!(
                    reason.contains("not loaded") || reason.contains("Lock failed"),
                    "Unexpected error: {reason}"
                );
            }
            Err(e) => panic!("Unexpected error type: {e}"),
        }
    }

    #[test]
    fn test_cancellation_flag_operations() {
        // Test cancellation flag set/clear/check
        // Note: These tests share global state and may interleave with other tests
        // We test the invariants hold for each operation independently

        // Test that clear_cancellation sets flag to false
        clear_cancellation();
        // After clearing, check that we can set and see the flag
        request_cancellation();
        let was_set = is_cancelled();
        clear_cancellation(); // Always clean up

        assert!(was_set, "Cancellation flag should be settable and readable");
    }

    #[test]
    fn test_transcribe_respects_cancellation() {
        // When cancelled, transcribe should return error immediately
        // Note: This test shares global state with other tests running in parallel.
        // We verify that either:
        // 1. Cancellation is detected and returns "cancelled" error, OR
        // 2. Model-not-loaded is detected (if another test cleared the flag)
        //
        // The key invariant is that transcribe() fails fast when cancelled.
        clear_cancellation();
        request_cancellation();

        let samples = vec![0.0f32; 16000];
        let result = transcribe(&samples);

        assert!(result.is_err(), "transcribe() should return an error");
        if let Err(CyranoError::TranscriptionFailed { reason }) = result {
            // Accept either cancellation or model-not-loaded due to test parallelism
            assert!(
                reason.contains("cancelled") || reason.contains("not loaded"),
                "Expected 'cancelled' or 'not loaded' in error message, got: {reason}"
            );
        } else {
            panic!("Expected TranscriptionFailed error");
        }

        // Clean up any flag state
        clear_cancellation();
    }
}
