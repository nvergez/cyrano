//! Shortcut registration business logic.
//!
//! This service handles global shortcut registration for the recording feature.
//! It manages the lifecycle of shortcuts including registration, unregistration,
//! and re-registration when settings change.

use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use crate::domain::CyranoError;
/// Default recording shortcut (Cmd+Shift+Space on macOS, Ctrl+Shift+Space elsewhere)
pub const DEFAULT_RECORDING_SHORTCUT: &str = "CommandOrControl+Shift+Space";

/// Tracks the currently registered recording shortcut for selective unregistration.
static CURRENT_RECORDING_SHORTCUT: Mutex<Option<String>> = Mutex::new(None);

/// Payload emitted when the recording shortcut is pressed.
#[derive(Clone, serde::Serialize)]
pub struct RecordingShortcutPayload {
    /// Unix timestamp in milliseconds when the shortcut was pressed
    pub timestamp: u64,
}

/// Gets the current Unix timestamp in milliseconds.
fn get_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Registers the recording global shortcut, unregistering any previously registered one.
///
/// # Arguments
/// * `app_handle` - The Tauri application handle
/// * `shortcut_str` - The shortcut string to register (e.g., "CommandOrControl+Shift+Space")
///
/// # Returns
/// * `Ok(())` if the shortcut was registered successfully
/// * `Err(String)` if registration failed
#[cfg(desktop)]
pub fn register_recording_shortcut(
    app_handle: &AppHandle,
    shortcut_str: &str,
) -> Result<(), CyranoError> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    let global_shortcut = app_handle.global_shortcut();

    // Lock the mutex to get the current shortcut and update it atomically
    let mut current_shortcut =
        CURRENT_RECORDING_SHORTCUT
            .lock()
            .map_err(|e| CyranoError::RecordingFailed {
                reason: format!("Failed to lock recording shortcut mutex: {e}"),
            })?;

    // Unregister the old shortcut if one exists
    if let Some(old_shortcut_str) = current_shortcut.take() {
        log::debug!("Unregistering old recording shortcut: {old_shortcut_str}");
        match old_shortcut_str.parse::<Shortcut>() {
            Ok(old_shortcut) => {
                if let Err(e) = global_shortcut.unregister(old_shortcut) {
                    log::warn!(
                        "Failed to unregister old recording shortcut '{old_shortcut_str}': {e}"
                    );
                    // Continue anyway - the old shortcut may have already been unregistered
                }
            }
            Err(e) => {
                log::warn!("Failed to parse old recording shortcut '{old_shortcut_str}': {e}");
                // Continue anyway - if we can't parse it, we can't unregister it
            }
        }
    }

    // Clone app_handle for the closure
    let app_handle_clone = app_handle.clone();

    // Register the new shortcut with handler
    global_shortcut
        .on_shortcut(shortcut_str, move |_app, _shortcut, event| {
            use tauri_plugin_global_shortcut::ShortcutState;
            if event.state == ShortcutState::Pressed {
                let start = Instant::now();
                let timestamp = get_timestamp_ms();
                log::info!("Recording shortcut triggered at timestamp: {timestamp}");

                let payload = RecordingShortcutPayload { timestamp };

                if let Err(e) = app_handle_clone.emit("recording-shortcut-pressed", payload) {
                    log::error!("Failed to emit recording-shortcut-pressed event: {e}");
                }

                // Toggle behavior: Check if recording is active, stop if so, start if not
                if crate::services::recording_service::is_recording() {
                    // Toggle off: stop recording
                    match crate::services::recording_service::stop_recording(&app_handle_clone) {
                        Ok(payload) => {
                            log::info!(
                                "Recording stopped: {}ms, {} samples",
                                payload.duration_ms,
                                payload.sample_count
                            );
                            // Overlay stays visible, state transitions to Transcribing

                            // Ensure model is loaded before transcription (Story 2.1)
                            // Model loading AND transcription are CPU-intensive, so run on spawned thread
                            let app_for_model = app_handle_clone.clone();
                            std::thread::spawn(move || {
                                // Clear any previous cancellation flag
                                crate::services::transcription_service::clear_cancellation();

                                match crate::services::transcription_service::ensure_model_loaded() {
                                    Ok(()) => {
                                        log::info!("Whisper model ready, starting transcription");

                                        // Emit transcription-started event
                                        let transcription_start = get_timestamp_ms();
                                        let _ = app_for_model.emit(
                                            "transcription-started",
                                            crate::services::recording_service::TranscriptionStartedPayload {
                                                timestamp: transcription_start,
                                            },
                                        );

                                        // Get audio samples
                                        let samples = match crate::services::recording_state::take_audio_samples() {
                                            Ok(s) => s,
                                            Err(e) => {
                                                log::error!("Failed to get audio samples: {e}");
                                                crate::services::recording_state::set_recording_state(
                                                    crate::domain::RecordingState::Error,
                                                );
                                                let _ = app_for_model.emit(
                                                    "transcription-failed",
                                                    crate::services::recording_service::TranscriptionFailedPayload {
                                                        error: crate::domain::CyranoError::TranscriptionFailed {
                                                            reason: e,
                                                        },
                                                    },
                                                );
                                                return;
                                            }
                                        };

                                        // Perform transcription
                                        match crate::services::transcription_service::transcribe(&samples) {
                                            Ok(text) => {
                                                let duration_ms = (get_timestamp_ms() - transcription_start) as u32;
                                                log::info!(
                                                    "Transcription complete: {} chars in {}ms",
                                                    text.len(),
                                                    duration_ms
                                                );

                                                // Copy to clipboard (FR12) - do this BEFORE emitting success event
                                                match crate::services::output_service::copy_to_clipboard(&text, &app_for_model) {
                                                    Ok(()) => {
                                                        log::debug!("Clipboard copy succeeded");
                                                        // Emit clipboard-copied event for UI feedback
                                                        let _ = app_for_model.emit(
                                                            "clipboard-copied",
                                                            crate::services::recording_service::ClipboardCopiedPayload {
                                                                text_length: text.len() as u32,
                                                            },
                                                        );
                                                    }
                                                    Err(e) => {
                                                        // Clipboard failure is non-fatal - log and continue
                                                        // User still gets the transcription, just needs to manually copy
                                                        log::warn!("Clipboard copy failed: {e}");
                                                        let _ = app_for_model.emit(
                                                            "clipboard-failed",
                                                            crate::services::recording_service::ClipboardFailedPayload {
                                                                error: e,
                                                            },
                                                        );
                                                    }
                                                }

                                                crate::services::recording_state::set_recording_state(
                                                    crate::domain::RecordingState::Done,
                                                );
                                                let _ = app_for_model.emit(
                                                    "transcription-complete",
                                                    crate::services::recording_service::TranscriptionCompletePayload {
                                                        text,
                                                        duration_ms,
                                                    },
                                                );
                                            }
                                            Err(e) => {
                                                // Check if this was a cancellation
                                                let is_cancellation = matches!(&e, crate::domain::CyranoError::TranscriptionFailed { reason } if reason.contains("cancelled"));

                                                if is_cancellation {
                                                    log::info!("Transcription was cancelled");
                                                    crate::services::recording_state::set_recording_state(
                                                        crate::domain::RecordingState::Idle,
                                                    );
                                                    let _ = app_for_model.emit(
                                                        "transcription-cancelled",
                                                        crate::services::recording_service::TranscriptionCancelledPayload {
                                                            timestamp: get_timestamp_ms(),
                                                        },
                                                    );
                                                } else {
                                                    log::error!("Transcription failed: {e}");
                                                    crate::services::recording_state::set_recording_state(
                                                        crate::domain::RecordingState::Error,
                                                    );
                                                    let _ = app_for_model.emit(
                                                        "transcription-failed",
                                                        crate::services::recording_service::TranscriptionFailedPayload {
                                                            error: e,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Model loading failed: {e}");
                                        // Set state to Error and emit recording-failed event
                                        crate::services::recording_state::set_recording_state(
                                            crate::domain::RecordingState::Error,
                                        );
                                        let payload =
                                            crate::services::recording_service::RecordingFailedPayload {
                                                error: e,
                                            };
                                        if let Err(emit_err) =
                                            app_for_model.emit("recording-failed", payload)
                                        {
                                            log::error!(
                                                "Failed to emit recording-failed event: {emit_err}"
                                            );
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("Failed to stop recording: {e}");
                            // Emit error event for overlay to display
                            let payload =
                                crate::services::recording_service::RecordingFailedPayload {
                                    error: e,
                                };
                            if let Err(emit_err) =
                                app_handle_clone.emit("recording-failed", payload)
                            {
                                log::error!("Failed to emit recording-failed event: {emit_err}");
                            }
                        }
                    }
                } else {
                    // Toggle on: start recording
                    match crate::services::recording_service::start_recording(&app_handle_clone) {
                        Ok(()) => {
                            log::info!("Recording started successfully");
                            // Show the recording overlay when recording starts
                            if let Err(e) =
                                crate::commands::recording_overlay::show_recording_overlay(
                                    app_handle_clone.clone(),
                                )
                            {
                                log::error!("Failed to show recording overlay: {e}");
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to start recording: {e}");
                            // Show overlay first so it can receive the error event
                            if let Err(overlay_err) =
                                crate::commands::recording_overlay::show_recording_overlay(
                                    app_handle_clone.clone(),
                                )
                            {
                                log::error!("Failed to show recording overlay: {overlay_err}");
                            }
                            // Now emit the recording-failed event so the overlay displays error state
                            let payload =
                                crate::services::recording_service::RecordingFailedPayload {
                                    error: e,
                                };
                            if let Err(emit_err) =
                                app_handle_clone.emit("recording-failed", payload)
                            {
                                log::error!("Failed to emit recording-failed event: {emit_err}");
                            }
                        }
                    }
                }

                let elapsed_ms = start.elapsed().as_millis();
                log::info!("Recording shortcut handler duration: {elapsed_ms}ms");
                if elapsed_ms > 100 {
                    log::warn!(
                        "Recording shortcut handler exceeded 100ms threshold: {elapsed_ms}ms"
                    );
                }
            }
        })
        .map_err(|e| CyranoError::RecordingFailed {
            reason: format!("Failed to register recording shortcut '{shortcut_str}': {e}"),
        })?;

    // Store the new shortcut for future unregistration
    *current_shortcut = Some(shortcut_str.to_string());
    log::debug!("Registered recording shortcut: {shortcut_str}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_recording_shortcut_is_valid() {
        // Verify the default shortcut constant is a non-empty string
        assert!(!DEFAULT_RECORDING_SHORTCUT.is_empty());
        assert!(DEFAULT_RECORDING_SHORTCUT.contains("CommandOrControl"));
        assert!(DEFAULT_RECORDING_SHORTCUT.contains("Shift"));
        assert!(DEFAULT_RECORDING_SHORTCUT.contains("Space"));
    }

    #[test]
    fn test_get_timestamp_ms_returns_reasonable_value() {
        let ts = get_timestamp_ms();
        // Should be a reasonable Unix timestamp (after 2020)
        let jan_2020_ms: u64 = 1577836800000;
        assert!(ts > jan_2020_ms, "Timestamp should be after January 2020");
    }

    #[test]
    fn test_recording_shortcut_payload_serializes() {
        let payload = RecordingShortcutPayload {
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&payload).expect("Should serialize");
        assert!(json.contains("1234567890"));
    }
}
