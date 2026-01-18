/**
 * SuccessIndicator - Visual indicator for successful transcription.
 *
 * Displays a green checkmark icon with success message.
 * Appears after transcription completes and text is copied to clipboard.
 */
export function SuccessIndicator() {
  return (
    <div className="flex items-center gap-3">
      {/* Green check icon */}
      <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-green-500/20">
        <svg
          className="h-4 w-4 text-green-500"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2.5}
            d="M5 13l4 4L19 7"
          />
        </svg>
      </div>

      {/* Success message */}
      <span className="text-sm font-medium text-green-500">Done</span>
    </div>
  )
}
