//! cpal audio constants and error conversions.
//!
//! Provides constants and error type conversions for cpal-based audio capture.
//! The actual audio capture is implemented in RecordingService using a dedicated thread.

use crate::domain::CyranoError;

/// Target sample rate for Whisper compatibility (16kHz)
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

// Error conversions from cpal errors to CyranoError

impl From<cpal::BuildStreamError> for CyranoError {
    fn from(e: cpal::BuildStreamError) -> Self {
        match e {
            cpal::BuildStreamError::DeviceNotAvailable => CyranoError::MicAccessDenied,
            cpal::BuildStreamError::StreamConfigNotSupported => CyranoError::RecordingFailed {
                reason: "Audio format not supported".to_string(),
            },
            _ => CyranoError::RecordingFailed {
                reason: e.to_string(),
            },
        }
    }
}

impl From<cpal::PlayStreamError> for CyranoError {
    fn from(e: cpal::PlayStreamError) -> Self {
        CyranoError::RecordingFailed {
            reason: e.to_string(),
        }
    }
}

impl From<cpal::DevicesError> for CyranoError {
    fn from(e: cpal::DevicesError) -> Self {
        CyranoError::RecordingFailed {
            reason: e.to_string(),
        }
    }
}

impl From<cpal::SupportedStreamConfigsError> for CyranoError {
    fn from(e: cpal::SupportedStreamConfigsError) -> Self {
        match e {
            cpal::SupportedStreamConfigsError::DeviceNotAvailable => CyranoError::MicAccessDenied,
            _ => CyranoError::RecordingFailed {
                reason: e.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_stream_error_conversion() {
        let err = cpal::BuildStreamError::DeviceNotAvailable;
        let cyrano_err: CyranoError = err.into();
        assert!(matches!(cyrano_err, CyranoError::MicAccessDenied));
    }

    #[test]
    fn test_supported_configs_error_conversion() {
        let err = cpal::SupportedStreamConfigsError::DeviceNotAvailable;
        let cyrano_err: CyranoError = err.into();
        assert!(matches!(cyrano_err, CyranoError::MicAccessDenied));
    }

    #[test]
    fn test_target_sample_rate() {
        assert_eq!(TARGET_SAMPLE_RATE, 16_000);
    }
}
