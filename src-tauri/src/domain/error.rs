//! Application error types.

use serde::Serialize;
use specta::Type;
use thiserror::Error;

/// Unified error type for all Cyrano operations.
#[derive(Debug, Clone, Serialize, Type, Error)]
pub enum CyranoError {
    /// User has not granted microphone access permission.
    #[error("Microphone access denied")]
    MicAccessDenied,

    /// The Whisper model file was not found at the expected location.
    #[error("Model not found at {path}")]
    ModelNotFound { path: String },

    /// Failed to load the Whisper model into memory.
    #[error("Model loading failed: {reason}")]
    ModelLoadFailed { reason: String },

    /// The transcription process failed.
    #[error("Transcription failed: {reason}")]
    TranscriptionFailed { reason: String },

    /// Audio recording failed.
    #[error("Recording failed: {reason}")]
    RecordingFailed { reason: String },

    /// Clipboard operation failed.
    #[error("Clipboard operation failed: {reason}")]
    ClipboardFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mic_access_denied_message() {
        let err = CyranoError::MicAccessDenied;
        assert_eq!(err.to_string(), "Microphone access denied");
    }

    #[test]
    fn test_model_not_found_message() {
        let err = CyranoError::ModelNotFound {
            path: "/path/to/model".to_string(),
        };
        assert_eq!(err.to_string(), "Model not found at /path/to/model");
    }

    #[test]
    fn test_model_load_failed_message() {
        let err = CyranoError::ModelLoadFailed {
            reason: "out of memory".to_string(),
        };
        assert_eq!(err.to_string(), "Model loading failed: out of memory");
    }

    #[test]
    fn test_transcription_failed_message() {
        let err = CyranoError::TranscriptionFailed {
            reason: "invalid audio format".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Transcription failed: invalid audio format"
        );
    }

    #[test]
    fn test_recording_failed_message() {
        let err = CyranoError::RecordingFailed {
            reason: "device disconnected".to_string(),
        };
        assert_eq!(err.to_string(), "Recording failed: device disconnected");
    }

    #[test]
    fn test_clipboard_failed_message() {
        let err = CyranoError::ClipboardFailed {
            reason: "access denied".to_string(),
        };
        assert_eq!(err.to_string(), "Clipboard operation failed: access denied");
    }

    #[test]
    fn test_error_serialization() {
        let err = CyranoError::MicAccessDenied;
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"MicAccessDenied\"");
    }

    #[test]
    fn test_error_with_fields_serialization() {
        let err = CyranoError::ModelNotFound {
            path: "/test/path".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("ModelNotFound"));
        assert!(json.contains("/test/path"));
    }
}
