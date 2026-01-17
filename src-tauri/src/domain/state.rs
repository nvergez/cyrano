//! Recording state enum.

use serde::{Deserialize, Serialize};

/// Represents the current state of the recording/transcription workflow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingState {
    /// No recording in progress, ready to start.
    #[default]
    Idle,
    /// Currently capturing audio from microphone.
    Recording,
    /// Audio captured, transcription in progress.
    Transcribing,
    /// Transcription complete, result available.
    Done,
    /// An error occurred during recording or transcription.
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_idle() {
        assert_eq!(RecordingState::default(), RecordingState::Idle);
    }

    #[test]
    fn test_state_serialization() {
        let state = RecordingState::Recording;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"Recording\"");
    }

    #[test]
    fn test_state_deserialization() {
        let state: RecordingState = serde_json::from_str("\"Transcribing\"").unwrap();
        assert_eq!(state, RecordingState::Transcribing);
    }
}
