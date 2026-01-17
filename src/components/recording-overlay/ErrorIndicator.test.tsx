import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ErrorIndicator } from './ErrorIndicator'

// Mock the Tauri bindings
vi.mock('@/lib/tauri-bindings', () => ({
  commands: {
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
  },
}))

describe('ErrorIndicator', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders mic access denied error correctly', () => {
    render(<ErrorIndicator error="MicAccessDenied" />)

    expect(screen.getByText('Microphone access denied')).toBeInTheDocument()
    expect(screen.getByText('Open Settings')).toBeInTheDocument()
  })

  it('renders generic recording failed error', () => {
    render(
      <ErrorIndicator error={{ RecordingFailed: { reason: 'test error' } }} />
    )

    expect(screen.getByText('Recording failed')).toBeInTheDocument()
    // Should not show Open Settings button for generic errors
    expect(screen.queryByText('Open Settings')).not.toBeInTheDocument()
  })

  it('renders error state with null error', () => {
    render(<ErrorIndicator error={null} />)

    expect(screen.getByText('Recording failed')).toBeInTheDocument()
  })

  it('calls openMicrophoneSettings when clicking the link', async () => {
    const { commands } = await import('@/lib/tauri-bindings')

    render(<ErrorIndicator error="MicAccessDenied" />)

    const settingsLink = screen.getByText('Open Settings')
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
})
