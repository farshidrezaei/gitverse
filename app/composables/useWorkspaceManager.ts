import type {
  GitBinaryInformation,
  RecentRepositoryRecord,
  RepositoryOverview,
} from '~/types/repository'

export function useWorkspaceManager() {
  const currentRepository = useState<RepositoryOverview | null>('workspace-current-repository', () => null)
  const recentRepositories = useState<RecentRepositoryRecord[]>('workspace-recent-repositories', () => [])
  const gitBinary = useState<GitBinaryInformation | null>('workspace-git-binary', () => null)
  const isLoading = ref<boolean>(false)
  const lastError = ref<string | null>(null)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function discoverGit(customPath?: string): Promise<GitBinaryInformation | null> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<GitBinaryInformation>('discover_git_binary', {
          customPath: customPath || null,
        })
        gitBinary.value = result
        return result
      } else {
        // Web / dev mode mock
        const mock: GitBinaryInformation = {
          executable_path: 'git',
          version_string: 'git version 2.43.0 (Mock/Web Preview)',
          is_valid: true,
          has_porcelain_v2_support: true,
        }
        gitBinary.value = mock
        return mock
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Failed to discover Git'
      return null
    }
  }

  async function fetchRecentRepositories(): Promise<RecentRepositoryRecord[]> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const list = await invoke<RecentRepositoryRecord[]>('get_recent_repositories', { limit: 20 })
        recentRepositories.value = list
        return list
      } else {
        // Fallback or demo repos
        recentRepositories.value = [
          {
            id: 'mock-gitverse',
            name: 'gitverse',
            path: '/home/farshid/Desktop/self/gitverse',
            last_opened_timestamp: Date.now(),
            is_pinned: true,
          },
        ]
        return recentRepositories.value
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Failed to fetch recents'
      return []
    }
  }

  async function openRepository(directoryPath: string): Promise<RepositoryOverview | null> {
    isLoading.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const overview = await invoke<RepositoryOverview>('open_repository', {
          directoryPath,
        })
        currentRepository.value = overview
        await fetchRecentRepositories()
        return overview
      } else {
        // Web preview mock
        const overview: RepositoryOverview = {
          path: directoryPath,
          name: directoryPath.split('/').filter(Boolean).pop() || 'repository',
          current_branch_name: 'master',
          head_commit_hash: '3ad5d4c9f1a2',
          is_empty: false,
          is_detached_head: false,
          working_tree_is_clean: true,
          ahead_count: 0,
          behind_count: 0,
        }
        currentRepository.value = overview
        return overview
      }
    } catch (error: any) {
      lastError.value = error?.message || `Failed to open ${directoryPath}`
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function closeRepository(): Promise<void> {
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('close_repository')
    }
    currentRepository.value = null
  }

  return {
    currentRepository,
    recentRepositories,
    gitBinary,
    isLoading,
    lastError,
    discoverGit,
    fetchRecentRepositories,
    openRepository,
    closeRepository,
  }
}
