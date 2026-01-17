//! Audio capture port (trait).
//!
//! Defines the interface that audio capture adapters must implement.

use crate::domain::CyranoError;

/// Abstraction over audio capture implementations.
pub trait AudioCapture {
    /// Start capturing audio.
    fn start_capture(&mut self) -> Result<(), CyranoError>;

    /// Stop capturing audio and return captured samples.
    fn stop_capture(&mut self) -> Result<Vec<f32>, CyranoError>;

    /// Whether audio capture is currently active.
    #[allow(dead_code)]
    fn is_capturing(&self) -> bool;
}
