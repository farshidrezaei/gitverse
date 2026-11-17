import { describe, expect, it } from 'vitest'
import type { MergeBranchOutcome, RebaseBranchOutcome } from '~/types/repository'
import { validateGitBranchName } from '~/utils/branchValidation'

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

  it('correctly validates branch name format and uniqueness', () => {
    const existing = ['main', 'develop', 'feature/login']

    // Valid branch names
    expect(validateGitBranchName('feature/login-v2', { currentBranchName: 'feature/login', existingBranchNames: existing }).isValid).toBe(true)
    expect(validateGitBranchName('bugfix_123', { existingBranchNames: existing }).isValid).toBe(true)
    expect(validateGitBranchName('v1.0.0-release', { existingBranchNames: existing }).isValid).toBe(true)

    // Uniqueness: already exists
    const duplicate = validateGitBranchName('develop', { existingBranchNames: existing })
    expect(duplicate.isValid).toBe(false)
    expect(duplicate.errorKey).toBe('branch.validationAlreadyExists')

    // Same as current
    const same = validateGitBranchName('feature/login', { currentBranchName: 'feature/login', existingBranchNames: existing })
    expect(same.isValid).toBe(false)
    expect(same.errorKey).toBe('branch.validationSameAsCurrent')

    // Empty
    expect(validateGitBranchName('   ').isValid).toBe(false)

    // Spaces
    const spaces = validateGitBranchName('feature new')
    expect(spaces.isValid).toBe(false)
    expect(spaces.errorKey).toBe('branch.validationNoSpaces')

    // Leading / trailing slashes or dashes
    expect(validateGitBranchName('/feature').isValid).toBe(false)
    expect(validateGitBranchName('feature/').isValid).toBe(false)
    expect(validateGitBranchName('-feature').isValid).toBe(false)
    expect(validateGitBranchName('feature-').isValid).toBe(false)
    expect(validateGitBranchName('.feature').isValid).toBe(false)

    // Consecutive separators
    expect(validateGitBranchName('feature//test').isValid).toBe(false)
    expect(validateGitBranchName('feature..test').isValid).toBe(false)

    // Lock file
    expect(validateGitBranchName('feature.lock').isValid).toBe(false)

    // Invalid chars
    expect(validateGitBranchName('feature~1').isValid).toBe(false)
    expect(validateGitBranchName('feature^2').isValid).toBe(false)
    expect(validateGitBranchName('feature:test').isValid).toBe(false)
  })
})
