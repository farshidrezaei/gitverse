import { describe, expect, it } from 'vitest'
import type { MergeBranchOutcome, RebaseBranchOutcome } from '~/types/repository'

describe('branch operations models and contracts', () => {
  it('correctly validates clean merge branch outcome', () => {
    const cleanMerge: MergeBranchOutcome = {
      is_successful: true,
      has_conflicts: false,
      message: 'Fast-forward',
    }

    expect(cleanMerge.is_successful).toBe(true)
    expect(cleanMerge.has_conflicts).toBe(false)
  })

  it('correctly validates conflicted merge branch outcome', () => {
    const conflictedMerge: MergeBranchOutcome = {
      is_successful: false,
      has_conflicts: true,
      message: 'CONFLICT (content): Merge conflict in app.vue',
    }

    expect(conflictedMerge.is_successful).toBe(false)
    expect(conflictedMerge.has_conflicts).toBe(true)
    expect(conflictedMerge.message).toContain('CONFLICT')
  })

  it('correctly validates rebase branch outcomes', () => {
    const cleanRebase: RebaseBranchOutcome = {
      is_successful: true,
      has_conflicts: false,
      message: 'Successfully rebased and updated refs/heads/feature.',
    }

    const conflictedRebase: RebaseBranchOutcome = {
      is_successful: false,
      has_conflicts: true,
      message: 'Failed to merge in the changes.\nPatch failed at 0001 add button',
    }

    expect(cleanRebase.is_successful).toBe(true)
    expect(conflictedRebase.is_successful).toBe(false)
    expect(conflictedRebase.has_conflicts).toBe(true)
  })
})
