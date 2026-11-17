import { describe, expect, it } from 'vitest'
import {
  calculateRemainingSeconds,
  hasElevatedConfirmationRisk,
  validateConfirmationKeyword,
} from '~/utils/confirmation'

describe('confirmation utilities', () => {
  describe('validateConfirmationKeyword', () => {
    it('returns true when no keyword is required', () => {
      expect(validateConfirmationKeyword('anything', '')).toBe(true)
      expect(validateConfirmationKeyword('', '   ')).toBe(true)
    })

    it('matches exact keyword with trimming', () => {
      expect(validateConfirmationKeyword('  FORCE PUSH  ', 'FORCE PUSH')).toBe(true)
      expect(validateConfirmationKeyword('FORCE PUSH', 'FORCE PUSH')).toBe(true)
      expect(validateConfirmationKeyword('force push', 'FORCE PUSH')).toBe(false)
    })

    it('supports case-insensitive comparison when specified', () => {
      expect(validateConfirmationKeyword('force push', 'FORCE PUSH', false)).toBe(true)
      expect(validateConfirmationKeyword('main', 'MAIN', false)).toBe(true)
      expect(validateConfirmationKeyword('other-branch', 'MAIN', false)).toBe(false)
    })
  })

  describe('calculateRemainingSeconds', () => {
    it('calculates remaining seconds rounded up accurately', () => {
      expect(calculateRemainingSeconds(0, 3000)).toBe(3)
      expect(calculateRemainingSeconds(500, 3000)).toBe(3)
      expect(calculateRemainingSeconds(1001, 3000)).toBe(2)
      expect(calculateRemainingSeconds(2001, 3000)).toBe(1)
      expect(calculateRemainingSeconds(2999, 3000)).toBe(1)
      expect(calculateRemainingSeconds(3000, 3000)).toBe(1)
    })

    it('never returns less than 1 second while active', () => {
      expect(calculateRemainingSeconds(4000, 3000)).toBe(1)
    })
  })

  describe('hasElevatedConfirmationRisk', () => {
    it('flags hold and keyword tiers as elevated risk', () => {
      expect(hasElevatedConfirmationRisk('standard')).toBe(false)
      expect(hasElevatedConfirmationRisk('hold')).toBe(true)
      expect(hasElevatedConfirmationRisk('keyword')).toBe(true)
    })
  })
})
