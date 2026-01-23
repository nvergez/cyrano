import { useState, useEffect, useCallback } from 'react'
import { commands, type PermissionStatus } from '@/lib/bindings'
import { logger } from '@/lib/logger'

/**
 * Hook for managing macOS accessibility permission.
 *
 * Accessibility permission is required for cursor insertion functionality.
 * Without this permission, the app falls back to clipboard-only output
 * (graceful degradation).
 *
 * @returns Object containing permission status and control functions
 *
 * @example
 * ```tsx
 * function SettingsPanel() {
 *   const {
 *     status,
 *     isGranted,
 *     isChecking,
 *     checkPermission,
 *     requestPermission,
 *     openSettings
 *   } = useAccessibilityPermission()
 *
 *   if (isChecking) return <Spinner />
 *
 *   return (
 *     <div>
 *       <p>Accessibility: {isGranted ? 'Enabled' : 'Disabled'}</p>
 *       {!isGranted && (
 *         <button onClick={requestPermission}>
 *           Enable Accessibility
 *         </button>
 *       )}
 *     </div>
 *   )
 * }
 * ```
 */
export function useAccessibilityPermission() {
  const [status, setStatus] = useState<PermissionStatus>('NotDetermined')
  const [isChecking, setIsChecking] = useState(false)

  /**
   * Check the current accessibility permission status.
   * Updates the hook's internal state with the result.
   */
  const checkPermission = useCallback(async () => {
    setIsChecking(true)
    try {
      const result = await commands.checkAccessibilityPermission()
      setStatus(result)
      logger.debug('Accessibility permission checked', { status: result })
    } catch (error) {
      logger.error('Failed to check accessibility permission', { error })
      // On error, assume not determined (safe default)
      setStatus('NotDetermined')
    } finally {
      setIsChecking(false)
    }
  }, [])

  /**
   * Request accessibility permission from the user.
   * On macOS, this triggers the system prompt directing to System Preferences.
   *
   * @returns true if permission was granted, false otherwise
   */
  const requestPermission = useCallback(async (): Promise<boolean> => {
    try {
      const result = await commands.requestAccessibilityPermission()
      if (result.status === 'ok') {
        // Recheck status after request
        await checkPermission()
        return result.data
      }
      logger.error('Failed to request accessibility permission', {
        error: result.error,
      })
      return false
    } catch (error) {
      logger.error('Failed to request accessibility permission', { error })
      return false
    }
  }, [checkPermission])

  /**
   * Open the Accessibility preferences pane in System Preferences.
   * Provides a convenient way for users to manually grant permission.
   */
  const openSettings = useCallback(async () => {
    try {
      const result = await commands.openAccessibilitySettings()
      if (result.status !== 'ok') {
        logger.error('Failed to open accessibility settings', {
          error: result.error,
        })
      }
    } catch (error) {
      logger.error('Failed to open accessibility settings', { error })
    }
  }, [])

  // Check permission on mount
  useEffect(() => {
    checkPermission()
  }, [checkPermission])

  return {
    /** Current permission status */
    status,
    /** Whether permission is granted (cursor insertion available) */
    isGranted: status === 'Granted',
    /** Whether a permission check is in progress */
    isChecking,
    /** Function to re-check permission status */
    checkPermission,
    /** Function to request permission (shows system prompt) */
    requestPermission,
    /** Function to open System Preferences > Accessibility */
    openSettings,
  }
}
