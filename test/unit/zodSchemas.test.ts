import { describe, expect, it } from 'vitest'
import {
  validateBranchNameWithSchema,
  gitBranchNameSchema,
} from '../../app/schemas/branchSchema'
import {
  validateRemoteWithSchema,
  gitRemoteNameSchema,
  gitRemoteUrlSchema,
} from '../../app/schemas/remoteSchema'
import { createStashInputSchema } from '../../app/schemas/stashSchema'
import { commitInputSchema } from '../../app/schemas/commitSchema'

describe('Zod Schemas Validation Suite', () => {
  describe('Branch Name Validation', () => {
    it('accepts valid branch names', () => {
      expect(gitBranchNameSchema.safeParse('main').success).toBe(true)
      expect(gitBranchNameSchema.safeParse('feature/login-flow').success).toBe(true)
      expect(gitBranchNameSchema.safeParse('bugfix/issue_123').success).toBe(true)
      expect(gitBranchNameSchema.safeParse('v1.0.0-rc1').success).toBe(true)
    })

    it('rejects invalid branch names with specific messages', () => {
      expect(gitBranchNameSchema.safeParse('').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('   ').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('feature with space').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('/leading-slash').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('trailing-slash/').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('-leading-dash').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('trailing-dash-').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('.leading-dot').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('consecutive//slashes').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('consecutive..dots').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('branch.lock').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('branch@{upstream}').success).toBe(false)
      expect(gitBranchNameSchema.safeParse('branch:colon').success).toBe(false)
    })

    it('validates against existing branch names and current branch', () => {
      const duplicateResult = validateBranchNameWithSchema('main', {
        existingBranchNames: ['main', 'develop'],
      })
      expect(duplicateResult.isValid).toBe(false)
      expect(duplicateResult.errorKey).toBe('branch.validationAlreadyExists')

      const sameAsCurrentResult = validateBranchNameWithSchema('feature', {
        currentBranchName: 'feature',
      })
      expect(sameAsCurrentResult.isValid).toBe(false)
      expect(sameAsCurrentResult.errorKey).toBe('branch.validationSameAsCurrent')
    })
  })

  describe('Remote Name & URL Validation', () => {
    it('accepts valid remote names', () => {
      expect(gitRemoteNameSchema.safeParse('origin').success).toBe(true)
      expect(gitRemoteNameSchema.safeParse('upstream').success).toBe(true)
      expect(gitRemoteNameSchema.safeParse('fork-2026').success).toBe(true)
    })

    it('rejects invalid remote names', () => {
      expect(gitRemoteNameSchema.safeParse('').success).toBe(false)
      expect(gitRemoteNameSchema.safeParse('origin with spaces').success).toBe(false)
      expect(gitRemoteNameSchema.safeParse('/origin').success).toBe(false)
      expect(gitRemoteNameSchema.safeParse('.').success).toBe(false)
      expect(gitRemoteNameSchema.safeParse('..').success).toBe(false)
    })

    it('accepts standard Git remote URLs', () => {
      expect(gitRemoteUrlSchema.safeParse('https://github.com/owner/repo.git').success).toBe(true)
      expect(gitRemoteUrlSchema.safeParse('http://github.com/owner/repo.git').success).toBe(true)
      expect(gitRemoteUrlSchema.safeParse('git@github.com:owner/repo.git').success).toBe(true)
      expect(gitRemoteUrlSchema.safeParse('ssh://git@github.com:22/owner/repo.git').success).toBe(true)
      expect(gitRemoteUrlSchema.safeParse('/home/user/repos/local.git').success).toBe(true)
    })

    it('rejects invalid Git remote URLs', () => {
      expect(gitRemoteUrlSchema.safeParse('').success).toBe(false)
      expect(gitRemoteUrlSchema.safeParse('not a remote url').success).toBe(false)
    })

    it('validates duplicate remote names', () => {
      const result = validateRemoteWithSchema('origin', 'https://github.com/owner/repo.git', {
        existingRemoteNames: ['origin', 'upstream'],
      })
      expect(result.isValid).toBe(false)
      expect(result.errorKey).toBe('remote.validationDuplicateName')
    })
  })

  describe('Stash & Commit Input Validation', () => {
    it('validates stash input schema', () => {
      expect(createStashInputSchema.safeParse({ message: 'Valid stash', shouldIncludeUntracked: true }).success).toBe(true)
      expect(createStashInputSchema.safeParse({ message: 'a'.repeat(600) }).success).toBe(false)
    })

    it('validates commit input schema', () => {
      expect(commitInputSchema.safeParse({ summary: 'feat: add feature' }).success).toBe(true)
      expect(commitInputSchema.safeParse({ summary: '' }).success).toBe(false)
      expect(commitInputSchema.safeParse({ summary: '   ' }).success).toBe(false)
    })
  })
})
