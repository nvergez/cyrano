//! Business logic orchestration layer.
//!
//! This module contains services that coordinate business logic.
//! Services depend on infrastructure adapters through traits (ports).

pub mod permission_service;
pub mod recording_service;
pub mod recording_state;
pub mod shortcut_service;
pub mod transcription_service;
