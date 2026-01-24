//! Cursor insertion service for text placement at cursor position.
//!
//! This service handles cursor insertion by simulating a Cmd+V paste
//! keystroke after text has been copied to the clipboard. It follows
//! the graceful degradation pattern: if insertion fails, the text
//! remains in the clipboard for manual pasting.

use crate::domain::CyranoError;
use crate::infrastructure::keyboard;
use crate::services::output_service;

/// Insert text at the current cursor position.
///
/// This function attempts to insert text at the cursor position by
/// simulating a Cmd+V paste keystroke. It requires that text has
/// already been copied to the clipboard.
///
/// # Returns
/// * `Ok(())` always - this function uses graceful degradation
///
/// # Graceful Degradation
/// This function NEVER returns an error to the caller. The philosophy is:
/// - Clipboard already has the text (prerequisite)
/// - If paste simulation fails, user can still paste manually
/// - No error visible to user - this is a bonus feature, not critical
///
/// # Prerequisites
/// - Text must already be on the clipboard
/// - Accessibility permission should be granted (checked internally)
///
/// # Notes
/// - If accessibility permission is not granted, the function returns
///   `Ok(())` without attempting paste simulation (graceful skip).
/// - If paste simulation fails, the error is logged but `Ok(())` is returned.
pub fn insert_at_cursor() -> Result<(), CyranoError> {
    // Check if cursor insertion is available (accessibility permission granted)
    if !output_service::is_cursor_insertion_available() {
        log::debug!("Cursor insertion skipped: accessibility permission not granted");
        return Ok(()); // Graceful degradation - not an error
    }

    // Small delay to ensure clipboard is ready after write
    // This improves reliability across different applications
    std::thread::sleep(std::time::Duration::from_millis(20));

    // Attempt to simulate Cmd+V paste
    match keyboard::simulate_paste() {
        Ok(()) => {
            log::info!("Cursor insertion successful via Cmd+V simulation");
            Ok(())
        }
        Err(e) => {
            // Log the error but return Ok - graceful degradation
            // The text is already in the clipboard, user can paste manually
            log::warn!(
                "Cursor insertion failed, but text is in clipboard for manual paste: {}",
                e
            );
            Ok(()) // Still return Ok - this is graceful degradation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_at_cursor_never_panics() {
        // This test verifies the function executes without panic.
        // The actual result depends on system permission state.
        let result = insert_at_cursor();

        // The function should ALWAYS return Ok due to graceful degradation
        assert!(result.is_ok(), "insert_at_cursor should always return Ok");
    }

    #[test]
    fn test_insert_at_cursor_returns_ok_type() {
        // Verify the return type is correct
        let result: Result<(), CyranoError> = insert_at_cursor();
        // Should be Ok regardless of system state
        assert!(result.is_ok());
    }
}
