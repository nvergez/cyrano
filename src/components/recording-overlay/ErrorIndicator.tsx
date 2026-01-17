import { commands, type CyranoError } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'

interface ErrorIndicatorProps {
  error: CyranoError | null
}

/**
 * ErrorIndicator - Visual indicator for recording error state.
 *
 * Displays a red cross icon with error message and provides
 * a link to open System Preferences when microphone access is denied.
 */
export function ErrorIndicator({ error }: ErrorIndicatorProps) {
  const isMicDenied = error === 'MicAccessDenied'

  const handleOpenSettings = async () => {
    logger.info('User clicked to open microphone settings')
    const result = await commands.openMicrophoneSettings()
    if (result.status === 'error') {
      logger.error('Failed to open microphone settings', {
        error: result.error,
      })
    }
  }

  const errorMessage = isMicDenied
    ? 'Microphone access denied'
    : 'Recording failed'

  return (
    <div className="flex items-center gap-3">
      {/* Red X icon */}
      <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-destructive/20">
        <svg
          className="h-4 w-4 text-destructive"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2.5}
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </div>

      {/* Error message and settings link */}
      <div className="flex flex-col">
        <span className="text-sm font-medium text-destructive">
          {errorMessage}
        </span>
        {isMicDenied && (
          <button
            onClick={handleOpenSettings}
            className="text-left text-xs text-muted-foreground underline hover:text-foreground"
          >
            Open System Preferences
          </button>
        )}
      </div>
    </div>
  )
}
