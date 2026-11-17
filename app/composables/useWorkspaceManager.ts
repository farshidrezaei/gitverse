import type {
  DetailedRepositoryStatus,
  GitBinaryInformation,
  RecentRepositoryRecord,
  RepositoryOverview,
} from '~/types/repository'

export function useWorkspaceManager() {
  const currentRepository = useState<RepositoryOverview | null>('workspace-current-repository', () => null)
  const repositoryStatus = useState<DetailedRepositoryStatus | null>('workspace-repository-status', () => null)
  const recentRepositories = useState<RecentRepositoryRecord[]>('workspace-recent-repositories', () => [])
  const gitBinary = useState<GitBinaryInformation | null>('workspace-git-binary', () => null)
  const isLoading = ref<boolean>(false)
  const lastError = ref<string | null>(null)

  let unlistenStatusWatcher: (() => void) | null = null

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchStatus(): Promise<DetailedRepositoryStatus | null> {
    if (!currentRepository.value) {
      repositoryStatus.value = null
      return null
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const status = await invoke<DetailedRepositoryStatus>('get_repository_status', {
          directoryPath: currentRepository.value.path,
        })
        repositoryStatus.value = status

        // Keep current repository sync counts updated
        if (currentRepository.value) {
          currentRepository.value.ahead_count = status.ahead_count
          currentRepository.value.behind_count = status.behind_count
          currentRepository.value.working_tree_is_clean =
            status.staged_files.length === 0 &&
            status.unstaged_files.length === 0 &&
            status.conflicted_files.length === 0
          if (status.branch_name) {
            currentRepository.value.current_branch_name = status.branch_name
          }
        }
        return status
      } else {
        // Mock status in dev preview
        const mock: DetailedRepositoryStatus = {
          branch_name: currentRepository.value.current_branch_name || 'master',
          head_commit_hash: 'dab7678166d',
          upstream_branch_name: 'origin/master',
          ahead_count: 0,
          behind_count: 0,
          is_detached_head: false,
          staged_files: [
            { path: 'src-tauri/Cargo.toml', original_path: null, status: 'modified', is_staged: true },
          ],
          unstaged_files: [
            { path: 'app/pages/index.vue', original_path: null, status: 'modified', is_staged: false },
            { path: 'scratch/notes.txt', original_path: null, status: 'untracked', is_staged: false },
          ],
          conflicted_files: [],
        }
        repositoryStatus.value = mock
        return mock
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Failed to fetch status'
      return null
    }
  }

  async function setupRealtimeListener(): Promise<void> {
    if (await isTauriEnvironment()) {
      const { listen } = await import('@tauri-apps/api/event')
      if (unlistenStatusWatcher) {
        unlistenStatusWatcher()
      }
      const unlisten = await listen('repository://status-changed', async () => {
        await fetchStatus()
      })
      unlistenStatusWatcher = unlisten
    }
  }

  async function stageFile(filePath: string): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('stage_file', {
        filePath,
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function unstageFile(filePath: string): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('unstage_file', {
        filePath,
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function stageAll(): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('stage_all', {
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function unstageAll(): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('unstage_all', {
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function discardFile(filePath: string, isUntracked: boolean): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('discard_file_changes', {
        filePath,
        isUntracked,
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function commitChanges(message: string, isAmend: boolean): Promise<boolean> {
    if (!currentRepository.value || !message.trim()) return false
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('commit', {
          message,
          isAmend,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()
      return true
    } catch (error: any) {
      lastError.value = error?.message || 'Commit failed'
      return false
    }
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
        await fetchStatus()
        await setupRealtimeListener()
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

  async function promptAndOpenRepository(): Promise<RepositoryOverview | null> {
    try {
      if (await isTauriEnvironment()) {
        const { open } = await import('@tauri-apps/plugin-dialog')
        const selected = await open({
          directory: true,
          multiple: false,
          title: 'Select Git Repository',
        })
        if (selected && typeof selected === 'string') {
          return await openRepository(selected)
        }
        return null
      } else {
        // Web mock
        const samplePath = '/home/farshid/Desktop/self/gitverse'
        return await openRepository(samplePath)
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Directory picker failed'
      return null
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
    repositoryStatus,
    recentRepositories,
    gitBinary,
    isLoading,
    lastError,
    discoverGit,
    fetchStatus,
    fetchRecentRepositories,
    openRepository,
    promptAndOpenRepository,
    closeRepository,
    stageFile,
    unstageFile,
    stageAll,
    unstageAll,
    discardFile,
    commitChanges,
  }
}
