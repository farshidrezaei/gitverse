import { describe, expect, it } from 'vitest'
import type { FileHistoryTimeline } from '~/types/repository'

describe('file history timeline and revision scrubber', () => {
  const sampleTimeline: FileHistoryTimeline = {
    file_path: 'src-tauri/src/domain/mod.rs',
    total_revisions: 3,
    revisions: [
      {
        commit_hash: 'a2d87c0edeb9f3d034e25a6f01fb98f28673ad99',
        author_name: 'Farshid Rezaei',
        author_email: 'farshid.net1@gmail.com',
        author_timestamp: 1700002000,
        commit_summary: 'feat(insights): add repository metrics',
        change_status: 'M',
      },
      {
        commit_hash: '5526a066db46bb9f0426c03b09ed33011c9acdef',
        author_name: 'Farshid Rezaei',
        author_email: 'farshid.net1@gmail.com',
        author_timestamp: 1700001000,
        commit_summary: 'feat(submodule): add submodule tracking',
        change_status: 'M',
      },
      {
        commit_hash: '09b90587d106cbcf9015886f02d05c4058babe6d',
        author_name: 'Farshid Rezaei',
        author_email: 'farshid.net1@gmail.com',
        author_timestamp: 1700000000,
        commit_summary: 'chore: initial commit',
        change_status: 'A',
      },
    ],
  }

  it('verifies timeline order is chronological newest to oldest', () => {
    expect(sampleTimeline.revisions[0].author_timestamp).toBeGreaterThan(
      sampleTimeline.revisions[1].author_timestamp
    )
    expect(sampleTimeline.revisions[1].author_timestamp).toBeGreaterThan(
      sampleTimeline.revisions[2].author_timestamp
    )
  })

  it('determines badge color based on change status', () => {
    function getStatusColor(status: string): string {
      if (status.startsWith('A')) return 'success'
      if (status.startsWith('M')) return 'primary'
      if (status.startsWith('R')) return 'warning'
      if (status.startsWith('D')) return 'error'
      return 'neutral'
    }

    expect(getStatusColor('A')).toBe('success')
    expect(getStatusColor('M')).toBe('primary')
    expect(getStatusColor('R100')).toBe('warning')
    expect(getStatusColor('D')).toBe('error')
  })

  it('clamps scrubber revision index within bounds', () => {
    function clampRevisionIndex(index: number, total: number): number {
      if (total <= 0) return 0
      return Math.max(0, Math.min(index, total - 1))
    }

    expect(clampRevisionIndex(-1, 3)).toBe(0)
    expect(clampRevisionIndex(1, 3)).toBe(1)
    expect(clampRevisionIndex(5, 3)).toBe(2)
  })
})
