import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, act } from '@testing-library/react'
import {
  RecordingOverlay,
  AUTO_DISMISS_SUCCESS_MS,
  AUTO_DISMISS_ERROR_MS,
} from './RecordingOverlay'
import { useUIStore } from '@/store/ui-store'

// Mock the Tauri bindings
vi.mock('@/lib/tauri-bindings', () => ({
  commands: {
    dismissRecordingOverlay: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: null }),
    cancelRecording: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    openMicrophoneSettings: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: null }),
    cancelTranscription: vi.fn(),
  },
}))

// Mock the logger
vi.mock('@/lib/logger', () => ({
  logger: {
    info: vi.fn(),
    error: vi.fn(),
    debug: vi.fn(),
    warn: vi.fn(),
  },
}))

describe('RecordingOverlay', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Reset store to default state
    useUIStore.setState({
      recordingState: 'idle',
      recordingError: null,
    })
  })

  it('renders recording indicator by default', () => {
    useUIStore.setState({ recordingState: 'recording' })
    render(<RecordingOverlay />)

    expect(screen.getByText('Recording...')).toBeInTheDocument()
  })

  it('renders transcribing indicator when state is transcribing', () => {
    useUIStore.setState({ recordingState: 'transcribing' })
    render(<RecordingOverlay />)

    expect(screen.getByText('Transcribing...')).toBeInTheDocument()
  })

  it('renders error indicator when recording state is error', () => {
    useUIStore.setState({
      recordingState: 'error',
      recordingError: 'MicAccessDenied',
    })
    render(<RecordingOverlay />)

    expect(screen.getByText('Microphone access denied')).toBeInTheDocument()
  })

  it('renders error indicator when recordingError is set', () => {
    useUIStore.setState({
      recordingState: 'recording',
      recordingError: { RecordingFailed: { reason: 'test error' } },
    })
    render(<RecordingOverlay />)

    expect(screen.getByText('Recording failed')).toBeInTheDocument()
  })

  it('shows spinner with animate-spin class in transcribing state', () => {
    useUIStore.setState({ recordingState: 'transcribing' })
    render(<RecordingOverlay />)

    const svg = document.querySelector('svg')
    expect(svg).toHaveClass('animate-spin')
  })

  it('renders success indicator when state is done', () => {
    useUIStore.setState({ recordingState: 'done' })
    render(<RecordingOverlay />)

    expect(screen.getByText('Done')).toBeInTheDocument()
  })

  it('shows green checkmark icon in done state', () => {
    useUIStore.setState({ recordingState: 'done' })
    render(<RecordingOverlay />)

    const svg = document.querySelector('svg')
    expect(svg).toBeInTheDocument()
    expect(svg).toHaveClass('text-green-500')
  })

  it('calls dismissRecordingOverlay when clicking in done state', async () => {
    const { commands } = await import('@/lib/tauri-bindings')
    useUIStore.setState({ recordingState: 'done' })
    const { container } = render(<RecordingOverlay />)

    const overlay = container.firstChild as HTMLElement
    fireEvent.click(overlay)

    expect(commands.dismissRecordingOverlay).toHaveBeenCalled()
  })

  describe('auto-dismiss behavior', () => {
    beforeEach(() => {
      vi.useFakeTimers()
    })

    afterEach(() => {
      vi.useRealTimers()
    })

    it('exports auto-dismiss delay constants', () => {
      expect(AUTO_DISMISS_SUCCESS_MS).toBe(1200)
      expect(AUTO_DISMISS_ERROR_MS).toBe(1800)
    })

    it('auto-dismisses after success state delay', async () => {
      const { commands } = await import('@/lib/tauri-bindings')
      useUIStore.setState({ recordingState: 'done', recordingError: null })
      render(<RecordingOverlay />)

      // Should not have dismissed immediately
      expect(commands.dismissRecordingOverlay).not.toHaveBeenCalled()

      // Fast-forward time past the success dismiss delay
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_SUCCESS_MS)
      })

      // Verify dismiss was called
      expect(commands.dismissRecordingOverlay).toHaveBeenCalled()
    })

    it('auto-dismisses after error state with longer delay', async () => {
      const { commands } = await import('@/lib/tauri-bindings')
      useUIStore.setState({
        recordingState: 'error',
        recordingError: 'MicAccessDenied',
      })
      render(<RecordingOverlay />)

      // Should NOT have dismissed after success delay
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_SUCCESS_MS)
      })
      expect(commands.dismissRecordingOverlay).not.toHaveBeenCalled()

      // Should dismiss after error delay
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_ERROR_MS - AUTO_DISMISS_SUCCESS_MS)
      })
      expect(commands.dismissRecordingOverlay).toHaveBeenCalled()
    })

    it('clicking cancels auto-dismiss timer and prevents double dismiss', async () => {
      const { commands } = await import('@/lib/tauri-bindings')
      useUIStore.setState({ recordingState: 'done', recordingError: null })
      const { container } = render(<RecordingOverlay />)

      // Click before timer fires
      const overlay = container.firstChild as HTMLElement
      await act(async () => {
        fireEvent.click(overlay)
      })

      // Should have been called once from click
      expect(commands.dismissRecordingOverlay).toHaveBeenCalledTimes(1)

      // Advance past timer - should not trigger second dismiss
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_SUCCESS_MS + 1000)
      })

      // Should still only have been called once (from click)
      expect(commands.dismissRecordingOverlay).toHaveBeenCalledTimes(1)
    })

    it('cleans up timer on unmount', async () => {
      const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout')

      useUIStore.setState({ recordingState: 'done', recordingError: null })
      const { unmount } = render(<RecordingOverlay />)

      unmount()

      // Verify clearTimeout was called during cleanup
      expect(clearTimeoutSpy).toHaveBeenCalled()
      clearTimeoutSpy.mockRestore()
    })

    it('does not auto-dismiss during recording state', async () => {
      const { commands } = await import('@/lib/tauri-bindings')
      useUIStore.setState({ recordingState: 'recording', recordingError: null })
      render(<RecordingOverlay />)

      // Advance past both delays
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_ERROR_MS + 1000)
      })

      // Should not have auto-dismissed
      expect(commands.dismissRecordingOverlay).not.toHaveBeenCalled()
    })

    it('does not auto-dismiss during transcribing state', async () => {
      const { commands } = await import('@/lib/tauri-bindings')
      useUIStore.setState({
        recordingState: 'transcribing',
        recordingError: null,
      })
      render(<RecordingOverlay />)

      // Advance past both delays
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_ERROR_MS + 1000)
      })

      // Should not have auto-dismissed
      expect(commands.dismissRecordingOverlay).not.toHaveBeenCalled()
    })

    it('resets state to idle after auto-dismiss', async () => {
      useUIStore.setState({ recordingState: 'done', recordingError: null })
      render(<RecordingOverlay />)

      // Fast-forward time past the success dismiss delay
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_SUCCESS_MS)
      })

      // Verify state was reset
      expect(useUIStore.getState().recordingState).toBe('idle')
    })

    it('clears error after auto-dismiss from error state', async () => {
      useUIStore.setState({
        recordingState: 'error',
        recordingError: 'MicAccessDenied',
      })
      render(<RecordingOverlay />)

      // Fast-forward time past the error dismiss delay
      await act(async () => {
        vi.advanceTimersByTime(AUTO_DISMISS_ERROR_MS)
      })

      // Verify error was cleared
      expect(useUIStore.getState().recordingError).toBeNull()
      expect(useUIStore.getState().recordingState).toBe('idle')
    })
  })
})
