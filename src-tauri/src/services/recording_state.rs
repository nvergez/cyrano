//! Recording state management.
//!
//! This provides a minimal in-memory state holder for the recording workflow.
//! The actual audio capture and buffer management is handled by recording_service.rs.

use std::sync::{Mutex, OnceLock};

use crate::domain::RecordingState;

static RECORDING_STATE: OnceLock<Mutex<RecordingState>> = OnceLock::new();
static AUDIO_BUFFER: OnceLock<Mutex<Vec<f32>>> = OnceLock::new();

fn recording_state() -> &'static Mutex<RecordingState> {
    RECORDING_STATE.get_or_init(|| Mutex::new(RecordingState::Idle))
}

fn audio_buffer() -> &'static Mutex<Vec<f32>> {
    AUDIO_BUFFER.get_or_init(|| Mutex::new(Vec::new()))
}

/// Set the current recording state.
pub fn set_recording_state(state: RecordingState) {
    match recording_state().lock() {
        Ok(mut guard) => {
            *guard = state;
        }
        Err(err) => {
            log::error!("Failed to lock recording state mutex: {err}");
        }
    }
}

/// Replace the global audio buffer with new samples.
pub fn set_audio_samples(samples: &[f32]) -> Result<(), String> {
    let mut buffer = audio_buffer()
        .lock()
        .map_err(|e| format!("Failed to lock audio buffer: {e}"))?;
    buffer.clear();
    buffer.extend_from_slice(samples);
    Ok(())
}

/// Take and clear the global audio buffer.
#[allow(dead_code)]
pub fn take_audio_samples() -> Result<Vec<f32>, String> {
    let mut buffer = audio_buffer()
        .lock()
        .map_err(|e| format!("Failed to lock audio buffer: {e}"))?;
    Ok(std::mem::take(&mut *buffer))
}

/// Clear the global audio buffer without returning it.
pub fn clear_audio_buffer() -> Result<(), String> {
    let mut buffer = audio_buffer()
        .lock()
        .map_err(|e| format!("Failed to lock audio buffer: {e}"))?;
    buffer.clear();
    Ok(())
}

#[cfg(test)]
pub fn get_recording_state() -> RecordingState {
    *recording_state()
        .lock()
        .expect("recording_state lock should succeed in tests")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_buffer_set_and_take() {
        // Clear any existing buffer from other tests
        let _ = clear_audio_buffer();

        let samples = vec![0.1_f32, 0.2_f32, 0.3_f32];
        set_audio_samples(&samples).expect("set_audio_samples should succeed");
        let taken = take_audio_samples().expect("take_audio_samples should succeed");
        assert_eq!(taken, samples);
        let empty = take_audio_samples().expect("take_audio_samples should succeed");
        assert!(empty.is_empty());
    }
}
