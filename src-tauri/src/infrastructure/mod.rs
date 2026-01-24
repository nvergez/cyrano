//! External integrations layer.
//!
//! This module contains adapters for external systems:
//! - Audio capture (cpal)
//! - Speech-to-text (whisper-rs)
//! - macOS accessibility APIs
//! - Keyboard simulation (CGEvent)

pub mod audio;
pub mod keyboard;
pub mod permissions;
pub mod whisper;
