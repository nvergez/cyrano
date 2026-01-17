import { describe, it, expect } from 'vitest'
import { getPlatformStrings, formatShortcut } from './platform-strings'

describe('getPlatformStrings', () => {
  describe('[P1] macOS strings', () => {
    it('returns correct macOS strings', () => {
      // GIVEN: macOS platform
      const platform = 'macos' as const

      // WHEN: Getting platform strings
      const strings = getPlatformStrings(platform)

      // THEN: Returns macOS-specific values
      expect(strings.revealInFileManager).toBe('Reveal in Finder')
      expect(strings.fileManagerName).toBe('Finder')
      expect(strings.modifierKey).toBe('Cmd')
      expect(strings.modifierKeySymbol).toBe('⌘')
      expect(strings.optionKey).toBe('Option')
      expect(strings.optionKeySymbol).toBe('⌥')
      expect(strings.preferencesLabel).toBe('Preferences')
      expect(strings.quitLabel).toBe('Quit')
      expect(strings.trashName).toBe('Trash')
    })
  })

  describe('[P1] Windows strings', () => {
    it('returns correct Windows strings', () => {
      // GIVEN: Windows platform
      const platform = 'windows' as const

      // WHEN: Getting platform strings
      const strings = getPlatformStrings(platform)

      // THEN: Returns Windows-specific values
      expect(strings.revealInFileManager).toBe('Show in Explorer')
      expect(strings.fileManagerName).toBe('Explorer')
      expect(strings.modifierKey).toBe('Ctrl')
      expect(strings.modifierKeySymbol).toBe('Ctrl')
      expect(strings.optionKey).toBe('Alt')
      expect(strings.optionKeySymbol).toBe('Alt')
      expect(strings.preferencesLabel).toBe('Settings')
      expect(strings.quitLabel).toBe('Exit')
      expect(strings.trashName).toBe('Recycle Bin')
    })
  })

  describe('[P1] Linux strings', () => {
    it('returns correct Linux strings', () => {
      // GIVEN: Linux platform
      const platform = 'linux' as const

      // WHEN: Getting platform strings
      const strings = getPlatformStrings(platform)

      // THEN: Returns Linux-specific values
      expect(strings.revealInFileManager).toBe('Show in Files')
      expect(strings.fileManagerName).toBe('Files')
      expect(strings.modifierKey).toBe('Ctrl')
      expect(strings.modifierKeySymbol).toBe('Ctrl')
      expect(strings.optionKey).toBe('Alt')
      expect(strings.optionKeySymbol).toBe('Alt')
      expect(strings.preferencesLabel).toBe('Preferences')
      expect(strings.quitLabel).toBe('Quit')
      expect(strings.trashName).toBe('Trash')
    })
  })

  describe('[P1] undefined platform handling', () => {
    it('defaults to macOS strings when platform is undefined', () => {
      // GIVEN: Undefined platform
      const platform = undefined

      // WHEN: Getting platform strings
      const strings = getPlatformStrings(platform)

      // THEN: Defaults to macOS values
      expect(strings.modifierKeySymbol).toBe('⌘')
      expect(strings.fileManagerName).toBe('Finder')
    })
  })
})

describe('formatShortcut', () => {
  describe('[P1] macOS shortcuts', () => {
    it('formats simple modifier+key shortcut on macOS', () => {
      // GIVEN: macOS platform and key K with mod modifier
      const platform = 'macos' as const

      // WHEN: Formatting shortcut
      const result = formatShortcut(platform, 'K')

      // THEN: Returns ⌘K format
      expect(result).toBe('⌘K')
    })

    it('formats shortcut with shift modifier on macOS', () => {
      // GIVEN: macOS platform with shift+mod modifiers
      const platform = 'macos' as const

      // WHEN: Formatting shortcut with shift
      const result = formatShortcut(platform, 'K', ['shift', 'mod'])

      // THEN: Returns ⇧⌘K format
      expect(result).toBe('⇧⌘K')
    })

    it('formats shortcut with alt modifier on macOS', () => {
      // GIVEN: macOS platform with alt+mod modifiers
      const platform = 'macos' as const

      // WHEN: Formatting shortcut with alt
      const result = formatShortcut(platform, 'K', ['alt', 'mod'])

      // THEN: Returns ⌥⌘K format
      expect(result).toBe('⌥⌘K')
    })

    it('formats shortcut with all modifiers on macOS', () => {
      // GIVEN: macOS platform with all modifiers
      const platform = 'macos' as const

      // WHEN: Formatting shortcut with all modifiers
      const result = formatShortcut(platform, 'K', ['shift', 'alt', 'mod'])

      // THEN: Returns ⇧⌥⌘K format
      expect(result).toBe('⇧⌥⌘K')
    })

    it('formats shortcut without modifiers on macOS', () => {
      // GIVEN: macOS platform with no modifiers
      const platform = 'macos' as const

      // WHEN: Formatting shortcut with empty modifiers array
      const result = formatShortcut(platform, 'F1', [])

      // THEN: Returns just the key
      expect(result).toBe('F1')
    })
  })

  describe('[P1] Windows shortcuts', () => {
    it('formats simple modifier+key shortcut on Windows', () => {
      // GIVEN: Windows platform and key K with mod modifier
      const platform = 'windows' as const

      // WHEN: Formatting shortcut
      const result = formatShortcut(platform, 'K')

      // THEN: Returns Ctrl+K format
      expect(result).toBe('Ctrl+K')
    })

    it('formats shortcut with shift modifier on Windows', () => {
      // GIVEN: Windows platform with shift+mod modifiers
      const platform = 'windows' as const

      // WHEN: Formatting shortcut with shift
      const result = formatShortcut(platform, 'K', ['shift', 'mod'])

      // THEN: Returns Shift+Ctrl+K format
      expect(result).toBe('Shift+Ctrl+K')
    })

    it('formats shortcut with alt modifier on Windows', () => {
      // GIVEN: Windows platform with alt+mod modifiers
      const platform = 'windows' as const

      // WHEN: Formatting shortcut with alt
      const result = formatShortcut(platform, 'K', ['alt', 'mod'])

      // THEN: Returns Alt+Ctrl+K format
      expect(result).toBe('Alt+Ctrl+K')
    })
  })

  describe('[P1] Linux shortcuts', () => {
    it('formats shortcut same as Windows on Linux', () => {
      // GIVEN: Linux platform
      const platform = 'linux' as const

      // WHEN: Formatting shortcut
      const result = formatShortcut(platform, 'S', ['shift', 'mod'])

      // THEN: Returns same format as Windows
      expect(result).toBe('Shift+Ctrl+S')
    })
  })

  describe('[P1] undefined platform handling', () => {
    it('defaults to macOS format when platform is undefined', () => {
      // GIVEN: Undefined platform
      const platform = undefined

      // WHEN: Formatting shortcut
      const result = formatShortcut(platform, 'K')

      // THEN: Defaults to macOS format
      expect(result).toBe('⌘K')
    })
  })
})
