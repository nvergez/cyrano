import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { SuccessIndicator } from './SuccessIndicator'

describe('SuccessIndicator', () => {
  it('renders the success indicator with green checkmark', () => {
    render(<SuccessIndicator />)

    // Check for the SVG checkmark icon
    const svg = document.querySelector('svg')
    expect(svg).toBeInTheDocument()
    expect(svg).toHaveClass('text-green-500')
  })

  it('displays the success message "Done"', () => {
    render(<SuccessIndicator />)

    expect(screen.getByText('Done')).toBeInTheDocument()
    expect(screen.getByText('Done')).toHaveClass('text-green-500')
  })

  it('renders with correct layout structure', () => {
    const { container } = render(<SuccessIndicator />)

    // Check for flex container with gap
    const flexContainer = container.querySelector('.flex.items-center.gap-3')
    expect(flexContainer).toBeInTheDocument()
  })

  it('renders checkmark icon in a circular background', () => {
    const { container } = render(<SuccessIndicator />)

    // Check for the circular icon container
    const iconContainer = container.querySelector(
      '.rounded-full.bg-green-500\\/20'
    )
    expect(iconContainer).toBeInTheDocument()
  })
})
