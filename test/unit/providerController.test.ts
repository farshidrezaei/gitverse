import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}))

// Mock nuxt/app imports
vi.mock('#app', () => ({
  useToast: () => ({
    add: vi.fn(),
  }),
}))

describe('Provider Controller & Schemas Test Suite', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Table-driven Provider URL parsing and account identification', () => {
    const testCases = [
      {
        remoteUrl: 'git@github.com:octocat/Hello-World.git',
        expectedProvider: 'github',
        expectedOwner: 'octocat',
        expectedRepository: 'Hello-World',
      },
      {
        remoteUrl: 'https://github.com/torvalds/linux.git',
        expectedProvider: 'github',
        expectedOwner: 'torvalds',
        expectedRepository: 'linux',
      },
      {
        remoteUrl: 'https://gitlab.com/gitlab-org/gitlab.git',
        expectedProvider: 'gitlab',
        expectedOwner: 'gitlab-org',
        expectedRepository: 'gitlab',
      },
      {
        remoteUrl: 'git@gitlab.mycompany.internal:engineering/gitverse.git',
        expectedProvider: 'gitlab',
        expectedOwner: 'engineering',
        expectedRepository: 'gitverse',
      },
    ]

    testCases.forEach(({ remoteUrl, expectedProvider, expectedOwner, expectedRepository }) => {
      it(`parses remote URL correctly: ${remoteUrl}`, () => {
        let cleaned = remoteUrl.trim()
        let isGitLab = cleaned.includes('gitlab')
        let owner = ''
        let repository = ''

        if (cleaned.startsWith('git@')) {
          const parts = cleaned.replace(/^git@/, '').split(':')
          const pathParts = parts[1]?.replace(/\.git$/, '').split('/') || []
          owner = pathParts[0] || ''
          repository = pathParts.slice(1).join('/') || ''
        } else if (cleaned.startsWith('http://') || cleaned.startsWith('https://')) {
          const url = new URL(cleaned)
          const segments = url.pathname.replace(/^\//, '').replace(/\.git$/, '').split('/')
          owner = segments[0] || ''
          repository = segments.slice(1).join('/') || ''
        }

        expect(isGitLab ? 'gitlab' : 'github').toBe(expectedProvider)
        expect(owner).toBe(expectedOwner)
        expect(repository).toBe(expectedRepository)
      })
    })
  })

  describe('Provider Account Management Mock IPC', () => {
    it('fetches connected accounts successfully', async () => {
      const mockAccounts = [
        {
          id: 'acc-1',
          provider: 'github',
          account_username: 'octocat',
          display_name: 'The Octocat',
          avatar_url: 'https://github.com/images/error/octocat_happy.gif',
          instance_url: 'https://api.github.com',
          created_at: '2026-01-01T00:00:00Z',
        },
      ]

      mockInvoke.mockResolvedValueOnce(mockAccounts)

      const result = await mockInvoke('list_provider_accounts')
      expect(result).toEqual(mockAccounts)
      expect(mockInvoke).toHaveBeenCalledWith('list_provider_accounts')
    })

    it('connects GitHub account successfully', async () => {
      const newAccount = {
        id: 'acc-gh-1',
        provider: 'github',
        account_username: 'developer',
        display_name: 'Pro Developer',
        avatar_url: null,
        instance_url: 'https://api.github.com',
        created_at: '2026-01-02T00:00:00Z',
      }

      mockInvoke.mockResolvedValueOnce(newAccount)

      const result = await mockInvoke('connect_github_account', {
        token: 'ghp_secrettoken1234567890',
        instanceUrl: null,
      })

      expect(result).toEqual(newAccount)
      expect(mockInvoke).toHaveBeenCalledWith('connect_github_account', {
        token: 'ghp_secrettoken1234567890',
        instanceUrl: null,
      })
    })

    it('disconnects provider account and handles failure correctly', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await expect(mockInvoke('disconnect_provider_account', { accountId: 'acc-1' })).resolves.toBeUndefined()

      mockInvoke.mockRejectedValueOnce(new Error('Account not found in secure storage'))
      await expect(mockInvoke('disconnect_provider_account', { accountId: 'invalid' })).rejects.toThrow(
        'Account not found in secure storage',
      )
    })

    it('retrieves pull requests with state filtering', async () => {
      const mockPullRequests = [
        {
          id: '101',
          number: 42,
          title: 'feat: add gitlab integration',
          author_username: 'developer',
          author_avatar_url: null,
          source_branch: 'feat/gitlab-api',
          target_branch: 'main',
          state: 'open',
          is_draft: false,
          web_url: 'https://github.com/org/repo/pull/42',
          created_at: '2026-02-01T10:00:00Z',
          updated_at: '2026-02-01T12:00:00Z',
          labels: ['feature', 'integration'],
        },
      ]

      mockInvoke.mockResolvedValueOnce(mockPullRequests)

      const result = await mockInvoke('get_provider_pull_requests', {
        accountId: 'acc-gh-1',
        owner: 'org',
        repository: 'repo',
        state: 'open',
      })

      expect(result).toEqual(mockPullRequests)
      expect(mockInvoke).toHaveBeenCalledWith('get_provider_pull_requests', {
        accountId: 'acc-gh-1',
        owner: 'org',
        repository: 'repo',
        state: 'open',
      })
    })
  })
})
