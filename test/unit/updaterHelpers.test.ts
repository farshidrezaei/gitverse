import { describe, expect, it } from 'vitest'
import {
  buildAvailableUpdateSummary,
  calculateDownloadProgressPercentage,
  normalizeUpdateReleaseNotes,
} from '~/utils/updaterHelpers'

describe('Updater Helpers', () => {
  it('calculates download progress percentage accurately', () => {
    expect(calculateDownloadProgressPercentage(0, 1000)).toBe(0)
    expect(calculateDownloadProgressPercentage(500, 1000)).toBe(50)
    expect(calculateDownloadProgressPercentage(1000, 1000)).toBe(100)
    expect(calculateDownloadProgressPercentage(1200, 1000)).toBe(100) // bounded to 100
  })

  it('handles zero or missing totalContentLength gracefully', () => {
    expect(calculateDownloadProgressPercentage(100, 0)).toBeNull()
    expect(calculateDownloadProgressPercentage(100, null)).toBeNull()
    expect(calculateDownloadProgressPercentage(100, undefined)).toBeNull()
    expect(calculateDownloadProgressPercentage(-50, 1000)).toBe(0)
  })

  it('normalizes release notes string', () => {
    expect(normalizeUpdateReleaseNotes('  Features added  ')).toBe('Features added')
    expect(normalizeUpdateReleaseNotes(undefined)).toBe('')
  })

  it('builds available update summary properly', () => {
    const summary = buildAvailableUpdateSummary({
      currentVersion: '1.0.0',
      version: '1.1.0',
      date: '2026-11-23',
      body: '### Added\n- New commit graph performance optimization',
    })

    expect(summary.currentVersion).toBe('1.0.0')
    expect(summary.version).toBe('1.1.0')
    expect(summary.publishedAt).toBe('2026-11-23')
    expect(summary.releaseNotes).toContain('New commit graph performance optimization')
  })
})
