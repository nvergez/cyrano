//! Port abstractions for infrastructure.
//!
//! This module contains trait definitions (ports) that infrastructure adapters implement.
//! Services depend on these traits, not on concrete implementations.

pub mod audio_capture;
pub mod transcriber;
