import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { useUIStore } from '@/store/ui-store'
import { RecordingIndicator } from './RecordingIndicator'
import { ErrorIndicator } from './ErrorIndicator'
import { TranscribingIndicator } from './TranscribingIndicator'

/**
 * RecordingOverlay - Main overlay component for recording state display.
 *
 * This component displays the current recording state with a click-to-cancel
 * behavior. Clicking anywhere on the overlay cancels the current recording
 * and dismisses the overlay. When an error occurs, displays the error state.
 */
export function RecordingOverlay() {
  const recordingState = useUIStore(state => state.recordingState)
  const recordingError = useUIStore(state => state.recordingError)
  const isError = recordingState === 'error' || recordingError !== null

  const handleClick = async () => {
    if (isError) {
      // In error state, just dismiss without cancelling
      logger.info('Recording overlay clicked in error state - dismissing')
      const {
        clearRecordingError,
        setRecordingOverlayVisible,
        setRecordingState,
      } = useUIStore.getState()
      clearRecordingError()
      setRecordingState('idle')
      setRecordingOverlayVisible(false)
      const result = await commands.dismissRecordingOverlay()
      if (result.status === 'error') {
        logger.error('Failed to dismiss overlay', { error: result.error })
      }
    } else if (recordingState === 'transcribing') {
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
      // In idle or done state, just dismiss
      logger.info('Recording overlay clicked in idle/done state - dismissing')
      const { setRecordingOverlayVisible, setRecordingState } =
        useUIStore.getState()
      setRecordingState('idle')
      setRecordingOverlayVisible(false)
      const result = await commands.dismissRecordingOverlay()
      if (result.status === 'error') {
        logger.error('Failed to dismiss overlay', { error: result.error })
      }
    }
  }

  const renderIndicator = () => {
    if (isError) {
      return <ErrorIndicator error={recordingError} />
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
