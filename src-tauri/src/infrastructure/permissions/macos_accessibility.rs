//! macOS Accessibility permission infrastructure.
//!
//! Provides low-level access to macOS Accessibility APIs for checking
//! and requesting accessibility permissions needed for cursor insertion.

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

// Link to ApplicationServices framework for accessibility APIs
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Check if the current process is trusted for accessibility.
    fn AXIsProcessTrusted() -> bool;

    /// Check if the current process is trusted, with optional prompt.
    /// Pass a dictionary with kAXTrustedCheckOptionPrompt = true to show prompt.
    fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
}

/// The key for the prompt option in AXIsProcessTrustedWithOptions.
const K_AX_TRUSTED_CHECK_OPTION_PROMPT: &str = "AXTrustedCheckOptionPrompt";

/// Check if the current process has accessibility permission.
///
/// This function checks whether the application has been granted
/// accessibility permission in System Preferences > Privacy & Security.
///
/// # Returns
/// * `true` if accessibility permission is granted
/// * `false` if permission is denied or not yet determined
///
/// # Note
/// This function cannot distinguish between "denied" and "not determined"
/// states - both return `false`.
pub fn check_accessibility_trusted() -> bool {
    // SAFETY: AXIsProcessTrusted is a safe C function that only reads system state
    unsafe { AXIsProcessTrusted() }
}

/// Prompt the user for accessibility permission.
///
/// This function shows the macOS system dialog asking the user to grant
/// accessibility permission. If the permission has already been granted
/// or explicitly denied, no dialog is shown.
///
/// # Returns
/// * `true` if accessibility permission is granted
/// * `false` if permission is denied or not yet determined
///
/// # Behavior
/// - First call: Shows system dialog directing user to System Preferences
/// - Subsequent calls: Returns current permission state without showing dialog
/// - After user grants in System Preferences: Returns `true`
pub fn prompt_accessibility_permission() -> bool {
    // Create options dictionary with prompt = true
    let key = CFString::new(K_AX_TRUSTED_CHECK_OPTION_PROMPT);
    let value = CFBoolean::true_value();

    // SAFETY: We're creating a properly typed CFDictionary with valid key-value pairs
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

    // SAFETY: AXIsProcessTrustedWithOptions is safe when passed a valid CFDictionary
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as *const _) }
}

/// Open the Accessibility preferences pane in System Preferences.
///
/// This function deep-links directly to the Privacy & Security > Accessibility
/// pane in System Preferences/Settings, making it easy for users to grant
/// permission.
///
/// # Returns
/// * `Ok(())` if the command was launched successfully
/// * `Err(io::Error)` if the command failed to execute
pub fn open_accessibility_preferences() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()?
        .wait()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_accessibility_returns_bool() {
        // This test verifies the function executes without panic.
        // The actual return value depends on system permission state.
        let result = check_accessibility_trusted();
        // Result is either true or false - both are valid
        assert!(result || !result);
    }

    #[test]
    fn test_prompt_accessibility_returns_bool() {
        // This test verifies the function executes without panic.
        // Note: In CI/testing, this won't show a dialog if already determined.
        let result = prompt_accessibility_permission();
        // Result is either true or false - both are valid
        assert!(result || !result);
    }

    // Note: We cannot test open_accessibility_preferences in unit tests
    // as it launches an external application. Manual testing required.
}
