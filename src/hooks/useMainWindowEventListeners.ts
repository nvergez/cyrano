import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useCommandContext } from './use-command-context'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'
import { useUIStore, type RecordingState } from '@/store/ui-store'
import { logger } from '@/lib/logger'
import type { CyranoError } from '@/lib/tauri-bindings'

/** Payload for recording-started event */
interface RecordingStartedPayload {
  timestamp: number
}

/** Payload for recording-stopped event */
interface RecordingStoppedPayload {
  duration_ms: number
  sample_count: number
}

/** Payload for recording-failed event */
interface RecordingFailedPayload {
  error: CyranoError
}

/**
 * Main window event listeners - handles global keyboard shortcuts and cross-window events.
 *
 * This hook composes specialized hooks for different event types:
 * - useKeyboardShortcuts: Global keyboard shortcuts (Cmd+, Cmd+1, Cmd+2)
 * - Quick pane submit listener: Cross-window communication from quick pane
 */
export function useMainWindowEventListeners() {
  const commandContext = useCommandContext()

  useKeyboardShortcuts(commandContext)

  // Listen for quick pane submissions (cross-window event)
  useEffect(() => {
    let isMounted = true
    let unlisten: (() => void) | null = null

    listen<{ text: string }>('quick-pane-submit', event => {
      logger.debug('Quick pane submit event received', {
        text: event.payload.text,
      })
      const { setLastQuickPaneEntry } = useUIStore.getState()
      setLastQuickPaneEntry(event.payload.text)
    })
      .then(unlistenFn => {
        if (!isMounted) {
          unlistenFn()
        } else {
          unlisten = unlistenFn
        }
      })
      .catch(error => {
        logger.error('Failed to setup quick-pane-submit listener', { error })
      })

    return () => {
      isMounted = false
      if (unlisten) {
        unlisten()
      }
    }
  }, [])

  // Listen for recording overlay/state events (cross-window)
  useEffect(() => {
    let isMounted = true
    const unlistenFns: (() => void)[] = []

    const safeSetRecordingState = (state: string) => {
      const normalized = state.toLowerCase() as RecordingState
      const { setRecordingState } = useUIStore.getState()
      setRecordingState(normalized)
    }

    listen('recording-overlay-shown', () => {
      if (!isMounted) return
      const { setRecordingOverlayVisible, setRecordingState } =
        useUIStore.getState()
      setRecordingOverlayVisible(true)
      setRecordingState('recording')
    })
      .then(unlisten => unlistenFns.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-overlay-shown listener', {
          error,
        })
      })

    listen('recording-overlay-dismissed', () => {
      if (!isMounted) return
      const { setRecordingOverlayVisible, setRecordingState } =
        useUIStore.getState()
      setRecordingOverlayVisible(false)
      setRecordingState('idle')
    })
      .then(unlisten => unlistenFns.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-overlay-dismissed listener', {
          error,
        })
      })

    listen('recording-cancelled', () => {
      if (!isMounted) return
      const { setRecordingOverlayVisible, setRecordingState } =
        useUIStore.getState()
      setRecordingOverlayVisible(false)
      setRecordingState('idle')
    })
      .then(unlisten => unlistenFns.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-cancelled listener', { error })
      })

    listen<{ state: string }>('recording-state-changed', event => {
      if (!isMounted) return
      safeSetRecordingState(event.payload.state)
    })
      .then(unlisten => unlistenFns.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-state-changed listener', {
          error,
        })
      })

    // Listen for recording-started event
    listen<RecordingStartedPayload>('recording-started', event => {
      if (!isMounted) return
      logger.info('Recording started', { timestamp: event.payload.timestamp })
      const { setRecordingState, clearRecordingError } = useUIStore.getState()
      clearRecordingError()
      setRecordingState('recording')
    })
      .then(unlisten => unlistenFns.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-started listener', { error })
      })

    // Listen for recording-stopped event
    listen<RecordingStoppedPayload>('recording-stopped', event => {
      if (!isMounted) return
      logger.info('Recording stopped', {
        durationMs: event.payload.duration_ms,
        sampleCount: event.payload.sample_count,
      })
      const { setRecordingState } = useUIStore.getState()
      setRecordingState('transcribing')
    })
      .then(unlisten => unlistenFns.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-stopped listener', { error })
      })

    // Listen for recording-failed event
    listen<RecordingFailedPayload>('recording-failed', event => {
      if (!isMounted) return
      logger.error('Recording failed', { error: event.payload.error })
      const { setRecordingError, setRecordingOverlayVisible } =
        useUIStore.getState()
      setRecordingError(event.payload.error)
      setRecordingOverlayVisible(true)
    })
      .then(unlisten => unlistenFns.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-failed listener', { error })
      })

    return () => {
      isMounted = false
      unlistenFns.forEach(unlisten => unlisten())
    }
  }, [])
}
