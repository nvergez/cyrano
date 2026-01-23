import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'

// Mock the bindings module
const mockCommands = {
  checkAccessibilityPermission: vi.fn(),
  requestAccessibilityPermission: vi.fn(),
  openAccessibilitySettings: vi.fn(),
}

vi.mock('@/lib/bindings', () => ({
  commands: mockCommands,
}))

// Mock the logger
vi.mock('@/lib/logger', () => ({
  logger: {
    warn: vi.fn(),
    error: vi.fn(),
    debug: vi.fn(),
    info: vi.fn(),
  },
}))

// Import after mocks are set up
const { useAccessibilityPermission } =
  await import('./useAccessibilityPermission')

describe('useAccessibilityPermission', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Default mock: permission not determined
    mockCommands.checkAccessibilityPermission.mockResolvedValue('NotDetermined')
  })

  describe('initial state', () => {
    it('starts with NotDetermined status and isChecking true', async () => {
      const { result } = renderHook(() => useAccessibilityPermission())

      // Initially checking
      expect(result.current.isChecking).toBe(true)

      // Wait for initial check to complete
      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      expect(result.current.status).toBe('NotDetermined')
      expect(result.current.isGranted).toBe(false)
    })

    it('checks permission on mount', async () => {
      renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(mockCommands.checkAccessibilityPermission).toHaveBeenCalledTimes(
          1
        )
      })
    })
  })

  describe('checkPermission', () => {
    it('updates status to Granted when permission is granted', async () => {
      mockCommands.checkAccessibilityPermission.mockResolvedValue('Granted')

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.status).toBe('Granted')
      })

      expect(result.current.isGranted).toBe(true)
    })

    it('updates status to Denied when permission is denied', async () => {
      mockCommands.checkAccessibilityPermission.mockResolvedValue('Denied')

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.status).toBe('Denied')
      })

      expect(result.current.isGranted).toBe(false)
    })

    it('handles check permission errors gracefully', async () => {
      mockCommands.checkAccessibilityPermission.mockRejectedValue(
        new Error('Check failed')
      )

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      // Should default to NotDetermined on error
      expect(result.current.status).toBe('NotDetermined')
    })
  })

  describe('requestPermission', () => {
    it('returns true when permission is granted', async () => {
      mockCommands.checkAccessibilityPermission.mockResolvedValue(
        'NotDetermined'
      )
      mockCommands.requestAccessibilityPermission.mockResolvedValue({
        status: 'ok',
        data: true,
      })

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      // Update mock for recheck after request
      mockCommands.checkAccessibilityPermission.mockResolvedValue('Granted')

      let requestResult: boolean | undefined
      await act(async () => {
        requestResult = await result.current.requestPermission()
      })

      expect(requestResult).toBe(true)
      expect(mockCommands.requestAccessibilityPermission).toHaveBeenCalled()
    })

    it('returns false when permission is denied', async () => {
      mockCommands.requestAccessibilityPermission.mockResolvedValue({
        status: 'ok',
        data: false,
      })

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      let requestResult: boolean | undefined
      await act(async () => {
        requestResult = await result.current.requestPermission()
      })

      expect(requestResult).toBe(false)
    })

    it('returns false on request error', async () => {
      mockCommands.requestAccessibilityPermission.mockResolvedValue({
        status: 'error',
        error: 'Request failed',
      })

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      let requestResult: boolean | undefined
      await act(async () => {
        requestResult = await result.current.requestPermission()
      })

      expect(requestResult).toBe(false)
    })

    it('rechecks permission after successful request', async () => {
      mockCommands.checkAccessibilityPermission.mockResolvedValue(
        'NotDetermined'
      )
      mockCommands.requestAccessibilityPermission.mockResolvedValue({
        status: 'ok',
        data: true,
      })

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      // Clear call count from initial mount
      mockCommands.checkAccessibilityPermission.mockClear()
      mockCommands.checkAccessibilityPermission.mockResolvedValue('Granted')

      await act(async () => {
        await result.current.requestPermission()
      })

      // Should have rechecked permission
      expect(mockCommands.checkAccessibilityPermission).toHaveBeenCalled()
    })
  })

  describe('openSettings', () => {
    it('calls openAccessibilitySettings command', async () => {
      mockCommands.openAccessibilitySettings.mockResolvedValue({ status: 'ok' })

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      await act(async () => {
        await result.current.openSettings()
      })

      expect(mockCommands.openAccessibilitySettings).toHaveBeenCalled()
    })

    it('handles openSettings errors gracefully', async () => {
      mockCommands.openAccessibilitySettings.mockResolvedValue({
        status: 'error',
        error: 'Failed to open',
      })

      const { result } = renderHook(() => useAccessibilityPermission())

      await waitFor(() => {
        expect(result.current.isChecking).toBe(false)
      })

      // Should not throw
      await act(async () => {
        await result.current.openSettings()
      })

      expect(mockCommands.openAccessibilitySettings).toHaveBeenCalled()
    })
  })
})
