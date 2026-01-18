import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { RecordingOverlay } from './RecordingOverlay'
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
})
