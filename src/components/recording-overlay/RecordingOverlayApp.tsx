import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { commands, type CyranoError } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { useUIStore } from '@/store/ui-store'
import { RecordingOverlay } from './RecordingOverlay'

/**
 * Apply theme from localStorage to document.
 * Follows the same pattern as QuickPaneApp.tsx for consistency.
 */
function applyTheme() {
  const theme = localStorage.getItem('ui-theme') || 'system'
  const root = document.documentElement

  root.classList.remove('light', 'dark')

  if (theme === 'system') {
    const systemTheme = window.matchMedia('(prefers-color-scheme: dark)')
      .matches
      ? 'dark'
      : 'light'
    root.classList.add(systemTheme)
  } else {
    root.classList.add(theme)
  }
}

/**
 * RecordingOverlayApp - Root component for the recording overlay window.
 *
 * This component handles:
 * - Theme synchronization with the main window
 * - Re-applying theme when window gains focus
 * - Rendering the recording overlay content
 */
export default function RecordingOverlayApp() {
  // Apply theme on mount and listen for theme changes from main window
  useEffect(() => {
    applyTheme()

    const unlisten = listen('theme-changed', () => {
      applyTheme()
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  // Report render time on first paint for NFR3 tracking
  useEffect(() => {
    const handle = requestAnimationFrame(() => {
      commands.reportRecordingOverlayRendered().then(result => {
        if (result.status === 'error') {
          logger.warn('Failed to report recording overlay render', {
            error: result.error,
          })
        }
      })
    })

    return () => {
      cancelAnimationFrame(handle)
    }
  }, [])

  // Re-apply theme when window becomes visible/focused
  useEffect(() => {
    const currentWindow = getCurrentWindow()
    const unlisten = currentWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        // Re-apply theme in case it changed while hidden
        applyTheme()
      }
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  // Listen for recording events to update overlay state
  // Note: The overlay window has its own Zustand store instance,
  // so we need to listen for events directly here
  useEffect(() => {
    const unlisteners: (() => void)[] = []

    // Listen for recording-failed to show error state
    listen<{ error: CyranoError }>('recording-failed', event => {
      logger.info('Recording overlay received recording-failed event', {
        error: event.payload.error,
      })
      const { setRecordingError } = useUIStore.getState()
      setRecordingError(event.payload.error)
    })
      .then(unlisten => unlisteners.push(unlisten))
      .catch(error => {
        logger.error('Failed to setup recording-failed listener in overlay', {
          error,
        })
      })

    // Listen for recording-state-changed to update state
    listen<{ state: string }>('recording-state-changed', event => {
      const { setRecordingState } = useUIStore.getState()
      setRecordingState(
        event.payload.state.toLowerCase() as
          | 'idle'
          | 'recording'
          | 'transcribing'
          | 'done'
          | 'error'
      )
    })
      .then(unlisten => unlisteners.push(unlisten))
      .catch(error => {
        logger.error(
          'Failed to setup recording-state-changed listener in overlay',
          {
            error,
          }
        )
      })

    return () => {
      unlisteners.forEach(unlisten => unlisten())
    }
  }, [])

  return <RecordingOverlay />
}
