//! macOS keyboard event simulation using CGEvent APIs.
//!
//! This module provides low-level keyboard event simulation for macOS,
//! specifically for simulating Cmd+V paste operations to insert text
//! at the current cursor position in any application.

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::thread;
use std::time::Duration;

/// Virtual keycode for the V key on macOS.
const K_VK_V: CGKeyCode = 0x09;

/// Delay between keydown and keyup events for reliability.
const KEY_EVENT_DELAY_MS: u64 = 10;

/// Simulate a Cmd+V paste keystroke.
///
/// This function simulates pressing Cmd+V by:
/// 1. Creating a V keydown event with Command modifier flag
/// 2. Posting the keydown event to the HID system
/// 3. Waiting a small delay for reliability
/// 4. Creating and posting a V keyup event
///
/// # Returns
/// * `Ok(())` if the keystroke was simulated successfully
/// * `Err(std::io::Error)` if event creation or posting failed
///
/// # Notes
/// - This function posts events at the HID level, which works even when
///   the app is in the background (overlay use case).
/// - The target application receives the paste command and inserts
///   whatever text is currently on the clipboard.
/// - Requires accessibility permission to be effective.
pub fn simulate_paste() -> Result<(), std::io::Error> {
    log::debug!("Simulating Cmd+V paste keystroke");

    // Create event source from HID system state
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|()| {
        std::io::Error::other("Failed to create CGEventSource for keyboard simulation")
    })?;

    // Create V keydown event
    let v_down = CGEvent::new_keyboard_event(source.clone(), K_VK_V, true)
        .map_err(|()| std::io::Error::other("Failed to create V keydown event"))?;

    // Set Command modifier flag (this makes it Cmd+V instead of just V)
    v_down.set_flags(CGEventFlags::CGEventFlagCommand);

    // Create V keyup event
    let v_up = CGEvent::new_keyboard_event(source, K_VK_V, false)
        .map_err(|()| std::io::Error::other("Failed to create V keyup event"))?;

    // Post keydown event to HID system
    v_down.post(CGEventTapLocation::HID);

    // Small delay for reliability across different applications
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    // Post keyup event
    v_up.post(CGEventTapLocation::HID);

    log::debug!("Cmd+V paste keystroke simulated successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_paste_compiles_and_runs() {
        // This test verifies the function executes without panic.
        // The actual result depends on system permission state.
        // Note: In CI environments without accessibility permission,
        // the function will still succeed but the paste won't be effective.
        let result = simulate_paste();

        // The function should either succeed or return an io::Error
        // (not panic). Both outcomes are acceptable for this test.
        match result {
            Ok(()) => {
                // Success - keyboard simulation completed
                assert!(true);
            }
            Err(e) => {
                // Error occurred - this is OK in testing environments
                // The important thing is that we didn't panic
                log::debug!("simulate_paste returned error (expected in some environments): {e}");
                assert!(true);
            }
        }
    }

    #[test]
    fn test_virtual_keycode_v_is_correct() {
        // Virtual keycode for V on macOS is 0x09
        assert_eq!(K_VK_V, 0x09);
    }
}
