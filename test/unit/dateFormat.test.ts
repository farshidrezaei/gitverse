import { describe, expect, it } from 'vitest'
import { formatRelativeTime, formatTime } from '~/utils/dateFormat'

describe('dateFormat Utils', () => {
  it('formats relative time accurately', () => {
    const nowSeconds = Math.floor(Date.now() / 1000)
    expect(formatRelativeTime(nowSeconds)).toBe('just now')
    expect(formatRelativeTime(nowSeconds - 120)).toBe('2m ago')
    expect(formatRelativeTime(nowSeconds - 7200)).toBe('2h ago')
    expect(formatRelativeTime(nowSeconds - 172800)).toBe('2d ago')
  })

  it('formats timestamp to time string (HH:MM:SS)', () => {
    const timestamp = new Date('2026-11-21T14:30:15').getTime()
    const formatted = formatTime(timestamp)
    expect(formatted).toMatch(/\d{2}:\d{2}:\d{2}/)
  })
})
