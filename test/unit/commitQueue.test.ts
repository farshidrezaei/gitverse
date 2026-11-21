import { describe, expect, it } from 'vitest'
import type { CommitQueueOperationResult } from '~/types/repository'

describe('commit queue operations data model', () => {
  const successOutcome: CommitQueueOperationResult = {
    is_successful: true,
    completed_commits: ['a1b2c3d', 'e4f5g6h'],
    failed_commit_hash: null,
    conflict_files: [],
    message: 'All cherry-picked commits applied successfully',
  }

  const conflictOutcome: CommitQueueOperationResult = {
    is_successful: false,
    completed_commits: ['a1b2c3d'],
    failed_commit_hash: 'e4f5g6h',
    conflict_files: ['src/main.rs', 'package.json'],
    message: 'Merge conflict detected during cherry-pick',
  }

  it('correctly reflects successful batch cherry-pick completion', () => {
    expect(successOutcome.is_successful).toBe(true)
    expect(successOutcome.completed_commits.length).toBe(2)
    expect(successOutcome.failed_commit_hash).toBeNull()
    expect(successOutcome.conflict_files).toEqual([])
  })

  it('correctly captures interrupted state and conflict files upon merge conflict', () => {
    expect(conflictOutcome.is_successful).toBe(false)
    expect(conflictOutcome.completed_commits.length).toBe(1)
    expect(conflictOutcome.failed_commit_hash).toBe('e4f5g6h')
    expect(conflictOutcome.conflict_files.length).toBe(2)
    expect(conflictOutcome.conflict_files).toContain('src/main.rs')
  })
})
