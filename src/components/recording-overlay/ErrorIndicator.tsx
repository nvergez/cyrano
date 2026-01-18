import { commands, type CyranoError } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'

interface ErrorIndicatorProps {
  error: CyranoError | null
}

/**
 * Extracts the error type and details from a CyranoError.
 */
function parseError(error: CyranoError | null): {
  type: 'mic' | 'model-not-found' | 'model-load-failed' | 'other'
  message: string
  details?: string
} {
  if (!error) {
    return { type: 'other', message: 'Unknown error' }
  }

  if (error === 'MicAccessDenied') {
    return { type: 'mic', message: 'Microphone access denied' }
  }

  if (typeof error === 'object') {
    if ('ModelNotFound' in error) {
      return {
        type: 'model-not-found',
        message: 'Model not found',
        details: error.ModelNotFound.path,
      }
    }
    if ('ModelLoadFailed' in error) {
      return {
        type: 'model-load-failed',
        message: 'Failed to load model',
        details: error.ModelLoadFailed.reason,
      }
    }
    if ('RecordingFailed' in error) {
      return {
        type: 'other',
        message: 'Recording failed',
        details: error.RecordingFailed.reason,
      }
    }
    if ('TranscriptionFailed' in error) {
      return {
        type: 'other',
        message: 'Transcription failed',
        details: error.TranscriptionFailed.reason,
      }
    }
  }

  return { type: 'other', message: 'Recording failed' }
}

/**
 * ErrorIndicator - Visual indicator for recording error state.
 *
 * Displays a red cross icon with error message and provides
 * actionable links to resolve the error (e.g., open settings or model directory).
 */
export function ErrorIndicator({ error }: ErrorIndicatorProps) {
  const { type, message, details } = parseError(error)

  const handleOpenMicSettings = async () => {
    logger.info('User clicked to open microphone settings')
    const result = await commands.openMicrophoneSettings()
    if (result.status === 'error') {
      logger.error('Failed to open microphone settings', {
        error: result.error,
      })
    }
  }

  const handleOpenModelDirectory = async () => {
    logger.info('User clicked to open model directory')
    const result = await commands.openModelDirectory()
    if (result.status === 'error') {
      logger.error('Failed to open model directory', {
        error: result.error,
      })
    }
  }

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

      {/* Error message and action link */}
      <div className="flex flex-col">
        <span className="text-sm font-medium text-destructive">{message}</span>

        {/* Details if available (model path or error reason) */}
        {details && type !== 'mic' && (
          <span className="max-w-[200px] truncate text-xs text-muted-foreground">
            {details}
          </span>
        )}

        {/* Microphone access denied - link to system settings */}
        {type === 'mic' && (
          <button
            onClick={handleOpenMicSettings}
            className="text-left text-xs text-muted-foreground underline hover:text-foreground"
          >
            Open System Preferences
          </button>
        )}

        {/* Model not found or load failed - link to open model directory */}
        {(type === 'model-not-found' || type === 'model-load-failed') && (
          <button
            onClick={handleOpenModelDirectory}
            className="text-left text-xs text-muted-foreground underline hover:text-foreground"
          >
            Open Model Directory
          </button>
        )}
      </div>
    </div>
  )
}
