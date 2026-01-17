//! Recording orchestration service.
//!
//! Manages the recording workflow including permission checks, audio capture,
//! and state transitions. Uses a dedicated thread for audio capture to handle
//! cpal's Stream type not being Send-safe.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};

use crate::domain::{CyranoError, PermissionStatus, RecordingState};
use crate::infrastructure::audio::cpal_adapter::CpalAdapter;
use crate::services::permission_service;
use crate::services::recording_state;
use crate::traits::audio_capture::AudioCapture;

/// Payload for the recording-started event.
#[derive(Clone, serde::Serialize)]
pub struct RecordingStartedPayload {
    /// Unix timestamp in milliseconds when recording started
    pub timestamp: u64,
}

/// Payload for the recording-stopped event.
#[derive(Clone, serde::Serialize, specta::Type)]
pub struct RecordingStoppedPayload {
    /// Duration of the recording in milliseconds (max ~49 days)
    pub duration_ms: u32,
    /// Number of audio samples captured
    pub sample_count: u32,
}

/// Payload for the recording-failed event.
#[derive(Clone, serde::Serialize)]
pub struct RecordingFailedPayload {
    /// Error that caused the recording to fail
    pub error: CyranoError,
}

/// Global recording state - holds the audio capture thread and buffer
struct RecordingContext {
    /// Flag to signal recording should stop
    stop_flag: Arc<AtomicBool>,
    /// Handle to the capture thread
    capture_thread: Option<JoinHandle<Result<Vec<f32>, CyranoError>>>,
    /// Timestamp when recording started
    start_timestamp: u64,
}

static RECORDING_CONTEXT: std::sync::OnceLock<Mutex<Option<RecordingContext>>> =
    std::sync::OnceLock::new();

fn recording_context() -> &'static Mutex<Option<RecordingContext>> {
    RECORDING_CONTEXT.get_or_init(|| Mutex::new(None))
}

/// Get the current Unix timestamp in milliseconds.
fn get_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Start recording audio from the microphone.
///
/// This function:
/// 1. Checks microphone permission
/// 2. Spawns a dedicated thread for audio capture
/// 3. Updates recording state to Recording
/// 4. Emits recording-started event
///
/// # Arguments
/// * `app` - The Tauri application handle for emitting events
///
/// # Returns
/// * `Ok(())` if recording started successfully
/// * `Err(CyranoError::MicAccessDenied)` if permission is denied
/// * `Err(CyranoError::RecordingFailed)` for other errors
pub fn start_recording(app: &AppHandle) -> Result<(), CyranoError> {
    // Check permission first
    let permission = permission_service::check_microphone_permission();
    if permission == PermissionStatus::Denied {
        log::warn!("Microphone permission denied");
        // Note: recording-failed event is emitted by the caller (shortcut_service)
        // AFTER showing the overlay, so the overlay window can receive it
        return Err(CyranoError::MicAccessDenied);
    }

    // Lock the context
    let mut ctx_guard = recording_context()
        .lock()
        .map_err(|e| CyranoError::RecordingFailed {
            reason: format!("Failed to lock recording context: {e}"),
        })?;

    // Check if already recording
    if ctx_guard.is_some() {
        log::warn!("Recording already in progress");
        return Ok(());
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let start_timestamp = get_timestamp_ms();

    let stop_flag_clone = stop_flag.clone();

    // Spawn audio capture thread
    let capture_thread = thread::spawn(move || -> Result<Vec<f32>, CyranoError> {
        run_audio_capture(stop_flag_clone)
    });

    *ctx_guard = Some(RecordingContext {
        stop_flag,
        capture_thread: Some(capture_thread),
        start_timestamp,
    });

    // Update state
    recording_state::set_recording_state(RecordingState::Recording);

    // Emit event
    let payload = RecordingStartedPayload {
        timestamp: start_timestamp,
    };
    if let Err(e) = app.emit("recording-started", payload) {
        log::error!("Failed to emit recording-started event: {e}");
    }

    log::info!("Recording started at timestamp {start_timestamp}");
    Ok(())
}

/// Stop recording and return the captured audio samples.
///
/// This function:
/// 1. Signals the capture thread to stop
/// 2. Waits for the thread to finish
/// 3. Returns the captured audio samples
/// 4. Updates recording state to Transcribing
/// 5. Emits recording-stopped event
///
/// # Arguments
/// * `app` - The Tauri application handle for emitting events
///
/// # Returns
/// * `Ok(RecordingStoppedPayload)` with recording info
/// * `Err(CyranoError::RecordingFailed)` if no recording was in progress
pub fn stop_recording(app: &AppHandle) -> Result<RecordingStoppedPayload, CyranoError> {
    let mut ctx_guard = recording_context()
        .lock()
        .map_err(|e| CyranoError::RecordingFailed {
            reason: format!("Failed to lock recording context: {e}"),
        })?;

    let ctx = ctx_guard.take().ok_or(CyranoError::RecordingFailed {
        reason: "No recording in progress".to_string(),
    })?;

    // Signal the capture thread to stop
    ctx.stop_flag.store(true, Ordering::SeqCst);

    // Wait for the capture thread to finish
    let samples = if let Some(handle) = ctx.capture_thread {
        match handle.join() {
            Ok(Ok(samples)) => {
                log::debug!("Audio capture thread finished successfully");
                samples
            }
            Ok(Err(e)) => {
                log::warn!("Audio capture thread returned error: {e}");
                Vec::new()
            }
            Err(_) => {
                log::error!("Audio capture thread panicked");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Store samples in the global audio buffer for later use
    if let Err(e) = store_audio_samples(&samples) {
        log::error!("Failed to store audio samples: {e}");
    }

    let stop_timestamp = get_timestamp_ms();
    let duration_ms = stop_timestamp.saturating_sub(ctx.start_timestamp) as u32;
    let sample_count = samples.len() as u32;

    // Update state
    recording_state::set_recording_state(RecordingState::Transcribing);

    let payload = RecordingStoppedPayload {
        duration_ms,
        sample_count,
    };

    // Emit event
    if let Err(e) = app.emit("recording-stopped", payload.clone()) {
        log::error!("Failed to emit recording-stopped event: {e}");
    }

    log::info!(
        "Recording stopped: {} samples, {}ms duration",
        sample_count,
        duration_ms
    );
    Ok(payload)
}

/// Store audio samples in the global buffer for transcription.
fn store_audio_samples(samples: &[f32]) -> Result<(), CyranoError> {
    recording_state::set_audio_samples(samples)
        .map_err(|e| CyranoError::RecordingFailed { reason: e })
}

/// Cancel the current recording, discarding all captured audio.
///
/// This function stops the audio capture thread and discards all samples.
/// Used when the user clicks on the overlay to cancel.
///
/// # Returns
/// The number of samples that were discarded.
pub fn cancel_recording() -> usize {
    let mut ctx_guard = match recording_context().lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock recording context for cancel: {e}");
            return 0;
        }
    };

    let ctx = match ctx_guard.take() {
        Some(ctx) => ctx,
        None => {
            log::debug!("No recording in progress to cancel");
            return 0;
        }
    };

    // Signal the capture thread to stop
    ctx.stop_flag.store(true, Ordering::SeqCst);

    // Wait for the capture thread to finish
    let sample_count = if let Some(handle) = ctx.capture_thread {
        match handle.join() {
            Ok(Ok(samples)) => {
                log::debug!("Audio capture thread finished on cancel");
                samples.len()
            }
            Ok(Err(e)) => {
                log::warn!("Audio capture thread error on cancel: {e}");
                0
            }
            Err(_) => {
                log::error!("Audio capture thread panicked on cancel");
                0
            }
        }
    } else {
        0
    };

    // Update state back to idle
    recording_state::set_recording_state(RecordingState::Idle);
    if let Err(e) = recording_state::clear_audio_buffer() {
        log::warn!("Failed to clear audio buffer on cancel: {e}");
    }

    log::info!("Recording cancelled, discarded {} samples", sample_count);
    sample_count
}

/// Run audio capture in a dedicated thread.
///
/// This function handles the actual cpal audio capture, running until
/// the stop_flag is set to true.
fn run_audio_capture(stop_flag: Arc<AtomicBool>) -> Result<Vec<f32>, CyranoError> {
    let mut capture: Box<dyn AudioCapture> = Box::new(CpalAdapter::new());
    capture.start_capture()?;

    log::info!("Audio capture started in dedicated thread");

    // Keep the stream alive until stop is signaled
    while !stop_flag.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(10));
    }

    log::info!("Audio capture stopping");
    capture.stop_capture()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_timestamp_ms_returns_reasonable_value() {
        let ts = get_timestamp_ms();
        // Should be after Jan 2020
        let jan_2020_ms: u64 = 1577836800000;
        assert!(ts > jan_2020_ms, "Timestamp should be after January 2020");
    }

    #[test]
    fn test_recording_started_payload_serializes() {
        let payload = RecordingStartedPayload {
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&payload).expect("Should serialize");
        assert!(json.contains("1234567890"));
    }

    #[test]
    fn test_recording_stopped_payload_serializes() {
        let payload = RecordingStoppedPayload {
            duration_ms: 5000u32,
            sample_count: 80000u32,
        };
        let json = serde_json::to_string(&payload).expect("Should serialize");
        assert!(json.contains("5000"));
        assert!(json.contains("80000"));
    }

    #[test]
    fn test_store_audio_samples_writes_to_buffer() {
        let samples = vec![0.1_f32, 0.2_f32, 0.3_f32];
        store_audio_samples(&samples).expect("store_audio_samples should succeed");
        let stored =
            recording_state::take_audio_samples().expect("take_audio_samples should succeed");
        assert_eq!(stored, samples);
    }

    #[test]
    fn test_cancel_recording_resets_state() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let handle = thread::spawn(move || {
            while !stop_flag_clone.load(Ordering::SeqCst) {
                thread::sleep(std::time::Duration::from_millis(1));
            }
            Ok(vec![0.0_f32; 10])
        });

        let ctx = RecordingContext {
            stop_flag,
            capture_thread: Some(handle),
            start_timestamp: 0,
        };

        *recording_context()
            .lock()
            .expect("recording context lock should succeed") = Some(ctx);

        recording_state::set_recording_state(RecordingState::Recording);
        let discarded = cancel_recording();

        assert_eq!(discarded, 10);
        assert_eq!(recording_state::get_recording_state(), RecordingState::Idle);
    }
}
