import { describe, expect, it } from 'vitest'
import { useCommandPalette } from '~/composables/useCommandPalette'

describe('useCommandPalette', () => {
  it('initializes with closed state', () => {
    const { isCommandPaletteOpen } = useCommandPalette()
    expect(isCommandPaletteOpen.value).toBe(false)
  })

  it('opens and closes command palette', () => {
    const { isCommandPaletteOpen, openCommandPalette, closeCommandPalette } = useCommandPalette()
    openCommandPalette()
    expect(isCommandPaletteOpen.value).toBe(true)

    closeCommandPalette()
    expect(isCommandPaletteOpen.value).toBe(false)
  })

  it('toggles command palette state', () => {
    const { isCommandPaletteOpen, toggleCommandPalette } = useCommandPalette()
    isCommandPaletteOpen.value = false

    toggleCommandPalette()
    expect(isCommandPaletteOpen.value).toBe(true)

    toggleCommandPalette()
    expect(isCommandPaletteOpen.value).toBe(false)
  })
})
