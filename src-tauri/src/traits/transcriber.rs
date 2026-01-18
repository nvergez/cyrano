//! Transcription port (trait).
//!
//! Defines the interface that speech-to-text adapters must implement.

use crate::domain::CyranoError;
use std::path::Path;

/// Abstraction over speech-to-text implementations.
pub trait Transcriber {
    /// Load a model from the specified path.
    fn load_model(&mut self, path: &Path) -> Result<(), CyranoError>;

    /// Transcribe audio samples to text.
    ///
    /// Audio must be 16kHz mono f32 samples.
    #[allow(dead_code)] // Will be used in Story 2.2
    fn transcribe(&self, samples: &[f32]) -> Result<String, CyranoError>;

    /// Whether a model is currently loaded.
    fn is_loaded(&self) -> bool;

    /// Unload the model to free memory.
    fn unload(&mut self) -> Result<(), CyranoError>;
}
