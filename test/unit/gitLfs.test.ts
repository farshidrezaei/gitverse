import { describe, expect, it } from 'vitest'
import type { GitLfsOverview } from '~/types/repository'

describe('git lfs data model', () => {
  const sampleOverview: GitLfsOverview = {
    is_lfs_installed: true,
    tracked_patterns: ['*.psd', 'assets/*.bin'],
    tracked_files: [
      {
        file_path: 'mockup.psd',
        object_identifier: 'oid123',
        size_in_bytes: 1048576,
      },
    ],
    locks: [
      {
        lock_identifier: '1',
        file_path: 'mockup.psd',
        owner_name: 'Farshid Rezaei',
      },
    ],
    total_lfs_size_in_bytes: 1048576,
  }

  it('correctly models active lfs environment and locks', () => {
    expect(sampleOverview.is_lfs_installed).toBe(true)
    expect(sampleOverview.tracked_patterns.length).toBe(2)
    expect(sampleOverview.tracked_files.length).toBe(1)
    expect(sampleOverview.locks.length).toBe(1)
    expect(sampleOverview.locks[0].owner_name).toBe('Farshid Rezaei')
  })
})
