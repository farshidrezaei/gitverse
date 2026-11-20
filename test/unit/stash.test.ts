import { describe, expect, it } from 'vitest'
import {
  formatStashDisplayMessage,
  formatStashRelativeTime,
  parseStashIndex,
} from '~/utils/stash'

describe('stash utilities', () => {
  describe('parseStashIndex', () => {
    it('correctly extracts numeric indices from stash identifiers', () => {
      expect(parseStashIndex('stash@{0}')).toBe(0)
      expect(parseStashIndex('stash@{3}')).toBe(3)
      expect(parseStashIndex('stash@{15}')).toBe(15)
    })

    it('falls back to 0 for malformed stash identifiers', () => {
      expect(parseStashIndex('invalid-stash')).toBe(0)
      expect(parseStashIndex('')).toBe(0)
    })
  })

  describe('formatStashDisplayMessage', () => {
    it('cleans up "WIP on branch: hash message" format', () => {
      expect(formatStashDisplayMessage('WIP on master: 1234567 initial commit')).toBe(
        'initial commit'
      )
      expect(formatStashDisplayMessage('WIP on feature/auth: fix login bug')).toBe(
        'fix login bug'
      )
    })

    it('cleans up "On branch: message" format', () => {
      expect(formatStashDisplayMessage('On master: my custom stash message')).toBe(
        'my custom stash message'
      )
      expect(formatStashDisplayMessage('On feature: work in progress')).toBe(
        'work in progress'
      )
    })

    it('returns raw message when no prefix is matched', () => {
      expect(formatStashDisplayMessage('Custom stash text without branch')).toBe(
        'Custom stash text without branch'
      )
    })
  })

  describe('formatStashRelativeTime', () => {
    it('formats seconds, minutes, hours, and days correctly', () => {
      const now = 1000000
      expect(formatStashRelativeTime(now - 10, now)).toBe('10s ago')
      expect(formatStashRelativeTime(now - 120, now)).toBe('2m ago')
      expect(formatStashRelativeTime(now - 7200, now)).toBe('2h ago')
      expect(formatStashRelativeTime(now - 172800, now)).toBe('2d ago')
    })
  })

  describe('stash details filtering', () => {
    it('filters changed files by file path search query', () => {
      const changedFiles = [
        { path: 'src/main.rs', original_path: null, status: 'modified' as const, additions: 5, deletions: 2 },
        { path: 'src/lib.rs', original_path: null, status: 'modified' as const, additions: 10, deletions: 0 },
        { path: 'package.json', original_path: null, status: 'modified' as const, additions: 1, deletions: 1 },
      ]

      const filterFiles = (query: string) => {
        const normalized = query.trim().toLowerCase()
        if (!normalized) return changedFiles
        return changedFiles.filter((file) => file.path.toLowerCase().includes(normalized))
      }

      expect(filterFiles('')).toHaveLength(3)
      expect(filterFiles('main')).toHaveLength(1)
      expect(filterFiles('main')[0]?.path).toBe('src/main.rs')
      expect(filterFiles('src')).toHaveLength(2)
      expect(filterFiles('nonexistent')).toHaveLength(0)
    })
  })
})
