import { Spinner } from '@/components/ui/spinner'

/**
 * TranscribingIndicator - Visual indicator for transcription in progress.
 *
 * Displays a spinning loader with "Transcribing..." text to provide
 * clear visual feedback that audio is being processed.
 */
export function TranscribingIndicator() {
  return (
    <div className="flex items-center gap-3">
      <Spinner className="size-5 text-blue-500" />
      <span className="text-lg font-medium text-foreground">
        Transcribing...
      </span>
    </div>
  )
}
