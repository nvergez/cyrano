//! Keyboard simulation infrastructure.
//!
//! Provides low-level keyboard event simulation for macOS.
//! Currently supports paste simulation (Cmd+V) for cursor insertion.

#[cfg(target_os = "macos")]
pub mod macos_keyboard;

#[cfg(target_os = "macos")]
pub use macos_keyboard::simulate_paste;
