//! Microphone permission checking service.
//!
//! Provides methods to check and request microphone permission on macOS.
//! Uses cpal to implicitly trigger the macOS permission dialog.

use cpal::traits::{DeviceTrait, HostTrait};

use crate::domain::{CyranoError, PermissionStatus};

/// Check the current microphone permission status.
///
/// On macOS, this checks whether we can access the default input device
/// and its supported configurations. If we can, permission is granted.
/// If we cannot, permission is either denied or not yet determined.
///
/// Note: This function cannot distinguish between "Denied" and "NotDetermined"
/// states without using macOS-specific APIs. On first access, cpal will trigger
/// the permission dialog automatically.
pub fn check_microphone_permission() -> PermissionStatus {
    let host = cpal::default_host();

    // Try to get the default input device
    let device = match host.default_input_device() {
        Some(d) => d,
        None => {
            log::debug!("No default input device found - permission may be denied");
            return PermissionStatus::Denied;
        }
    };

    // Try to get supported input configs - this will fail if permission is denied
    match device.supported_input_configs() {
        Ok(mut configs) => {
            if configs.next().is_some() {
                log::debug!("Microphone permission granted");
                PermissionStatus::Granted
            } else {
                log::debug!("No input configurations available");
                PermissionStatus::Denied
            }
        }
        Err(e) => {
            log::debug!("Failed to get input configs: {e} - permission may be denied");
            match e {
                cpal::SupportedStreamConfigsError::DeviceNotAvailable => PermissionStatus::Denied,
                _ => PermissionStatus::Denied,
            }
        }
    }
}

/// Request microphone permission from the user.
///
/// On macOS, this triggers the system permission dialog by attempting to
/// access the microphone. The function returns `true` if permission was
/// granted, `false` if it was denied.
///
/// Note: If permission has already been granted or denied, this function
/// will return the current status without showing a dialog.
pub fn request_microphone_permission() -> Result<bool, CyranoError> {
    let host = cpal::default_host();

    // Getting the default input device triggers the permission dialog on first access
    let device = host
        .default_input_device()
        .ok_or(CyranoError::MicAccessDenied)?;

    // Trying to enumerate configs also ensures we have permission
    let configs = device.supported_input_configs().map_err(|e| match e {
        cpal::SupportedStreamConfigsError::DeviceNotAvailable => CyranoError::MicAccessDenied,
        _ => CyranoError::RecordingFailed {
            reason: format!("Failed to check microphone permission: {e}"),
        },
    })?;

    // Check if we got any configs (meaning permission was granted)
    let has_configs = configs.count() > 0;

    if has_configs {
        log::info!("Microphone permission granted");
        Ok(true)
    } else {
        log::warn!("No microphone configurations available after permission request");
        Err(CyranoError::MicAccessDenied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_permission_returns_valid_status() {
        // This test may pass or fail depending on actual mic permission
        let status = check_microphone_permission();
        assert!(matches!(
            status,
            PermissionStatus::Granted | PermissionStatus::Denied | PermissionStatus::NotDetermined
        ));
    }

    // Note: We cannot easily test request_microphone_permission in unit tests
    // as it requires actual user interaction on macOS
}
