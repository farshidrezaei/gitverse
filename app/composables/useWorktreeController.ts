import type { GitWorktreeSummary, RepositoryOverview } from '~/types/repository'

function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useWorktreeController() {
  const currentRepository = tryUseNuxtApp()
    ? useRepositoryWorkspace().currentRepository
    : ref<RepositoryOverview | null>(null)

  const worktreeList = safeState<GitWorktreeSummary[]>('workspace-worktree-list', () => [])
  const isLoadingWorktrees = safeState<boolean>('workspace-is-loading-worktrees', () => false)

  function notifySuccess(message: string): void {
    if (tryUseNuxtApp()) {
      useNotificationService().notifySuccess(message)
    }
  }

  function notifyError(message: string): void {
    if (tryUseNuxtApp()) {
      useNotificationService().notifyError(message)
    }
  }

  function translate(key: string, parameters?: Record<string, unknown>): string {
    if (tryUseNuxtApp()) {
      return useI18n().t(key, parameters || {})
    }
    return key
  }

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchWorktrees(): Promise<GitWorktreeSummary[]> {
    if (!currentRepository.value) {
      worktreeList.value = []
      return []
    }

    try {
      isLoadingWorktrees.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<GitWorktreeSummary[]>('list_worktrees', {
          directoryPath: currentRepository.value.path,
        })
        worktreeList.value = records
        return records
      } else {
        // Fallback for browser mock mode
        if (worktreeList.value.length === 0) {
          worktreeList.value = [
            {
              directory_path: currentRepository.value.path,
              head_commit_hash: currentRepository.value.head_commit_hash || 'abc1234',
              branch_name: currentRepository.value.current_branch_name,
              is_bare: false,
              is_locked: false,
              lock_reason: null,
              is_prunable: false,
              is_main: true,
            },
          ]
        }
        return worktreeList.value
      }
    } catch (error) {
      console.error('Failed to fetch worktrees:', error)
      notifyError(translate('worktree.fetchFailed'))
      return []
    } finally {
      isLoadingWorktrees.value = false
    }
  }

  async function addWorktree(
    worktreePath: string,
    branchName?: string,
    isNewBranch: boolean = false,
  ): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isLoadingWorktrees.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('add_worktree', {
          directoryPath: currentRepository.value.path,
          worktreePath,
          branchName: branchName || null,
          isNewBranch,
        })
      } else {
        worktreeList.value.push({
          directory_path: worktreePath,
          head_commit_hash: 'mockhead123',
          branch_name: branchName || 'new-worktree',
          is_bare: false,
          is_locked: false,
          lock_reason: null,
          is_prunable: false,
          is_main: false,
        })
      }

      notifySuccess(translate('worktree.addSuccess', { path: worktreePath }))
      await fetchWorktrees()
      return true
    } catch (error) {
      console.error('Failed to add worktree:', error)
      notifyError(translate('worktree.addFailed', { error: String(error) }))
      return false
    } finally {
      isLoadingWorktrees.value = false
    }
  }

  async function removeWorktree(
    worktreePath: string,
    force: boolean = false,
  ): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isLoadingWorktrees.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('remove_worktree', {
          directoryPath: currentRepository.value.path,
          worktreePath,
          force,
        })
      } else {
        worktreeList.value = worktreeList.value.filter(
          item => item.directory_path !== worktreePath,
        )
      }

      notifySuccess(translate('worktree.removeSuccess', { path: worktreePath }))
      await fetchWorktrees()
      return true
    } catch (error) {
      console.error('Failed to remove worktree:', error)
      notifyError(translate('worktree.removeFailed', { error: String(error) }))
      return false
    } finally {
      isLoadingWorktrees.value = false
    }
  }

  return {
    worktreeList,
    isLoadingWorktrees,
    fetchWorktrees,
    addWorktree,
    removeWorktree,
  }
}
