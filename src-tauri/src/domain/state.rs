//! Recording state enums.

use serde::{Deserialize, Serialize};
use specta::Type;

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

/// Represents the microphone permission status on macOS.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum PermissionStatus {
    /// Permission has been granted by the user.
    Granted,
    /// Permission has been explicitly denied by the user.
    Denied,
    /// Permission has not yet been requested (first launch).
    #[default]
    NotDetermined,
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

    #[test]
    fn test_default_permission_status_is_not_determined() {
        assert_eq!(PermissionStatus::default(), PermissionStatus::NotDetermined);
    }

    #[test]
    fn test_permission_status_serialization() {
        let status = PermissionStatus::Granted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Granted\"");
    }

    #[test]
    fn test_permission_status_deserialization() {
        let status: PermissionStatus = serde_json::from_str("\"Denied\"").unwrap();
        assert_eq!(status, PermissionStatus::Denied);
    }
}
