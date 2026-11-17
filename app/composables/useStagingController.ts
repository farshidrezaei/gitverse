import type { DetailedRepositoryStatus } from '~/types/repository'

export interface SelectedDiffTarget {
  filePath: string
  isStaged: boolean
}

let unlistenStatusWatcher: (() => void) | null = null

export function useStagingController() {
  const { currentRepository } = useRepositoryWorkspace()
  const repositoryStatus = useState<DetailedRepositoryStatus | null>(
    'workspace-repository-status',
    () => null
  )
  const selectedDiffTarget = useState<SelectedDiffTarget | null>(
    'workspace-selected-diff-target',
    () => null
  )
  const stagingViewMode = useState<'tree' | 'flat'>(
    'workspace-staging-view-mode',
    () => 'tree'
  )
  const isOperatingStaging = useState<boolean>('workspace-is-operating-staging', () => false)

  const { pushAction } = useActionHistory()
  const { notifySuccess, notifyError, notifyUndoable } = useNotificationService()
  const { t } = useI18n()

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
        return status
      } else {
        const mockStatus: DetailedRepositoryStatus = {
          branch_name: 'master',
          head_commit_hash: '5897482',
          upstream_branch_name: 'origin/master',
          ahead_count: 0,
          behind_count: 0,
          is_detached_head: false,
          staged_files: [
            {
              path: 'app/composables/useActionHistory.ts',
              original_path: null,
              status: 'added',
              is_staged: true,
            },
          ],
          unstaged_files: [
            {
              path: 'app/pages/index.vue',
              original_path: null,
              status: 'modified',
              is_staged: false,
            },
          ],
          conflicted_files: [],
          operation_state: 'none',
        }
        repositoryStatus.value = mockStatus
        return mockStatus
      }
    } catch (error) {
      console.error('Failed to fetch repository status:', error)
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

  async function stageFile(filePath: string, recordHistory = true): Promise<void> {
    if (!currentRepository.value) return
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('stage_file', {
          filePath,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()

      if (recordHistory) {
        pushAction({
          title: t('notifications.fileStaged', { path: filePath }),
          undo: async () => {
            await unstageFile(filePath, false)
            notifySuccess(t('notifications.fileUnstaged', { path: filePath }))
          },
          redo: async () => {
            await stageFile(filePath, false)
            notifySuccess(t('notifications.fileStaged', { path: filePath }))
          },
        })
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    }
  }

  async function unstageFile(filePath: string, recordHistory = true): Promise<void> {
    if (!currentRepository.value) return
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('unstage_file', {
          filePath,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()

      if (recordHistory) {
        pushAction({
          title: t('notifications.fileUnstaged', { path: filePath }),
          undo: async () => {
            await stageFile(filePath, false)
            notifySuccess(t('notifications.fileStaged', { path: filePath }))
          },
          redo: async () => {
            await unstageFile(filePath, false)
            notifySuccess(t('notifications.fileUnstaged', { path: filePath }))
          },
        })
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    }
  }

  async function stageAll(recordHistory = true): Promise<void> {
    if (!currentRepository.value) return
    isOperatingStaging.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('stage_all', {
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()

      if (recordHistory) {
        pushAction({
          title: t('notifications.allStaged'),
          undo: async () => {
            await unstageAll(false)
            notifySuccess(t('notifications.allUnstaged'))
          },
          redo: async () => {
            await stageAll(false)
            notifySuccess(t('notifications.allStaged'))
          },
        })
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    } finally {
      isOperatingStaging.value = false
    }
  }

  async function unstageAll(recordHistory = true): Promise<void> {
    if (!currentRepository.value) return
    isOperatingStaging.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('unstage_all', {
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()

      if (recordHistory) {
        pushAction({
          title: t('notifications.allUnstaged'),
          undo: async () => {
            await stageAll(false)
            notifySuccess(t('notifications.allStaged'))
          },
          redo: async () => {
            await unstageAll(false)
            notifySuccess(t('notifications.allUnstaged'))
          },
        })
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    } finally {
      isOperatingStaging.value = false
    }
  }

  async function discardFile(filePath: string, isUntracked: boolean = false): Promise<void> {
    if (!currentRepository.value) return
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('discard_file_changes', {
          filePath,
          isUntracked,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()
      if (selectedDiffTarget.value?.filePath === filePath) {
        clearDiffSelection()
      }
      notifySuccess(t('notifications.fileDiscarded', { path: filePath }))
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    }
  }

  async function discardAllUnstaged(): Promise<void> {
    if (!currentRepository.value || !repositoryStatus.value) return
    const unstagedFiles = repositoryStatus.value.unstaged_files || []
    if (unstagedFiles.length === 0) return

    isOperatingStaging.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        for (const file of unstagedFiles) {
          await invoke('discard_file_changes', {
            filePath: file.path,
            isUntracked: file.status === 'untracked',
            directoryPath: currentRepository.value.path,
          })
        }
      }
      await fetchStatus()
      clearDiffSelection()
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
    } finally {
      isOperatingStaging.value = false
    }
  }

  async function commitChanges(message: string, isAmend: boolean): Promise<boolean> {
    if (!currentRepository.value || !message.trim()) return false
    try {
      let commitHash = ''
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        commitHash = await invoke<string>('commit', {
          message,
          isAmend,
          directoryPath: currentRepository.value.path,
        })
      } else {
        commitHash = 'mock' + Math.random().toString(16).substring(2, 10)
      }

      await fetchStatus()
      clearDiffSelection()

      notifySuccess(
        t('notifications.commitSuccess', { hash: commitHash.slice(0, 7) })
      )
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      return false
    }
  }

  function clearDiffSelection(): void {
    selectedDiffTarget.value = null
  }

  return {
    repositoryStatus,
    selectedDiffTarget,
    stagingViewMode,
    isOperatingStaging,
    fetchStatus,
    setupRealtimeListener,
    stageFile,
    unstageFile,
    stageAll,
    unstageAll,
    discardFile,
    discardAllUnstaged,
    commitChanges,
    clearDiffSelection,
  }
}
