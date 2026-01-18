import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { TranscribingIndicator } from './TranscribingIndicator'

describe('TranscribingIndicator', () => {
  it('renders transcribing text', () => {
    render(<TranscribingIndicator />)

    expect(screen.getByText('Transcribing...')).toBeInTheDocument()
  })

  it('renders a spinner icon', () => {
    render(<TranscribingIndicator />)

    // Check for SVG element with spinner styling
    const svg = document.querySelector('svg')
    expect(svg).toBeInTheDocument()
    expect(svg).toHaveClass('animate-spin')
  })

  it('applies blue color to spinner', () => {
    render(<TranscribingIndicator />)

    const svg = document.querySelector('svg')
    expect(svg).toHaveClass('text-blue-500')
  })
})
