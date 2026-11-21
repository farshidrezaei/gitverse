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
import {
  connectGitHubAccountSchema,
  connectGitLabAccountSchema,
  createPullRequestInputSchema,
} from '../../app/schemas/providerSchema'
import {
  createBundleInputSchema,
  verifyBundleInputSchema,
  exportArchiveInputSchema,
} from '../../app/schemas/archiveSchema'
import { trackLfsPatternInputSchema } from '../../app/schemas/lfsSchema'
import { customActionSchema } from '../../app/schemas/customActionSchema'
import { searchRepositoryInputSchema } from '../../app/schemas/searchSchema'
import { startBisectInputSchema, bisectDecisionSchema } from '../../app/schemas/bisectSchema'
import { applyPatchInputSchema, patchContentSchema } from '../../app/schemas/patchSchema'

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

  describe('Provider Account & Pull Request Validation', () => {
    it('validates GitHub account connection inputs', () => {
      expect(
        connectGitHubAccountSchema.safeParse({
          personalAccessToken: 'ghp_123456789012345678901234567890',
          instanceUrl: 'https://github.com',
        }).success,
      ).toBe(true)

      // Token too short
      expect(
        connectGitHubAccountSchema.safeParse({
          personalAccessToken: 'short',
        }).success,
      ).toBe(false)

      // Invalid instance url
      expect(
        connectGitHubAccountSchema.safeParse({
          personalAccessToken: 'ghp_123456789012345678901234567890',
          instanceUrl: 'not-a-url',
        }).success,
      ).toBe(false)
    })

    it('validates GitLab account connection inputs', () => {
      expect(
        connectGitLabAccountSchema.safeParse({
          personalAccessToken: 'glpat-12345678901234567890',
          instanceUrl: 'https://gitlab.com',
        }).success,
      ).toBe(true)

      // Token with spaces
      expect(
        connectGitLabAccountSchema.safeParse({
          personalAccessToken: 'glpat-12345 678901234567890',
        }).success,
      ).toBe(false)
    })

    it('validates create pull request inputs', () => {
      expect(
        createPullRequestInputSchema.safeParse({
          title: 'feat: add awesome feature',
          sourceBranch: 'feature/login',
          targetBranch: 'main',
          isDraft: false,
        }).success,
      ).toBe(true)

      expect(
        createPullRequestInputSchema.safeParse({
          title: '',
          sourceBranch: 'feature/login',
          targetBranch: 'main',
        }).success,
      ).toBe(false)
    })
  })

  describe('Archive & Bundle Input Validation', () => {
    it('validates bundle create and verify schemas', () => {
      expect(createBundleInputSchema.safeParse({ outputBundlePath: '/tmp/repo.bundle', revisionRange: '--all' }).success).toBe(true)
      expect(createBundleInputSchema.safeParse({ outputBundlePath: '' }).success).toBe(false)
      expect(verifyBundleInputSchema.safeParse({ bundlePath: '/tmp/repo.bundle' }).success).toBe(true)
      expect(verifyBundleInputSchema.safeParse({ bundlePath: '' }).success).toBe(false)
    })

    it('validates export archive schema', () => {
      expect(exportArchiveInputSchema.safeParse({ outputArchivePath: '/tmp/archive.zip', reference: 'HEAD' }).success).toBe(true)
      expect(exportArchiveInputSchema.safeParse({ outputArchivePath: '', reference: 'HEAD' }).success).toBe(false)
      expect(exportArchiveInputSchema.safeParse({ outputArchivePath: '/tmp/archive.zip', reference: '' }).success).toBe(false)
    })
  })

  describe('Git LFS & Custom Action Validation', () => {
    it('validates Git LFS pattern schema', () => {
      expect(trackLfsPatternInputSchema.safeParse({ pattern: '*.psd' }).success).toBe(true)
      expect(trackLfsPatternInputSchema.safeParse({ pattern: 'assets/*.bin' }).success).toBe(true)
      expect(trackLfsPatternInputSchema.safeParse({ pattern: '' }).success).toBe(false)
      expect(trackLfsPatternInputSchema.safeParse({ pattern: 'bad\npattern' }).success).toBe(false)
    })

    it('validates Custom Action schema', () => {
      expect(
        customActionSchema.safeParse({
          action_identifier: 'run_tests',
          display_name: 'Run Unit Tests',
          command_string: 'npm test',
          icon_name: 'i-lucide-play',
          description_text: 'Executes vitest suite',
        }).success,
      ).toBe(true)

      expect(
        customActionSchema.safeParse({
          action_identifier: 'invalid identifier with spaces',
          display_name: 'Action',
          command_string: 'echo hi',
        }).success,
      ).toBe(false)

      expect(
        customActionSchema.safeParse({
          action_identifier: 'valid_id',
          display_name: '',
          command_string: 'echo hi',
        }).success,
      ).toBe(false)
    })
  })

  describe('Search, Bisect & Patch Validation', () => {
    it('validates repository search schema', () => {
      expect(searchRepositoryInputSchema.safeParse({ query: 'myFunction', isCaseSensitive: true }).success).toBe(true)
      expect(searchRepositoryInputSchema.safeParse({ query: '' }).success).toBe(false)
      expect(searchRepositoryInputSchema.safeParse({ query: 'bad\0null' }).success).toBe(false)
    })

    it('validates bisect start and decision schemas', () => {
      expect(startBisectInputSchema.safeParse({ badCommitHash: 'HEAD', goodCommitHash: 'v1.0.0' }).success).toBe(true)
      expect(startBisectInputSchema.safeParse({ badCommitHash: '', goodCommitHash: 'v1.0.0' }).success).toBe(false)
      expect(startBisectInputSchema.safeParse({ badCommitHash: 'bad hash with spaces', goodCommitHash: 'v1.0.0' }).success).toBe(false)

      expect(bisectDecisionSchema.safeParse('good').success).toBe(true)
      expect(bisectDecisionSchema.safeParse('bad').success).toBe(true)
      expect(bisectDecisionSchema.safeParse('skip').success).toBe(true)
      expect(bisectDecisionSchema.safeParse('invalid_decision').success).toBe(false)
    })

    it('validates patch content and application schemas', () => {
      expect(patchContentSchema.safeParse('diff --git a/f b/f\n--- a/f\n+++ b/f').success).toBe(true)
      expect(patchContentSchema.safeParse('').success).toBe(false)
      expect(applyPatchInputSchema.safeParse({ patchContent: 'diff content', isCached: true, isReverse: false }).success).toBe(true)
      expect(applyPatchInputSchema.safeParse({ patchContent: '' }).success).toBe(false)
    })
  })
})
