import type { MergeBranchOutcome, RebaseBranchOutcome, RepositoryReferences } from '~/types/repository'
import { useRepositoryWorkspace } from '~/composables/useRepositoryWorkspace'
import { useActionHistory } from '~/composables/useActionHistory'
import { useNotificationService } from '~/composables/useNotificationService'
import { useConfirmationDialog } from '~/composables/useConfirmationDialog'
import { useStagingController } from '~/composables/useStagingController'
import { useCommitGraph } from '~/composables/useCommitGraph'

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
  const { confirmStandard } = useConfirmationDialog()
  const { repositoryStatus, fetchStatus } = useStagingController()
  const { fetchCommitGraph } = useCommitGraph()
  const { t } = useI18n()

  const currentBranchName = computed(() => {
    return (
      repositoryStatus.value?.branch_name ||
      repositoryReferences.value?.branches.find((branch) => branch.is_head)?.name ||
      'HEAD'
    )
  })

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

  async function mergeBranch(branchName: string, noFastForward = false): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    const targetBranch = currentBranchName.value
    const isConfirmed = await confirmStandard({
      title: t('branch.mergeConfirmTitle', { source: branchName, target: targetBranch }),
      description: t('branch.mergeConfirmDescription', { source: branchName, target: targetBranch }),
      confirmLabel: t('branch.mergeAction'),
      variant: 'primary',
    })
    if (!isConfirmed) {
      return false
    }

    try {
      isOperatingBranches.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const outcome = await invoke<MergeBranchOutcome>('merge_branch', {
          branchName,
          noFastForward,
          directoryPath: currentRepository.value.path,
        })

        if (outcome.is_successful) {
          notifySuccess(t('branch.mergeSuccess', { source: branchName, target: targetBranch }))
        } else if (outcome.has_conflicts) {
          notifyError(t('branch.mergeConflict', { source: branchName }))
        }
      }

      await fetchStatus()
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : `Failed to merge ${branchName}`
      notifyError(t('common.error'), errorMessage)
      await fetchStatus()
      return false
    } finally {
      isOperatingBranches.value = false
    }
  }

  async function rebaseBranch(upstreamBranch: string): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    const currentBranch = currentBranchName.value
    const isConfirmed = await confirmStandard({
      title: t('branch.rebaseConfirmTitle', { current: currentBranch, upstream: upstreamBranch }),
      description: t('branch.rebaseConfirmDescription', { current: currentBranch, upstream: upstreamBranch }),
      confirmLabel: t('branch.rebaseAction'),
      variant: 'warning',
    })
    if (!isConfirmed) {
      return false
    }

    try {
      isOperatingBranches.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const outcome = await invoke<RebaseBranchOutcome>('rebase_branch', {
          upstreamBranch,
          directoryPath: currentRepository.value.path,
        })

        if (outcome.is_successful) {
          notifySuccess(t('branch.rebaseSuccess', { current: currentBranch, upstream: upstreamBranch }))
        } else if (outcome.has_conflicts) {
          notifyError(t('branch.rebaseConflict', { upstream: upstreamBranch }))
        }
      }

      await fetchStatus()
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : `Failed to rebase onto ${upstreamBranch}`
      notifyError(t('common.error'), errorMessage)
      await fetchStatus()
      return false
    } finally {
      isOperatingBranches.value = false
    }
  }

  async function renameBranch(oldName: string, newName: string): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    const trimmedNewName = newName.trim()
    if (!trimmedNewName || trimmedNewName === oldName) {
      return false
    }

    try {
      isOperatingBranches.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('rename_branch', {
          oldName,
          newName: trimmedNewName,
          directoryPath: currentRepository.value.path,
        })
      }

      notifySuccess(t('branch.renameSuccess', { oldName, newName: trimmedNewName }))

      // Update favorites if renamed
      if (favoriteBranches.value.includes(oldName)) {
        toggleFavoriteBranch(oldName)
        toggleFavoriteBranch(trimmedNewName)
      }

      await fetchReferences()
      await fetchStatus()
      await fetchCommitGraph()
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : `Failed to rename branch ${oldName}`
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
    currentBranchName,
    loadFavoriteBranches,
    toggleFavoriteBranch,
    isFavoriteBranch,
    fetchReferences,
    checkoutBranch,
    createBranch,
    deleteBranch,
    mergeBranch,
    rebaseBranch,
    renameBranch,
  }
}
