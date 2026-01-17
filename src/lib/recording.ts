/**
 * Recording helpers - TypeScript wrappers for recording-related Tauri commands.
 *
 * This file provides convenient functions for managing audio recording,
 * including permission checking and recording lifecycle.
 */

import {
  commands,
  type CyranoError,
  type PermissionStatus,
  type RecordingStoppedPayload,
  unwrapResult,
} from './tauri-bindings'
import { logger } from './logger'

/**
 * Check if microphone permission is granted.
 * @returns The current permission status
 */
export async function checkMicrophonePermission(): Promise<PermissionStatus> {
  logger.debug('Checking microphone permission')
  return commands.checkMicrophonePermission()
}

/**
 * Request microphone permission from the user.
 * On macOS, this triggers the system permission dialog if not previously requested.
 * @returns true if permission was granted
 * @throws CyranoError if permission was denied
 */
export async function requestMicrophonePermission(): Promise<boolean> {
  logger.debug('Requesting microphone permission')
  const result = await commands.requestMicrophonePermission()
  return unwrapResult(result)
}

/**
 * Start recording audio from the microphone.
 * @throws CyranoError if recording fails to start
 */
export async function startRecording(): Promise<void> {
  logger.debug('Starting recording')
  const result = await commands.startRecording()
  unwrapResult(result)
  logger.info('Recording started')
}

/**
 * Stop recording and get the recording information.
 * @returns Recording info including duration and sample count
 * @throws CyranoError if no recording was in progress
 */
export async function stopRecording(): Promise<RecordingStoppedPayload> {
  logger.debug('Stopping recording')
  const result = await commands.stopRecording()
  const payload = unwrapResult(result)
  logger.info('Recording stopped', {
    durationMs: payload.duration_ms,
    sampleCount: payload.sample_count,
  })
  return payload
}

/**
 * Open the macOS System Preferences to microphone settings.
 * Useful when the user needs to grant microphone permission.
 */
export async function openMicrophoneSettings(): Promise<void> {
  logger.debug('Opening microphone settings')
  const result = await commands.openMicrophoneSettings()
  if (result.status === 'error') {
    logger.error('Failed to open microphone settings', { error: result.error })
    throw new Error(result.error)
  }
  logger.info('Opened microphone settings')
}

/**
 * Type guard to check if an error is a microphone access denied error.
 */
export function isMicAccessDenied(error: CyranoError): boolean {
  return error === 'MicAccessDenied'
}

/**
 * Type guard to check if an error is a recording failed error.
 */
export function isRecordingFailed(
  error: CyranoError
): error is { RecordingFailed: { reason: string } } {
  return typeof error === 'object' && 'RecordingFailed' in error
}
