import { describe, it, expect, vi, beforeEach } from 'vitest'
import { toast } from 'sonner'

// Mock sonner toast
vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}))

// Mock logger
vi.mock('./logger', () => ({
  logger: {
    debug: vi.fn(),
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}))

// Mock tauri-bindings - needs to be before the import
vi.mock('./tauri-bindings', () => ({
  commands: {
    sendNativeNotification: vi.fn(),
  },
}))

describe('notifications', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('notify', () => {
    describe('[P1] toast notifications', () => {
      it('sends info toast by default', async () => {
        // GIVEN: Title and message
        const { notify } = await import('./notifications')

        // WHEN: Calling notify without type
        await notify('Test Title', 'Test message')

        // THEN: Sends info toast
        expect(toast.info).toHaveBeenCalledWith('Test Title: Test message', {})
      })

      it('sends success toast when type is success', async () => {
        // GIVEN: Success notification type
        const { notify } = await import('./notifications')

        // WHEN: Calling notify with success type
        await notify('Success', 'Operation completed', { type: 'success' })

        // THEN: Sends success toast
        expect(toast.success).toHaveBeenCalledWith(
          'Success: Operation completed',
          {}
        )
      })

      it('sends error toast when type is error', async () => {
        // GIVEN: Error notification type
        const { notify } = await import('./notifications')

        // WHEN: Calling notify with error type
        await notify('Error', 'Something went wrong', { type: 'error' })

        // THEN: Sends error toast
        expect(toast.error).toHaveBeenCalledWith(
          'Error: Something went wrong',
          {}
        )
      })

      it('sends warning toast when type is warning', async () => {
        // GIVEN: Warning notification type
        const { notify } = await import('./notifications')

        // WHEN: Calling notify with warning type
        await notify('Warning', 'Be careful', { type: 'warning' })

        // THEN: Sends warning toast
        expect(toast.warning).toHaveBeenCalledWith('Warning: Be careful', {})
      })

      it('handles title-only notification', async () => {
        // GIVEN: Title without message
        const { notify } = await import('./notifications')

        // WHEN: Calling notify with only title
        await notify('Just a title')

        // THEN: Sends toast with title only
        expect(toast.info).toHaveBeenCalledWith('Just a title', {})
      })

      it('passes duration option to toast', async () => {
        // GIVEN: Custom duration
        const { notify } = await import('./notifications')

        // WHEN: Calling notify with duration
        await notify('Title', 'Message', { duration: 5000 })

        // THEN: Passes duration to toast
        expect(toast.info).toHaveBeenCalledWith('Title: Message', {
          duration: 5000,
        })
      })
    })

    describe('[P1] native notifications', () => {
      it('sends native notification when native option is true', async () => {
        // GIVEN: Native notification option
        const { commands } = await import('./tauri-bindings')
        vi.mocked(commands.sendNativeNotification).mockResolvedValue({
          status: 'ok',
          data: null,
        })
        const { notify } = await import('./notifications')

        // WHEN: Calling notify with native option
        await notify('Native Title', 'Native message', { native: true })

        // THEN: Calls native notification command
        expect(commands.sendNativeNotification).toHaveBeenCalledWith(
          'Native Title',
          'Native message'
        )
        // Should not call toast
        expect(toast.info).not.toHaveBeenCalled()
      })

      it('handles null message for native notification', async () => {
        // GIVEN: Native notification without message
        const { commands } = await import('./tauri-bindings')
        vi.mocked(commands.sendNativeNotification).mockResolvedValue({
          status: 'ok',
          data: null,
        })
        const { notify } = await import('./notifications')

        // WHEN: Calling native notify without message
        await notify('Title Only', undefined, { native: true })

        // THEN: Passes null for message
        expect(commands.sendNativeNotification).toHaveBeenCalledWith(
          'Title Only',
          null
        )
      })

      it('falls back to toast when native notification fails', async () => {
        // GIVEN: Native notification that fails
        const { commands } = await import('./tauri-bindings')
        vi.mocked(commands.sendNativeNotification).mockResolvedValue({
          status: 'error',
          error: 'Permission denied',
        })
        const { notify } = await import('./notifications')

        // WHEN: Calling native notify that fails
        await notify('Failed Native', 'Message', { native: true })

        // THEN: Falls back to error toast
        expect(toast.error).toHaveBeenCalledWith('Failed Native: Message')
      })
    })
  })

  describe('convenience functions', () => {
    describe('[P1] notifications.success', () => {
      it('sends success notification', async () => {
        // GIVEN: Success convenience function
        const { notifications } = await import('./notifications')

        // WHEN: Calling success
        await notifications.success('Done', 'Task completed')

        // THEN: Sends success toast
        expect(toast.success).toHaveBeenCalledWith('Done: Task completed', {})
      })
    })

    describe('[P1] notifications.error', () => {
      it('sends error notification', async () => {
        // GIVEN: Error convenience function
        const { notifications } = await import('./notifications')

        // WHEN: Calling error
        await notifications.error('Failed', 'Something broke')

        // THEN: Sends error toast
        expect(toast.error).toHaveBeenCalledWith('Failed: Something broke', {})
      })
    })

    describe('[P1] notifications.info', () => {
      it('sends info notification', async () => {
        // GIVEN: Info convenience function
        const { notifications } = await import('./notifications')

        // WHEN: Calling info
        await notifications.info('Info', 'FYI')

        // THEN: Sends info toast
        expect(toast.info).toHaveBeenCalledWith('Info: FYI', {})
      })
    })

    describe('[P1] notifications.warning', () => {
      it('sends warning notification', async () => {
        // GIVEN: Warning convenience function
        const { notifications } = await import('./notifications')

        // WHEN: Calling warning
        await notifications.warning('Caution', 'Be careful')

        // THEN: Sends warning toast
        expect(toast.warning).toHaveBeenCalledWith('Caution: Be careful', {})
      })
    })

    describe('[P2] exported shorthand functions', () => {
      it('exports success shorthand', async () => {
        // GIVEN: Exported success function
        const { success } = await import('./notifications')

        // WHEN: Calling shorthand
        await success('Quick', 'Test')

        // THEN: Works same as notifications.success
        expect(toast.success).toHaveBeenCalled()
      })

      it('exports error shorthand', async () => {
        // GIVEN: Exported error function
        const { error } = await import('./notifications')

        // WHEN: Calling shorthand
        await error('Quick Error')

        // THEN: Works same as notifications.error
        expect(toast.error).toHaveBeenCalled()
      })
    })
  })
})
