//! Recording state and buffer management.
//!
//! This provides a minimal in-memory state holder for the recording workflow.
//! Audio capture is implemented in Story 1.4; this module keeps a placeholder
//! buffer so cancel can clear partial data.

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

/// Clear any buffered audio samples.
/// Returns the number of samples cleared.
pub fn clear_audio_buffer() -> usize {
    match audio_buffer().lock() {
        Ok(mut buffer) => {
            let len = buffer.len();
            buffer.clear();
            len
        }
        Err(err) => {
            log::error!("Failed to lock audio buffer mutex: {err}");
            0
        }
    }
}

/// Cancel the current recording: set state to Idle and clear buffer.
pub fn cancel_recording() -> usize {
    set_recording_state(RecordingState::Idle);
    clear_audio_buffer()
}
