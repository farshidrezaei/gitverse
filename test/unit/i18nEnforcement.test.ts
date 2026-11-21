import { describe, expect, it } from 'vitest'
import { validateLocaleDirectory } from '../../scripts/validate-i18n.mjs'

describe('Zero-Missing-Translations & i18n Strict Contract', () => {
  it('guarantees 100% key parity between en and fa, non-empty strings, and placeholder matching', () => {
    const errors = validateLocaleDirectory()
    expect(errors).toEqual([])
  })
})
