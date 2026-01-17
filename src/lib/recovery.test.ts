import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock logger
vi.mock('@/lib/logger', () => ({
  logger: {
    debug: vi.fn(),
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}))

// Mock tauri-bindings
const mockCommands = {
  saveEmergencyData: vi.fn(),
  loadEmergencyData: vi.fn(),
  cleanupOldRecoveryFiles: vi.fn(),
}

vi.mock('@/lib/tauri-bindings', () => ({
  commands: mockCommands,
}))

describe('recovery', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('saveEmergencyData', () => {
    describe('[P1] successful save', () => {
      it('saves data successfully', async () => {
        // GIVEN: Valid filename and data
        mockCommands.saveEmergencyData.mockResolvedValue({
          status: 'ok',
          data: null,
        })
        const { saveEmergencyData } = await import('./recovery')

        // WHEN: Saving emergency data
        await saveEmergencyData('test-file', { foo: 'bar' })

        // THEN: Calls backend with correct params
        expect(mockCommands.saveEmergencyData).toHaveBeenCalledWith(
          'test-file',
          { foo: 'bar' }
        )
      })

      it('handles silent option', async () => {
        // GIVEN: Silent option enabled
        mockCommands.saveEmergencyData.mockResolvedValue({
          status: 'ok',
          data: null,
        })
        const { saveEmergencyData } = await import('./recovery')
        const { logger } = await import('@/lib/logger')

        // WHEN: Saving with silent option
        await saveEmergencyData('test-file', { data: 'test' }, { silent: true })

        // THEN: Does not log info message
        expect(logger.info).not.toHaveBeenCalledWith(
          'Emergency data saved successfully',
          expect.anything()
        )
      })
    })

    describe('[P1] error handling', () => {
      it('throws error on validation error', async () => {
        // GIVEN: Backend returns validation error
        mockCommands.saveEmergencyData.mockResolvedValue({
          status: 'error',
          error: { type: 'ValidationError', message: 'Invalid filename' },
        })
        const { saveEmergencyData } = await import('./recovery')

        // WHEN/THEN: Save throws with formatted message
        await expect(
          saveEmergencyData('invalid', { data: 'test' })
        ).rejects.toThrow('Validation error: Invalid filename')
      })

      it('throws error on data too large', async () => {
        // GIVEN: Backend returns data too large error
        mockCommands.saveEmergencyData.mockResolvedValue({
          status: 'error',
          error: { type: 'DataTooLarge', max_bytes: 1024 },
        })
        const { saveEmergencyData } = await import('./recovery')

        // WHEN/THEN: Save throws with size info
        await expect(
          saveEmergencyData('large-file', { huge: 'data' })
        ).rejects.toThrow('Data too large (max 1024 bytes)')
      })

      it('throws error on IO error', async () => {
        // GIVEN: Backend returns IO error
        mockCommands.saveEmergencyData.mockResolvedValue({
          status: 'error',
          error: { type: 'IoError', message: 'Disk full' },
        })
        const { saveEmergencyData } = await import('./recovery')

        // WHEN/THEN: Save throws with IO error message
        await expect(saveEmergencyData('file', {})).rejects.toThrow(
          'IO error: Disk full'
        )
      })
    })
  })

  describe('loadEmergencyData', () => {
    describe('[P1] successful load', () => {
      it('loads data successfully', async () => {
        // GIVEN: Backend returns data
        const testData = { saved: 'data', timestamp: 12345 }
        mockCommands.loadEmergencyData.mockResolvedValue({
          status: 'ok',
          data: testData,
        })
        const { loadEmergencyData } = await import('./recovery')

        // WHEN: Loading emergency data
        const result = await loadEmergencyData('test-file')

        // THEN: Returns the data
        expect(result).toEqual(testData)
        expect(mockCommands.loadEmergencyData).toHaveBeenCalledWith('test-file')
      })

      it('returns typed data', async () => {
        // GIVEN: Backend returns typed data
        interface TestData {
          content: string
          version: number
        }
        mockCommands.loadEmergencyData.mockResolvedValue({
          status: 'ok',
          data: { content: 'hello', version: 1 },
        })
        const { loadEmergencyData } = await import('./recovery')

        // WHEN: Loading with type parameter
        const result = await loadEmergencyData<TestData>('typed-file')

        // THEN: Returns correctly typed data
        expect(result?.content).toBe('hello')
        expect(result?.version).toBe(1)
      })
    })

    describe('[P1] file not found handling', () => {
      it('returns null when file not found', async () => {
        // GIVEN: Backend returns file not found error
        mockCommands.loadEmergencyData.mockResolvedValue({
          status: 'error',
          error: { type: 'FileNotFound' },
        })
        const { loadEmergencyData } = await import('./recovery')

        // WHEN: Loading non-existent file
        const result = await loadEmergencyData('non-existent')

        // THEN: Returns null (not throwing)
        expect(result).toBeNull()
      })
    })

    describe('[P1] error handling', () => {
      it('throws error on parse error', async () => {
        // GIVEN: Backend returns parse error
        mockCommands.loadEmergencyData.mockResolvedValue({
          status: 'error',
          error: { type: 'ParseError', message: 'Invalid JSON' },
        })
        const { loadEmergencyData } = await import('./recovery')

        // WHEN/THEN: Load throws with parse error message
        await expect(loadEmergencyData('corrupt-file')).rejects.toThrow(
          'Parse error: Invalid JSON'
        )
      })

      it('throws error on IO error', async () => {
        // GIVEN: Backend returns IO error
        mockCommands.loadEmergencyData.mockResolvedValue({
          status: 'error',
          error: { type: 'IoError', message: 'Permission denied' },
        })
        const { loadEmergencyData } = await import('./recovery')

        // WHEN/THEN: Load throws with IO error message
        await expect(loadEmergencyData('protected-file')).rejects.toThrow(
          'IO error: Permission denied'
        )
      })
    })
  })

  describe('cleanupOldFiles', () => {
    describe('[P1] successful cleanup', () => {
      it('returns count of removed files', async () => {
        // GIVEN: Backend returns cleanup count
        mockCommands.cleanupOldRecoveryFiles.mockResolvedValue({
          status: 'ok',
          data: 5,
        })
        const { cleanupOldFiles } = await import('./recovery')

        // WHEN: Running cleanup
        const result = await cleanupOldFiles()

        // THEN: Returns the count
        expect(result).toBe(5)
      })

      it('returns zero when no files to clean', async () => {
        // GIVEN: No files to clean
        mockCommands.cleanupOldRecoveryFiles.mockResolvedValue({
          status: 'ok',
          data: 0,
        })
        const { cleanupOldFiles } = await import('./recovery')

        // WHEN: Running cleanup
        const result = await cleanupOldFiles()

        // THEN: Returns zero
        expect(result).toBe(0)
      })
    })

    describe('[P1] error handling', () => {
      it('throws error on failure', async () => {
        // GIVEN: Backend returns error
        mockCommands.cleanupOldRecoveryFiles.mockResolvedValue({
          status: 'error',
          error: { type: 'IoError', message: 'Cannot access directory' },
        })
        const { cleanupOldFiles } = await import('./recovery')

        // WHEN/THEN: Cleanup throws
        await expect(cleanupOldFiles()).rejects.toThrow(
          'IO error: Cannot access directory'
        )
      })
    })
  })

  describe('saveCrashState', () => {
    describe('[P1] crash state saving', () => {
      it('saves crash state with metadata', async () => {
        // GIVEN: App state and crash info
        mockCommands.saveEmergencyData.mockResolvedValue({
          status: 'ok',
          data: null,
        })
        const { saveCrashState } = await import('./recovery')
        const appState = { currentPage: '/dashboard' }
        const crashInfo = { error: 'Test error', stack: 'stack trace' }

        // WHEN: Saving crash state
        await saveCrashState(appState, crashInfo)

        // THEN: Saves with crash- prefix and metadata
        expect(mockCommands.saveEmergencyData).toHaveBeenCalledWith(
          expect.stringMatching(/^crash-\d+$/),
          expect.objectContaining({
            state: appState,
            crashInfo,
            timestamp: expect.any(Number),
            userAgent: expect.any(String),
            url: expect.any(String),
          })
        )
      })

      it('saves crash state without crash info', async () => {
        // GIVEN: App state only
        mockCommands.saveEmergencyData.mockResolvedValue({
          status: 'ok',
          data: null,
        })
        const { saveCrashState } = await import('./recovery')
        const appState = { data: 'important' }

        // WHEN: Saving crash state without crash info
        await saveCrashState(appState)

        // THEN: Saves with undefined crashInfo
        expect(mockCommands.saveEmergencyData).toHaveBeenCalledWith(
          expect.stringMatching(/^crash-\d+$/),
          expect.objectContaining({
            state: appState,
            crashInfo: undefined,
          })
        )
      })
    })

    describe('[P2] error resilience', () => {
      it('does not throw when save fails', async () => {
        // GIVEN: Backend that fails
        mockCommands.saveEmergencyData.mockRejectedValue(
          new Error('Save failed')
        )
        const { saveCrashState } = await import('./recovery')

        // WHEN: Saving crash state that fails
        // THEN: Does not throw (silently fails)
        await expect(saveCrashState({ state: 'test' })).resolves.toBeUndefined()
      })

      it('logs error when save fails', async () => {
        // GIVEN: Backend that fails
        mockCommands.saveEmergencyData.mockRejectedValue(
          new Error('Network error')
        )
        const { saveCrashState } = await import('./recovery')
        const { logger } = await import('@/lib/logger')

        // WHEN: Saving crash state that fails
        await saveCrashState({ state: 'test' })

        // THEN: Logs the error
        expect(logger.error).toHaveBeenCalledWith(
          'Failed to save crash state',
          expect.objectContaining({ error: expect.any(Error) })
        )
      })
    })
  })
})
