import { useEffect, useRef } from 'react'
import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { useUIStore } from '@/store/ui-store'
import { RecordingIndicator } from './RecordingIndicator'
import { ErrorIndicator } from './ErrorIndicator'
import { SuccessIndicator } from './SuccessIndicator'
import { TranscribingIndicator } from './TranscribingIndicator'

/** Auto-dismiss delay for success state (1.2 seconds) */
export const AUTO_DISMISS_SUCCESS_MS = 1200

/** Auto-dismiss delay for error state (1.8 seconds - longer for error reading) */
export const AUTO_DISMISS_ERROR_MS = 1800

/**
 * RecordingOverlay - Main overlay component for recording state display.
 *
 * This component displays the current recording state with a click-to-cancel
 * behavior. Clicking anywhere on the overlay cancels the current recording
 * and dismisses the overlay. When an error occurs, displays the error state.
 * Terminal states (done/error) auto-dismiss after a short delay.
 */
export function RecordingOverlay() {
  const recordingState = useUIStore(state => state.recordingState)
  const recordingError = useUIStore(state => state.recordingError)
  const isError = recordingState === 'error' || recordingError !== null

  // Ref to store timeout ID for cleanup and cancellation
  const dismissTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  /**
   * Shared dismiss logic for both manual click and auto-dismiss.
   * Clears any pending timeout, resets state, and hides overlay.
   */
  const dismissOverlay = async () => {
    // Clear any pending timeout first
    if (dismissTimeoutRef.current) {
      clearTimeout(dismissTimeoutRef.current)
      dismissTimeoutRef.current = null
    }

    const {
      clearRecordingError,
      setRecordingOverlayVisible,
      setRecordingState,
      clearTranscriptionResult,
    } = useUIStore.getState()

    clearRecordingError()
    setRecordingState('idle')
    setRecordingOverlayVisible(false)
    clearTranscriptionResult()

    const result = await commands.dismissRecordingOverlay()
    if (result.status === 'error') {
      logger.error('Failed to dismiss overlay', { error: result.error })
    }
  }

  // Auto-dismiss effect for done/error states
  useEffect(() => {
    // Only set timer for terminal states (done or error)
    if (recordingState === 'done' || isError) {
      const delay = isError ? AUTO_DISMISS_ERROR_MS : AUTO_DISMISS_SUCCESS_MS

      logger.debug('Starting auto-dismiss timer', {
        state: isError ? 'error' : 'done',
        delayMs: delay,
      })

      dismissTimeoutRef.current = setTimeout(() => {
        logger.info('Auto-dismissing overlay', {
          state: isError ? 'error' : 'done',
        })
        dismissOverlay()
      }, delay)
    }

    // Cleanup: clear timeout if state changes or component unmounts
    return () => {
      if (dismissTimeoutRef.current) {
        clearTimeout(dismissTimeoutRef.current)
        dismissTimeoutRef.current = null
      }
    }
  }, [recordingState, isError])

  const handleClick = async () => {
    // Clear any pending auto-dismiss timeout
    if (dismissTimeoutRef.current) {
      clearTimeout(dismissTimeoutRef.current)
      dismissTimeoutRef.current = null
    }

    if (recordingState === 'transcribing') {
      // In transcribing state, cancel transcription
      logger.info('Recording overlay clicked - cancelling transcription')
      // Request cancellation - the backend will emit transcription-cancelled event
      // which will trigger the cleanup in RecordingOverlayApp
      commands.cancelTranscription()
    } else if (recordingState === 'recording') {
      // In recording state, cancel recording
      logger.info('Recording overlay clicked - cancelling recording')
      const result = await commands.cancelRecording()
      if (result.status === 'error') {
        logger.error('Failed to cancel recording', { error: result.error })
      }
    } else {
      // In idle, done, or error state - just dismiss
      logger.info('Recording overlay clicked - dismissing', {
        state: recordingState,
        isError,
      })
      await dismissOverlay()
    }
  }

  const renderIndicator = () => {
    if (isError) {
      return <ErrorIndicator error={recordingError} />
    }
    if (recordingState === 'done') {
      return <SuccessIndicator />
    }
    if (recordingState === 'transcribing') {
      return <TranscribingIndicator />
    }
    return <RecordingIndicator />
  }

  return (
    <div
      onClick={handleClick}
      className="flex h-screen w-screen cursor-pointer items-center justify-center rounded-xl border border-border bg-background/95 px-6 py-4 shadow-lg backdrop-blur"
    >
      {renderIndicator()}
    </div>
  )
}
