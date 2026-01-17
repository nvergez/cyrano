/**
 * RecordingIndicator - Visual indicator for active recording state.
 *
 * Displays a pulsing red dot with "Recording..." text to provide
 * clear visual feedback that audio capture is in progress.
 */
export function RecordingIndicator() {
  return (
    <div className="flex items-center gap-3">
      <span className="h-3 w-3 animate-pulse-recording rounded-full bg-red-500" />
      <span className="text-lg font-medium text-foreground">Recording...</span>
    </div>
  )
}
