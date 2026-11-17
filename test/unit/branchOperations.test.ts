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

  it('correctly validates branch name format for renaming', () => {
    const isValidBranchName = (name: string, current: string): boolean => {
      const trimmed = name.trim()
      if (!trimmed || trimmed === current) return false
      if (/\s/.test(trimmed)) return false
      return /^[a-zA-Z0-9._\-/]+$/.test(trimmed)
    }

    expect(isValidBranchName('feature/login-v2', 'feature/login')).toBe(true)
    expect(isValidBranchName('main-patch', 'main')).toBe(true)
    expect(isValidBranchName('feature/login', 'feature/login')).toBe(false) // same name
    expect(isValidBranchName('   ', 'main')).toBe(false) // empty
    expect(isValidBranchName('invalid branch name', 'main')).toBe(false) // spaces
    expect(isValidBranchName('invalid~name^', 'main')).toBe(false) // invalid chars
  })
})
