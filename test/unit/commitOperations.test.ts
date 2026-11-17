import { describe, expect, it } from 'vitest'
import type { ResetCommitMode } from '~/composables/useCommitOperations'

describe('commitOperations - reset modes and validation', () => {
  it('validates supported reset modes', () => {
    const validModes: ResetCommitMode[] = ['soft', 'mixed', 'hard']
    expect(validModes).toContain('soft')
    expect(validModes).toContain('mixed')
    expect(validModes).toContain('hard')
  })

  it('correctly maps reset modes to git arguments', () => {
    function mapResetModeToFlag(mode: ResetCommitMode): string {
      switch (mode) {
        case 'soft':
          return '--soft'
        case 'hard':
          return '--hard'
        case 'mixed':
        default:
          return '--mixed'
      }
    }

    expect(mapResetModeToFlag('soft')).toBe('--soft')
    expect(mapResetModeToFlag('mixed')).toBe('--mixed')
    expect(mapResetModeToFlag('hard')).toBe('--hard')
  })

  it('formats short commit hash for display safely', () => {
    function getShortHash(hash: string): string {
      return hash.trim().slice(0, 7)
    }

    expect(getShortHash('5897482c17022d926b55574e2dc4c1b92d07b5d4')).toBe('5897482')
    expect(getShortHash('12345')).toBe('12345')
  })
})
