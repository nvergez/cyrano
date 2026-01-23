//! Permission-related infrastructure adapters.
//!
//! Platform-specific implementations for checking and requesting
//! system permissions (accessibility, microphone, etc.).

#[cfg(target_os = "macos")]
pub mod macos_accessibility;
