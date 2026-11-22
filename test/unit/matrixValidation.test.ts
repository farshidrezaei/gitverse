import { describe, expect, it } from 'vitest'
import { gitBranchNameSchema, validateBranchNameWithSchema } from '../../app/schemas/branchSchema'
import { gitRemoteNameSchema, gitRemoteUrlSchema, validateRemoteWithSchema } from '../../app/schemas/remoteSchema'
import { commitInputSchema } from '../../app/schemas/commitSchema'
import { createStashInputSchema } from '../../app/schemas/stashSchema'
import { createPullRequestInputSchema, gitHubTokenSchema, gitLabTokenSchema, instanceUrlSchema } from '../../app/schemas/providerSchema'

describe('Table-Driven Validation Matrix', () => {
  describe('Branch Name Validation Matrix', () => {
    interface BranchTestCase {
      input: string
      isValid: boolean
      expectedErrorKey?: string
      existingBranches?: string[]
      currentBranch?: string
    }

    const testCases: BranchTestCase[] = [
      // Valid cases
      { input: 'main', isValid: true },
      { input: 'master', isValid: true },
      { input: 'develop', isValid: true },
      { input: 'feature/login-system', isValid: true },
      { input: 'bugfix/issue-42', isValid: true },
      { input: 'release/v1.0.0-rc.1', isValid: true },
      { input: 'user/john.doe/task_123', isValid: true },
      { input: 'a', isValid: true },
      { input: 'deeply/nested/branch/hierarchy/name', isValid: true },

      // Empty & Whitespace
      { input: '', isValid: false, expectedErrorKey: 'branch.validationEmpty' },
      { input: '   ', isValid: false, expectedErrorKey: 'branch.validationEmpty' },
      { input: 'feature branch', isValid: false, expectedErrorKey: 'branch.validationNoSpaces' },
      { input: 'feature\tbranch', isValid: false, expectedErrorKey: 'branch.validationNoSpaces' },

      // Illegal edge characters
      { input: '/leading-slash', isValid: false, expectedErrorKey: 'branch.validationNoLeadingTrailingSlash' },
      { input: 'trailing-slash/', isValid: false, expectedErrorKey: 'branch.validationNoLeadingTrailingSlash' },
      { input: '-leading-dash', isValid: false, expectedErrorKey: 'branch.validationNoLeadingTrailingDash' },
      { input: 'trailing-dash-', isValid: false, expectedErrorKey: 'branch.validationNoLeadingTrailingDash' },
      { input: '.leading-dot', isValid: false, expectedErrorKey: 'branch.validationNoLeadingTrailingDot' },
      { input: 'trailing-dot.', isValid: false, expectedErrorKey: 'branch.validationNoLeadingTrailingDot' },

      // Consecutive characters & forbidden sequences
      { input: 'consecutive//slashes', isValid: false, expectedErrorKey: 'branch.validationNoConsecutiveSeparators' },
      { input: 'consecutive..dots', isValid: false, expectedErrorKey: 'branch.validationNoConsecutiveSeparators' },
      { input: 'branch.lock', isValid: false, expectedErrorKey: 'branch.validationCannotEndWithLock' },
      { input: 'branch@{upstream}', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },

      // Forbidden characters
      { input: 'branch~1', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },
      { input: 'branch^2', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },
      { input: 'branch:colon', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },
      { input: 'branch?question', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },
      { input: 'branch*star', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },
      { input: 'branch[bracket', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },
      { input: 'branch\\backslash', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },
      { input: 'branch@alone', isValid: false, expectedErrorKey: 'branch.validationInvalidCharacters' },

      // Duplicate & current branch checks
      { input: 'feature-x', isValid: false, expectedErrorKey: 'branch.validationAlreadyExists', existingBranches: ['feature-x', 'main'] },
      { input: 'current-branch', isValid: false, expectedErrorKey: 'branch.validationSameAsCurrent', currentBranch: 'current-branch' },
      { input: 'new-feature', isValid: true, existingBranches: ['main', 'develop'], currentBranch: 'main' },
    ]

    it.each(testCases)('evaluates branch name: "$input" -> valid: $isValid', ({ input, isValid, expectedErrorKey, existingBranches, currentBranch }) => {
      if (existingBranches || currentBranch) {
        const result = validateBranchNameWithSchema(input, {
          existingBranchNames: existingBranches,
          currentBranchName: currentBranch,
        })
        expect(result.isValid).toBe(isValid)
        if (expectedErrorKey) {
          expect(result.errorKey).toBe(expectedErrorKey)
        }
      } else {
        const parseResult = gitBranchNameSchema.safeParse(input)
        expect(parseResult.success).toBe(isValid)
        if (!isValid && expectedErrorKey && !parseResult.success) {
          expect(parseResult.error.issues[0]?.message).toBe(expectedErrorKey)
        }
      }
    })
  })

  describe('Remote Name & URL Matrix', () => {
    interface RemoteTestCase {
      name: string
      url: string
      isValid: boolean
      expectedErrorKey?: string
      existingRemotes?: string[]
    }

    const remoteTestCases: RemoteTestCase[] = [
      // Valid combinations
      { name: 'origin', url: 'https://github.com/owner/repository.git', isValid: true },
      { name: 'upstream', url: 'git@github.com:owner/repository.git', isValid: true },
      { name: 'fork-2026', url: 'ssh://git@gitlab.company.internal:2222/org/project.git', isValid: true },
      { name: 'local_mirror', url: '/var/repos/mirror.git', isValid: true },
      { name: 'backup', url: 'git://git.kernel.org/pub/scm/git/git.git', isValid: true },

      // Invalid Remote Names
      { name: '', url: 'https://github.com/owner/repo.git', isValid: false, expectedErrorKey: 'remote.validationEmptyName' },
      { name: '   ', url: 'https://github.com/owner/repo.git', isValid: false, expectedErrorKey: 'remote.validationEmptyName' },
      { name: 'remote with spaces', url: 'https://github.com/owner/repo.git', isValid: false, expectedErrorKey: 'remote.validationInvalidName' },
      { name: '.', url: 'https://github.com/owner/repo.git', isValid: false, expectedErrorKey: 'remote.validationInvalidName' },
      { name: '..', url: 'https://github.com/owner/repo.git', isValid: false, expectedErrorKey: 'remote.validationInvalidName' },
      { name: 'origin', url: 'https://github.com/owner/repo.git', isValid: false, expectedErrorKey: 'remote.validationDuplicateName', existingRemotes: ['origin', 'upstream'] },

      // Invalid Remote URLs
      { name: 'origin', url: '', isValid: false, expectedErrorKey: 'remote.validationEmptyUrl' },
      { name: 'origin', url: '   ', isValid: false, expectedErrorKey: 'remote.validationEmptyUrl' },
      { name: 'origin', url: 'ftp://unsupported.server.com/repo.git', isValid: false, expectedErrorKey: 'remote.validationEmptyUrl' },
      { name: 'origin', url: 'just a plain string', isValid: false, expectedErrorKey: 'remote.validationEmptyUrl' },
    ]

    it.each(remoteTestCases)('evaluates remote: name="$name" url="$url" -> valid: $isValid', ({ name, url, isValid, expectedErrorKey, existingRemotes }) => {
      const result = validateRemoteWithSchema(name, url, { existingRemoteNames: existingRemotes })
      expect(result.isValid).toBe(isValid)
      if (expectedErrorKey) {
        expect(result.errorKey).toBe(expectedErrorKey)
      }
    })
  })

  describe('Commit Input Schema Matrix', () => {
    interface CommitTestCase {
      summary: string
      description?: string
      isAmend?: boolean
      isValid: boolean
      expectedErrorKey?: string
    }

    const commitTestCases: CommitTestCase[] = [
      { summary: 'feat: add interactive rebase dialog', isValid: true },
      { summary: 'fix(parser): handle CRLF newlines', description: 'Detailed breakdown of the fix.', isValid: true },
      { summary: 'امضای تغییرات جدید به زبان فارسی', isValid: true },
      { summary: 'docs: update README', isAmend: true, isValid: true },
      { summary: '', isValid: false, expectedErrorKey: 'commit.validationEmptySummary' },
      { summary: '   \n  \t ', isValid: false, expectedErrorKey: 'commit.validationEmptySummary' },
      { summary: 'a'.repeat(501), isValid: false, expectedErrorKey: 'commit.validationSummaryTooLong' },
    ]

    it.each(commitTestCases)('evaluates commit input: "$summary" -> valid: $isValid', ({ summary, description, isAmend, isValid, expectedErrorKey }) => {
      const result = commitInputSchema.safeParse({
        summary,
        description,
        isAmend,
      })
      expect(result.success).toBe(isValid)
      if (!isValid && expectedErrorKey && !result.success) {
        expect(result.error.issues[0]?.message).toBe(expectedErrorKey)
      }
    })
  })

  describe('Stash Input Schema Matrix', () => {
    interface StashTestCase {
      message?: string
      shouldIncludeUntracked?: boolean
      shouldKeepIndex?: boolean
      isValid: boolean
      expectedErrorKey?: string
    }

    const stashTestCases: StashTestCase[] = [
      { message: undefined, isValid: true },
      { message: '', isValid: true },
      { message: 'WIP on feature branch', isValid: true },
      { message: 'نگهداری موقت تغییرات', shouldIncludeUntracked: true, shouldKeepIndex: true, isValid: true },
      { message: 'a'.repeat(501), isValid: false, expectedErrorKey: 'stash.validationMessageTooLong' },
    ]

    it.each(stashTestCases)('evaluates stash schema: message="$message" -> valid: $isValid', ({ message, shouldIncludeUntracked, shouldKeepIndex, isValid, expectedErrorKey }) => {
      const result = createStashInputSchema.safeParse({
        message,
        shouldIncludeUntracked,
        shouldKeepIndex,
      })
      expect(result.success).toBe(isValid)
      if (!isValid && expectedErrorKey && !result.success) {
        expect(result.error.issues[0]?.message).toBe(expectedErrorKey)
      }
    })
  })

  describe('Pull Request & Token Matrix', () => {
    interface PrTestCase {
      title: string
      sourceBranch: string
      targetBranch: string
      isValid: boolean
      expectedErrorKey?: string
    }

    const prTestCases: PrTestCase[] = [
      { title: 'feat: add commit graph', sourceBranch: 'feature/graph', targetBranch: 'main', isValid: true },
      { title: 'fix: resolve race condition', sourceBranch: 'bugfix/race', targetBranch: 'develop', isValid: true },
      { title: '', sourceBranch: 'feature/x', targetBranch: 'main', isValid: false, expectedErrorKey: 'provider.validationEmptyTitle' },
      { title: '   ', sourceBranch: 'feature/x', targetBranch: 'main', isValid: false, expectedErrorKey: 'provider.validationEmptyTitle' },
      { title: 'a'.repeat(256), sourceBranch: 'feature/x', targetBranch: 'main', isValid: false, expectedErrorKey: 'provider.validationTitleTooLong' },
      { title: 'Valid title', sourceBranch: '', targetBranch: 'main', isValid: false, expectedErrorKey: 'branch.validationEmpty' },
      { title: 'Valid title', sourceBranch: 'feature', targetBranch: '', isValid: false, expectedErrorKey: 'branch.validationEmpty' },
    ]

    it.each(prTestCases)('evaluates PR schema: title="$title" -> valid: $isValid', ({ title, sourceBranch, targetBranch, isValid, expectedErrorKey }) => {
      const result = createPullRequestInputSchema.safeParse({
        title,
        sourceBranch,
        targetBranch,
      })
      expect(result.success).toBe(isValid)
      if (!isValid && expectedErrorKey && !result.success) {
        expect(result.error.issues[0]?.message).toBe(expectedErrorKey)
      }
    })

    it('evaluates personal access token requirements', () => {
      // GitHub token
      expect(gitHubTokenSchema.safeParse('ghp_12345678901234567890').success).toBe(true)
      expect(gitHubTokenSchema.safeParse('short_token').success).toBe(false)
      expect(gitHubTokenSchema.safeParse('token with spaces inside it!').success).toBe(false)

      // GitLab token
      expect(gitLabTokenSchema.safeParse('glpat-12345678901234567890').success).toBe(true)
      expect(gitLabTokenSchema.safeParse('too_short').success).toBe(false)

      // Instance URLs
      expect(instanceUrlSchema.safeParse('https://github.com').success).toBe(true)
      expect(instanceUrlSchema.safeParse('https://gitlab.mycompany.org:8443').success).toBe(true)
      expect(instanceUrlSchema.safeParse('ftp://bad-scheme.com').success).toBe(false)
      expect(instanceUrlSchema.safeParse('not a url').success).toBe(false)
    })
  })
})
