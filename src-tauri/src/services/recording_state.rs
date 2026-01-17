//! Recording state management.
//!
//! This provides a minimal in-memory state holder for the recording workflow.
//! The actual audio capture and buffer management is handled by recording_service.rs.

use std::sync::{Mutex, OnceLock};

use crate::domain::RecordingState;

static RECORDING_STATE: OnceLock<Mutex<RecordingState>> = OnceLock::new();

fn recording_state() -> &'static Mutex<RecordingState> {
    RECORDING_STATE.get_or_init(|| Mutex::new(RecordingState::Idle))
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
