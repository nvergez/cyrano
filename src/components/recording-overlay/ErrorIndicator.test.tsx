import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ErrorIndicator } from './ErrorIndicator'

// Mock the Tauri bindings
vi.mock('@/lib/tauri-bindings', () => ({
  commands: {
    openMicrophoneSettings: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: null }),
    openModelDirectory: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
}))

// Mock the logger
vi.mock('@/lib/logger', () => ({
  logger: {
    info: vi.fn(),
    error: vi.fn(),
  },
}))

describe('ErrorIndicator', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders mic access denied error correctly', () => {
    render(<ErrorIndicator error="MicAccessDenied" />)

    expect(screen.getByText('Microphone access denied')).toBeInTheDocument()
    expect(screen.getByText('Open System Preferences')).toBeInTheDocument()
  })

  it('renders generic recording failed error', () => {
    render(
      <ErrorIndicator error={{ RecordingFailed: { reason: 'test error' } }} />
    )

    expect(screen.getByText('Recording failed')).toBeInTheDocument()
    expect(screen.getByText('test error')).toBeInTheDocument()
    // Should not show Open Settings or Model Directory button for generic errors
    expect(
      screen.queryByText('Open System Preferences')
    ).not.toBeInTheDocument()
    expect(screen.queryByText('Open Model Directory')).not.toBeInTheDocument()
  })

  it('renders error state with null error', () => {
    render(<ErrorIndicator error={null} />)

    expect(screen.getByText('Unknown error')).toBeInTheDocument()
  })

  it('calls openMicrophoneSettings when clicking the link', async () => {
    const { commands } = await import('@/lib/tauri-bindings')

    render(<ErrorIndicator error="MicAccessDenied" />)

    const settingsLink = screen.getByText('Open System Preferences')
    fireEvent.click(settingsLink)

    expect(commands.openMicrophoneSettings).toHaveBeenCalled()
  })

  it('renders a red X icon', () => {
    render(<ErrorIndicator error="MicAccessDenied" />)

    // Check for SVG element (red X icon)
    const svg = document.querySelector('svg')
    expect(svg).toBeInTheDocument()
    expect(svg).toHaveClass('text-destructive')
  })

  // Model error tests (Story 2.1)
  it('renders ModelNotFound error correctly', () => {
    render(
      <ErrorIndicator
        error={{ ModelNotFound: { path: '/Users/test/.cyrano/models/' } }}
      />
    )

    expect(screen.getByText('Model not found')).toBeInTheDocument()
    expect(screen.getByText('/Users/test/.cyrano/models/')).toBeInTheDocument()
    expect(screen.getByText('Open Model Directory')).toBeInTheDocument()
    expect(
      screen.queryByText('Open System Preferences')
    ).not.toBeInTheDocument()
  })

  it('renders ModelLoadFailed error correctly', () => {
    render(
      <ErrorIndicator
        error={{ ModelLoadFailed: { reason: 'corrupted model file' } }}
      />
    )

    expect(screen.getByText('Failed to load model')).toBeInTheDocument()
    expect(screen.getByText('corrupted model file')).toBeInTheDocument()
    expect(screen.getByText('Open Model Directory')).toBeInTheDocument()
  })

  it('calls openModelDirectory when clicking the link for model errors', async () => {
    const { commands } = await import('@/lib/tauri-bindings')

    render(<ErrorIndicator error={{ ModelNotFound: { path: '/test/path' } }} />)

    const modelLink = screen.getByText('Open Model Directory')
    fireEvent.click(modelLink)

    expect(commands.openModelDirectory).toHaveBeenCalled()
  })

  it('renders TranscriptionFailed error correctly', () => {
    render(
      <ErrorIndicator
        error={{ TranscriptionFailed: { reason: 'invalid audio format' } }}
      />
    )

    expect(screen.getByText('Transcription failed')).toBeInTheDocument()
    expect(screen.getByText('invalid audio format')).toBeInTheDocument()
    // No special action link for transcription errors
    expect(screen.queryByText('Open Model Directory')).not.toBeInTheDocument()
    expect(
      screen.queryByText('Open System Preferences')
    ).not.toBeInTheDocument()
  })
})
