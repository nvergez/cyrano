import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { logger } from '@/lib/logger'

/**
 * Payload emitted when the recording shortcut is pressed.
 */
interface RecordingShortcutPayload {
  /** Unix timestamp in milliseconds when the shortcut was pressed */
  timestamp: number
}

/**
 * Hook to listen for the recording shortcut event from the Tauri backend.
 *
 * This hook sets up a listener for the `recording-shortcut-pressed` event
 * which is emitted when the user presses the global recording shortcut
 * (default: Cmd+Shift+Space on macOS, Ctrl+Shift+Space on Windows/Linux).
 *
 * @param onShortcutPressed - Callback function to invoke when the shortcut is pressed.
 *                            Will be called with no arguments.
 *
 * @example
 * ```tsx
 * function RecordingComponent() {
 *   const handleRecordingShortcut = useCallback(() => {
 *     console.log('Recording shortcut pressed!');
 *     // Start or toggle recording
 *   }, []);
 *
 *   useRecordingShortcut(handleRecordingShortcut);
 *
 *   return <div>Press Cmd+Shift+Space to record</div>;
 * }
 * ```
 */
export function useRecordingShortcut(onShortcutPressed: () => void): void {
  useEffect(() => {
    let isMounted = true
    let unlisten: (() => void) | null = null

    listen<RecordingShortcutPayload>('recording-shortcut-pressed', event => {
      if (!isMounted) return

      const receiveTime = Date.now()
      const pressTime = event.payload.timestamp
      const latency = receiveTime - pressTime

      logger.debug('Recording shortcut pressed', {
        pressTime,
        receiveTime,
        latencyMs: latency,
      })

      // Log performance warning if latency exceeds threshold (NFR1: < 100ms)
      if (latency > 100) {
        logger.warn('Recording shortcut latency exceeded 100ms threshold', {
          latencyMs: latency,
        })
      }

      onShortcutPressed()
    })
      .then(unlistenFn => {
        if (!isMounted) {
          // Component unmounted before listener was set up
          unlistenFn()
        } else {
          unlisten = unlistenFn
        }
      })
      .catch(error => {
        logger.error('Failed to setup recording-shortcut-pressed listener', {
          error,
        })
      })

    return () => {
      isMounted = false
      if (unlisten) {
        unlisten()
      }
    }
  }, [onShortcutPressed])
}
