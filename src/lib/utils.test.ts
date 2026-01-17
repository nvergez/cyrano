import { describe, it, expect } from 'vitest'
import { cn } from './utils'

describe('cn (className utility)', () => {
  describe('[P1] basic functionality', () => {
    it('merges multiple class names', () => {
      // GIVEN: Multiple class name strings
      // WHEN: Merging them with cn
      const result = cn('foo', 'bar')

      // THEN: Returns merged classes
      expect(result).toBe('foo bar')
    })

    it('handles single class name', () => {
      // GIVEN: Single class name
      // WHEN: Passing to cn
      const result = cn('foo')

      // THEN: Returns the class unchanged
      expect(result).toBe('foo')
    })

    it('handles empty input', () => {
      // GIVEN: No arguments
      // WHEN: Calling cn
      const result = cn()

      // THEN: Returns empty string
      expect(result).toBe('')
    })
  })

  describe('[P1] conditional classes', () => {
    it('filters out falsy values', () => {
      // GIVEN: Mix of truthy and falsy values
      const showBar = false
      // WHEN: Passing to cn
      const result = cn('foo', showBar && 'bar', 'baz', undefined, null)

      // THEN: Only includes truthy values
      expect(result).toBe('foo baz')
    })

    it('handles conditional expressions', () => {
      // GIVEN: Conditional expression
      const isActive = true
      const isDisabled = false

      // WHEN: Using conditional classes
      const result = cn('base', isActive && 'active', isDisabled && 'disabled')

      // THEN: Only includes true conditions
      expect(result).toBe('base active')
    })
  })

  describe('[P1] tailwind merge functionality', () => {
    it('merges conflicting tailwind classes correctly', () => {
      // GIVEN: Conflicting padding classes
      // WHEN: Merging with cn
      const result = cn('px-2 py-1', 'px-4')

      // THEN: Later class wins for conflicts
      expect(result).toBe('py-1 px-4')
    })

    it('merges conflicting color classes', () => {
      // GIVEN: Conflicting text color classes
      // WHEN: Merging with cn
      const result = cn('text-red-500', 'text-blue-500')

      // THEN: Later class wins
      expect(result).toBe('text-blue-500')
    })

    it('preserves non-conflicting classes', () => {
      // GIVEN: Non-conflicting classes
      // WHEN: Merging with cn
      const result = cn('text-red-500', 'bg-blue-500', 'p-4')

      // THEN: All classes are preserved
      expect(result).toBe('text-red-500 bg-blue-500 p-4')
    })

    it('handles complex tailwind patterns', () => {
      // GIVEN: Complex tailwind class combinations
      // WHEN: Merging with cn
      const result = cn(
        'flex items-center justify-center',
        'hover:bg-gray-100',
        'flex-col'
      )

      // THEN: flex-col overrides flex direction but preserves others
      expect(result).toContain('items-center')
      expect(result).toContain('justify-center')
      expect(result).toContain('hover:bg-gray-100')
      expect(result).toContain('flex-col')
    })
  })

  describe('[P2] object syntax', () => {
    it('handles clsx object syntax', () => {
      // GIVEN: Object with conditional classes
      // WHEN: Passing to cn
      const result = cn({ foo: true, bar: false, baz: true })

      // THEN: Only includes truthy keys
      expect(result).toBe('foo baz')
    })

    it('combines strings and objects', () => {
      // GIVEN: Mix of strings and objects
      // WHEN: Passing to cn
      const result = cn('base', { active: true, disabled: false })

      // THEN: Combines all truthy values
      expect(result).toBe('base active')
    })
  })

  describe('[P2] array syntax', () => {
    it('handles array of class names', () => {
      // GIVEN: Array of classes
      // WHEN: Passing to cn
      const result = cn(['foo', 'bar'])

      // THEN: Flattens and joins
      expect(result).toBe('foo bar')
    })

    it('handles nested arrays', () => {
      // GIVEN: Nested array of classes
      // WHEN: Passing to cn
      const result = cn(['foo', ['bar', 'baz']])

      // THEN: Flattens all levels
      expect(result).toBe('foo bar baz')
    })
  })
})
