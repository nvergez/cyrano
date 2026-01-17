import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { RecordingIndicator } from './RecordingIndicator'

/**
 * RecordingOverlay - Main overlay component for recording state display.
 *
 * This component displays the current recording state with a click-to-cancel
 * behavior. Clicking anywhere on the overlay cancels the current recording
 * and dismisses the overlay.
 */
export function RecordingOverlay() {
  const handleClick = async () => {
    logger.info('Recording overlay clicked - cancelling recording')
    const result = await commands.cancelRecording()
    if (result.status === 'error') {
      logger.error('Failed to cancel recording', { error: result.error })
    }
  }

  return (
    <div
      onClick={handleClick}
      className="flex h-screen w-screen cursor-pointer items-center justify-center rounded-xl border border-border bg-background/95 px-6 py-4 shadow-lg backdrop-blur"
    >
      <RecordingIndicator />
    </div>
  )
}
