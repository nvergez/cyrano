//! Whisper-rs adapter for speech-to-text transcription.

use crate::domain::CyranoError;
use crate::traits::transcriber::Transcriber;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Adapter wrapping whisper-rs for speech-to-text transcription.
pub struct WhisperAdapter {
    context: Option<WhisperContext>,
}

impl WhisperAdapter {
    /// Create a new WhisperAdapter with no model loaded.
    pub fn new() -> Self {
        Self { context: None }
    }
}

impl Default for WhisperAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Transcriber for WhisperAdapter {
    fn load_model(&mut self, model_path: &Path) -> Result<(), CyranoError> {
        if !model_path.exists() {
            return Err(CyranoError::ModelNotFound {
                path: model_path.display().to_string(),
            });
        }

        let path_str = model_path
            .to_str()
            .ok_or_else(|| CyranoError::ModelLoadFailed {
                reason: "Invalid path encoding".to_string(),
            })?;

        let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
            .map_err(|e| CyranoError::ModelLoadFailed {
                reason: e.to_string(),
            })?;

        self.context = Some(ctx);
        log::info!("Whisper model loaded from: {}", model_path.display());
        Ok(())
    }

    fn transcribe(&self, samples: &[f32]) -> Result<String, CyranoError> {
        let ctx = self
            .context
            .as_ref()
            .ok_or_else(|| CyranoError::TranscriptionFailed {
                reason: "Model not loaded".to_string(),
            })?;

        let mut state = ctx
            .create_state()
            .map_err(|e| CyranoError::TranscriptionFailed {
                reason: format!("Failed to create state: {e}"),
            })?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, samples)
            .map_err(|e| CyranoError::TranscriptionFailed {
                reason: format!("Transcription failed: {e}"),
            })?;

        let num_segments =
            state
                .full_n_segments()
                .map_err(|e| CyranoError::TranscriptionFailed {
                    reason: format!("Failed to get segments: {e}"),
                })?;

        let mut result = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                result.push_str(&segment);
            }
        }

        Ok(result.trim().to_string())
    }

    fn is_loaded(&self) -> bool {
        self.context.is_some()
    }

    fn unload(&mut self) -> Result<(), CyranoError> {
        if self.context.is_some() {
            log::info!("Unloading Whisper model");
        }
        self.context = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_adapter_not_loaded_initially() {
        let adapter = WhisperAdapter::new();
        assert!(!adapter.is_loaded());
    }

    #[test]
    fn test_model_not_found_error() {
        let mut adapter = WhisperAdapter::new();
        let fake_path = PathBuf::from("/nonexistent/model.bin");
        let result = adapter.load_model(&fake_path);
        assert!(result.is_err());
        if let Err(CyranoError::ModelNotFound { path }) = result {
            assert!(path.contains("nonexistent"));
        } else {
            panic!("Expected ModelNotFound error");
        }
    }

    #[test]
    fn test_transcribe_without_model_fails() {
        let adapter = WhisperAdapter::new();
        let samples = vec![0.0f32; 16000];
        let result = adapter.transcribe(&samples);
        assert!(result.is_err());
        if let Err(CyranoError::TranscriptionFailed { reason }) = result {
            assert!(reason.contains("not loaded"));
        } else {
            panic!("Expected TranscriptionFailed error");
        }
    }

    #[test]
    fn test_unload_when_no_model() {
        let mut adapter = WhisperAdapter::new();
        let result = adapter.unload();
        assert!(result.is_ok());
        assert!(!adapter.is_loaded());
    }
}
