//! Core domain types.
//!
//! This module contains pure domain types with NO dependencies on other project modules.
//! Only standard library and serialization crates are allowed here.

// These types are foundation for future features - allow unused until integrated
#![allow(dead_code, unused_imports)]

mod error;
mod state;

pub use error::CyranoError;
pub use state::{PermissionStatus, RecordingState};
