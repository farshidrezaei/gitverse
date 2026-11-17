import type { RepositoryReferences } from '~/types/repository'

export function useBranchController() {
  const { currentRepository } = useRepositoryWorkspace()
  const repositoryReferences = useState<RepositoryReferences | null>(
    'workspace-references',
    () => null
  )
  const favoriteBranches = useState<string[]>('workspace-favorite-branches', () => [])
  const isOperatingBranches = useState<boolean>('workspace-is-operating-branches', () => false)

  const { pushAction } = useActionHistory()
  const { notifySuccess, notifyError, notifyUndoable } = useNotificationService()
  const { t } = useI18n()

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  function getFavoritesStorageKey(): string {
    return `gitverse-favorites-${currentRepository.value?.path || 'default'}`
  }

  function loadFavoriteBranches(): void {
    if (import.meta.client && currentRepository.value) {
      try {
        const stored = localStorage.getItem(getFavoritesStorageKey())
        favoriteBranches.value = stored ? JSON.parse(stored) : []
      } catch (error) {
        console.error('Failed to load favorite branches:', error)
        favoriteBranches.value = []
      }
    }
  }

  function toggleFavoriteBranch(branchName: string): void {
    const set = new Set(favoriteBranches.value)
    if (set.has(branchName)) {
      set.delete(branchName)
    } else {
      set.add(branchName)
    }
    favoriteBranches.value = Array.from(set)
    if (import.meta.client) {
      try {
        localStorage.setItem(getFavoritesStorageKey(), JSON.stringify(favoriteBranches.value))
      } catch (error) {
        console.error('Failed to save favorite branches:', error)
      }
    }
  }

  function isFavoriteBranch(branchName: string): boolean {
    return favoriteBranches.value.includes(branchName)
  }

  async function fetchReferences(): Promise<RepositoryReferences | null> {
    if (!currentRepository.value) {
      repositoryReferences.value = null
      return null
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const references = await invoke<RepositoryReferences>('get_repository_references', {
          directoryPath: currentRepository.value.path,
        })
        repositoryReferences.value = references
        return references
      } else {
        const mockReferences: RepositoryReferences = {
          branches: [
            {
              name: 'master',
              is_head: true,
              upstream_name: 'origin/master',
              ahead_count: 0,
              behind_count: 0,
              head_commit_hash: '5897482',
            },
            {
              name: 'feature/diff-viewer',
              is_head: false,
              upstream_name: null,
              ahead_count: 0,
              behind_count: 0,
              head_commit_hash: 'a991385',
            },
          ],
          remote_branches: [
            { name: 'origin/master', remote_name: 'origin', head_commit_hash: '5897482' },
          ],
          tags: [{ name: 'v0.1.0-alpha', commit_hash: '5897482' }],
          remotes: [
            {
              name: 'origin',
              fetch_url: 'https://github.com/user/gitverse.git',
              push_url: 'https://github.com/user/gitverse.git',
            },
          ],
        }
        repositoryReferences.value = mockReferences
        return mockReferences
      }
    } catch (error) {
      console.error('Failed to fetch references:', error)
      return null
    }
  }

  async function checkoutBranch(
    branchName: string,
    createIfMissing = false,
    startPoint?: string
  ): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperatingBranches.value = true

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('checkout_branch', {
          branchName,
          createIfMissing,
          startPoint: startPoint || null,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchReferences()
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : `Failed to checkout ${branchName}`
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingBranches.value = false
    }
  }

  async function createBranch(branchName: string, startPoint?: string): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperatingBranches.value = true

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('create_branch', {
          branchName,
          startPoint: startPoint || null,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchReferences()
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : `Failed to create branch ${branchName}`
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingBranches.value = false
    }
  }

  async function deleteBranch(branchName: string, force = false): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperatingBranches.value = true

    // Find previous commit hash of branch before deletion to enable safe Undo
    const existingBranch = repositoryReferences.value?.branches.find(
      (b) => b.name === branchName
    )
    const savedHeadCommitHash = existingBranch?.head_commit_hash

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('delete_branch', {
          branchName,
          force,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchReferences()

      // Undo function: recreate branch at exact commit hash
      const undoAction = async () => {
        const restored = await createBranch(branchName, savedHeadCommitHash || undefined)
        if (restored) {
          notifySuccess(t('notifications.branchRestored', { name: branchName }))
        }
      }

      pushAction({
        title: t('notifications.branchDeleted', { name: branchName }),
        undo: undoAction,
        redo: async () => {
          await deleteBranch(branchName, true)
        },
      })

      // Toast with actionable Undo button
      notifyUndoable(
        t('notifications.branchDeleted', { name: branchName }),
        savedHeadCommitHash ? `Commit: ${savedHeadCommitHash.slice(0, 7)}` : undefined,
        t('notifications.undo'),
        undoAction
      )

      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : `Failed to delete branch ${branchName}`
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingBranches.value = false
    }
  }

  return {
    repositoryReferences,
    favoriteBranches,
    isOperatingBranches,
    loadFavoriteBranches,
    toggleFavoriteBranch,
    isFavoriteBranch,
    fetchReferences,
    checkoutBranch,
    createBranch,
    deleteBranch,
  }
}
