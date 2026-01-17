/**
 * Recording overlay invoke helpers.
 *
 * Provides typed wrappers for recording overlay Tauri commands
 * with consistent error handling.
 */

import { commands } from './tauri-bindings'
import { logger } from './logger'

/**
 * Shows the recording overlay window.
 * @returns true if successful, false if an error occurred
 */
export async function showRecordingOverlay(): Promise<boolean> {
  const result = await commands.showRecordingOverlay()
  if (result.status === 'error') {
    logger.error('Failed to show recording overlay', { error: result.error })
    return false
  }
  return true
}

/**
 * Dismisses the recording overlay window.
 * @returns true if successful, false if an error occurred
 */
export async function dismissRecordingOverlay(): Promise<boolean> {
  const result = await commands.dismissRecordingOverlay()
  if (result.status === 'error') {
    logger.error('Failed to dismiss recording overlay', { error: result.error })
    return false
  }
  return true
}

/**
 * Toggles the recording overlay window visibility.
 * @returns true if successful, false if an error occurred
 */
export async function toggleRecordingOverlay(): Promise<boolean> {
  const result = await commands.toggleRecordingOverlay()
  if (result.status === 'error') {
    logger.error('Failed to toggle recording overlay', { error: result.error })
    return false
  }
  return true
}

/**
 * Cancels the current recording, dismisses overlay, returns to idle state.
 * @returns true if successful, false if an error occurred
 */
export async function cancelRecording(): Promise<boolean> {
  const result = await commands.cancelRecording()
  if (result.status === 'error') {
    logger.error('Failed to cancel recording', { error: result.error })
    return false
  }
  return true
}
