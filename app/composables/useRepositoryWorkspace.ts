import { formatApplicationError } from '~/utils/tauriClient'
import type {
  DirectoryListing,
  GitBinaryInformation,
  RecentRepositoryRecord,
  RepositoryOverview,
} from '~/types/repository'
import {
  LAST_OPENED_REPOSITORY_STORAGE_KEY,
  OPEN_REPOSITORY_TABS_STORAGE_KEY,
  useWorkspaceTabController,
} from '~/composables/useWorkspaceTabController'

export function useRepositoryWorkspace() {
  const currentRepository = useState<RepositoryOverview | null>('workspace-current-repository', () => null)
  const recentRepositories = useState<RecentRepositoryRecord[]>('workspace-recent-repositories', () => [])
  const gitBinary = useState<GitBinaryInformation | null>('workspace-git-binary', () => null)
  const isLoadingRepository = useState<boolean>('workspace-is-loading-repository', () => false)
  const isOperating = useState<boolean>('workspace-is-operating', () => false)
  const lastError = useState<string | null>('workspace-last-error', () => null)

  const { notifyError, notifySuccess } = useNotificationService()
  const { t } = useI18n()

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function discoverGit(customPath?: string): Promise<GitBinaryInformation | null> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const info = await invoke<GitBinaryInformation>('discover_git_binary', {
          customPath: customPath || null,
        })
        gitBinary.value = info
        return info
      } else {
        const mockBinary: GitBinaryInformation = {
          executable_path: '/usr/bin/git',
          version_string: 'git version 2.43.0 (browser preview)',
          is_valid: true,
          has_porcelain_v2_support: true,
        }
        gitBinary.value = mockBinary
        return mockBinary
      }
    } catch (error) {
      gitBinary.value = {
        executable_path: '',
        version_string: '',
        is_valid: false,
        has_porcelain_v2_support: false,
      }
      return null
    }
  }

  async function fetchRecentRepositories(): Promise<RecentRepositoryRecord[]> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<RecentRepositoryRecord[]>('get_recent_repositories')
        recentRepositories.value = records
        return records
      } else {
        const mockRecent: RecentRepositoryRecord[] = [
          {
            id: 'mock-repo-1',
            path: '/home/farshid/Desktop/self/gitverse',
            name: 'gitverse',
            last_opened_timestamp: Math.floor(Date.now() / 1000),
            is_pinned: false,
          },
        ]
        recentRepositories.value = mockRecent
        return mockRecent
      }
    } catch (error) {
      console.warn('Failed to fetch recent repositories:', error)
      return []
    }
  }

  async function removeRecentRepository(directoryPath: string): Promise<void> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('remove_recent_repository', { directoryPath })
      }
      recentRepositories.value = recentRepositories.value.filter(
        (record) => record.path !== directoryPath
      )
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    }
  }

  async function clearRecentRepositories(): Promise<void> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('clear_recent_repositories')
      }
      recentRepositories.value = []
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    }
  }

  async function openRepository(directoryPath: string): Promise<RepositoryOverview | null> {
    isLoadingRepository.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const overview = await invoke<RepositoryOverview>('open_repository', {
          directoryPath,
        })
        currentRepository.value = overview
        await fetchRecentRepositories()

        if (import.meta.client) {
          try {
            localStorage.setItem(LAST_OPENED_REPOSITORY_STORAGE_KEY, directoryPath)
          } catch (storageError) {
            console.error('Failed to save last opened repository:', storageError)
          }
        }
        return overview
      } else {
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
    } catch (error) {
      const rawError = error as { type?: string; message?: string } | Error | string
      const isInvalidRepo =
        (typeof rawError === 'object' && rawError !== null && 'type' in rawError && rawError.type === 'InvalidRepository') ||
        (typeof rawError === 'string' && rawError.includes('InvalidRepository')) ||
        (rawError instanceof Error && rawError.message.includes('InvalidRepository'))

      if (isInvalidRepo) {
        lastError.value = t('repository.notGitRepository')
        notifyError(t('repository.notGitRepository'), t('repository.notGitRepositoryDetails'))
      } else {
        const errorMessage = error instanceof Error ? error.message : String(error)
        lastError.value = errorMessage
        notifyError(t('common.error'), errorMessage)
      }
      return null
    } finally {
      isLoadingRepository.value = false
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
        const samplePath = '/home/farshid/Desktop/self/gitverse'
        return await openRepository(samplePath)
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Directory picker failed'
      lastError.value = errorMessage
      notifyError(t('common.error'), errorMessage)
      return null
    }
  }

  async function closeRepository(): Promise<void> {
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('close_repository')
    }
    if (import.meta.client) {
      try {
        localStorage.removeItem(LAST_OPENED_REPOSITORY_STORAGE_KEY)
        localStorage.removeItem(OPEN_REPOSITORY_TABS_STORAGE_KEY)
      } catch {
        // ignore
      }
    }
    currentRepository.value = null
  }

  async function restoreLastOpenedRepository(): Promise<RepositoryOverview | null> {
    if (!import.meta.client) return null

    let savedPath: string | null = null
    let savedTabsJson: string | null = null
    try {
      savedPath = localStorage.getItem(LAST_OPENED_REPOSITORY_STORAGE_KEY)
      savedTabsJson = localStorage.getItem(OPEN_REPOSITORY_TABS_STORAGE_KEY)
    } catch {
      // ignore
    }

    if (!savedPath) {
      return null
    }

    if (savedTabsJson) {
      try {
        const savedTabs = JSON.parse(savedTabsJson) as Array<{ path: string; name: string }>
        if (Array.isArray(savedTabs) && savedTabs.length > 0) {
          const { openRepositoryTab, activateRepositoryTab } = useWorkspaceTabController()
          for (const tab of savedTabs) {
            openRepositoryTab(tab.path, tab.name)
          }
          if (savedPath) {
            activateRepositoryTab(savedPath)
          }
        }
      } catch {
        // ignore
      }
    }

    try {
      const overview = await openRepository(savedPath)
      if (overview) return overview
    } catch (error) {
      console.warn('Failed to restore last opened repository from path:', savedPath, error)
      try {
        localStorage.removeItem(LAST_OPENED_REPOSITORY_STORAGE_KEY)
        localStorage.removeItem(OPEN_REPOSITORY_TABS_STORAGE_KEY)
      } catch {
        // ignore
      }
    }

    return null
  }

  function clearError(): void {
    lastError.value = null
  }

  async function listDirectoryFolders(directoryPath?: string): Promise<DirectoryListing | null> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        return await invoke<DirectoryListing>('list_directory_folders', {
          directoryPath: directoryPath?.trim() || null,
        })
      } else {
        // Mock browser listing
        const mockFolders = [
          { name: 'gitverse', path: '/home/farshid/Desktop/self/gitverse', is_git_repository: true },
          { name: 'carasan', path: '/home/farshid/Desktop/self/carasan', is_git_repository: true },
          { name: 'notes', path: '/home/farshid/Desktop/self/notes', is_git_repository: false },
        ]
        const mockFiles = [
          { name: 'fix-login.patch', path: '/home/farshid/Desktop/self/fix-login.patch', extension: 'patch', size_bytes: 1420 },
          { name: 'backup.bundle', path: '/home/farshid/Desktop/self/backup.bundle', extension: 'bundle', size_bytes: 52400 },
          { name: 'README.md', path: '/home/farshid/Desktop/self/README.md', extension: 'md', size_bytes: 3820 },
        ]
        return {
          current_path: directoryPath || '/home/farshid/Desktop/self',
          parent_path: '/home/farshid/Desktop',
          folders: mockFolders,
          files: mockFiles,
          is_current_git_repository: false,
        }
      }
    } catch (error) {
      notifyError(t('common.error'), formatApplicationError(error))
      return null
    }
  }

  async function readFileText(filePath: string): Promise<string | null> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        return await invoke<string>('read_file_text', {
          filePath,
        })
      } else {
        return '// Mock file text content'
      }
    } catch (error) {
      notifyError(t('common.error'), formatApplicationError(error))
      return null
    }
  }

  return {
    currentRepository,
    recentRepositories,
    gitBinary,
    isLoadingRepository,
    isOperating,
    lastError,
    discoverGit,
    fetchRecentRepositories,
    removeRecentRepository,
    clearRecentRepositories,
    openRepository,
    promptAndOpenRepository,
    listDirectoryFolders,
    readFileText,
    closeRepository,
    restoreLastOpenedRepository,
    clearError,
  }
}
