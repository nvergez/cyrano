import { render, screen } from '@/test/test-utils'
import { describe, it, expect, vi } from 'vitest'
import App from './App'

// Tauri bindings are mocked globally in src/test/setup.ts

vi.mock('@/lib/commands', async () => {
  const actual = await vi.importActual('@/lib/commands')
  return {
    ...(actual as object),
    initializeCommandSystem: vi.fn(),
  }
})

describe('App', () => {
  it('renders main window layout', async () => {
    render(<App />)
    expect(
      await screen.findByRole('heading', { name: /hello world/i })
    ).toBeInTheDocument()
  })

  it('renders title bar with traffic light buttons', async () => {
    render(<App />)
    // Find specifically the window control buttons in the title bar
    const titleBarButtons = (await screen.findAllByRole('button')).filter(
      button =>
        button.getAttribute('aria-label')?.includes('window') ||
        button.className.includes('window-control')
    )
    // Should have at least the window control buttons
    expect(titleBarButtons.length).toBeGreaterThan(0)
  })
})
