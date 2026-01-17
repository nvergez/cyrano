import { create } from 'zustand'
import { devtools } from 'zustand/middleware'

import type { CyranoError } from '@/lib/tauri-bindings'

/** Recording workflow states (mirrors Rust RecordingState enum) */
export type RecordingState =
  | 'idle'
  | 'recording'
  | 'transcribing'
  | 'done'
  | 'error'

interface UIState {
  leftSidebarVisible: boolean
  rightSidebarVisible: boolean
  commandPaletteOpen: boolean
  preferencesOpen: boolean
  lastQuickPaneEntry: string | null
  recordingOverlayVisible: boolean
  recordingState: RecordingState
  recordingError: CyranoError | null

  toggleLeftSidebar: () => void
  setLeftSidebarVisible: (visible: boolean) => void
  toggleRightSidebar: () => void
  setRightSidebarVisible: (visible: boolean) => void
  toggleCommandPalette: () => void
  setCommandPaletteOpen: (open: boolean) => void
  togglePreferences: () => void
  setPreferencesOpen: (open: boolean) => void
  setLastQuickPaneEntry: (text: string) => void
  setRecordingOverlayVisible: (visible: boolean) => void
  setRecordingState: (state: RecordingState) => void
  setRecordingError: (error: CyranoError | null) => void
  clearRecordingError: () => void
}

export const useUIStore = create<UIState>()(
  devtools(
    set => ({
      leftSidebarVisible: true,
      rightSidebarVisible: true,
      commandPaletteOpen: false,
      preferencesOpen: false,
      lastQuickPaneEntry: null,
      recordingOverlayVisible: false,
      recordingState: 'idle' as RecordingState,
      recordingError: null,

      toggleLeftSidebar: () =>
        set(
          state => ({ leftSidebarVisible: !state.leftSidebarVisible }),
          undefined,
          'toggleLeftSidebar'
        ),

      setLeftSidebarVisible: visible =>
        set(
          { leftSidebarVisible: visible },
          undefined,
          'setLeftSidebarVisible'
        ),

      toggleRightSidebar: () =>
        set(
          state => ({ rightSidebarVisible: !state.rightSidebarVisible }),
          undefined,
          'toggleRightSidebar'
        ),

      setRightSidebarVisible: visible =>
        set(
          { rightSidebarVisible: visible },
          undefined,
          'setRightSidebarVisible'
        ),

      toggleCommandPalette: () =>
        set(
          state => ({ commandPaletteOpen: !state.commandPaletteOpen }),
          undefined,
          'toggleCommandPalette'
        ),

      setCommandPaletteOpen: open =>
        set({ commandPaletteOpen: open }, undefined, 'setCommandPaletteOpen'),

      togglePreferences: () =>
        set(
          state => ({ preferencesOpen: !state.preferencesOpen }),
          undefined,
          'togglePreferences'
        ),

      setPreferencesOpen: open =>
        set({ preferencesOpen: open }, undefined, 'setPreferencesOpen'),

      setLastQuickPaneEntry: text =>
        set({ lastQuickPaneEntry: text }, undefined, 'setLastQuickPaneEntry'),

      setRecordingOverlayVisible: visible =>
        set(
          { recordingOverlayVisible: visible },
          undefined,
          'setRecordingOverlayVisible'
        ),

      setRecordingState: state =>
        set({ recordingState: state }, undefined, 'setRecordingState'),

      setRecordingError: error =>
        set(
          { recordingError: error, recordingState: 'error' },
          undefined,
          'setRecordingError'
        ),

      clearRecordingError: () =>
        set({ recordingError: null }, undefined, 'clearRecordingError'),
    }),
    {
      name: 'ui-store',
    }
  )
)
