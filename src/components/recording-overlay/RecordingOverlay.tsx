import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { useUIStore } from '@/store/ui-store'
import { RecordingIndicator } from './RecordingIndicator'
import { ErrorIndicator } from './ErrorIndicator'

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
    } else {
      logger.info('Recording overlay clicked - cancelling recording')
      const result = await commands.cancelRecording()
      if (result.status === 'error') {
        logger.error('Failed to cancel recording', { error: result.error })
      }
    }
  }

  return (
    <div
      onClick={handleClick}
      className="flex h-screen w-screen cursor-pointer items-center justify-center rounded-xl border border-border bg-background/95 px-6 py-4 shadow-lg backdrop-blur"
    >
      {isError ? (
        <ErrorIndicator error={recordingError} />
      ) : (
        <RecordingIndicator />
      )}
    </div>
  )
}
