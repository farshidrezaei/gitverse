import { useRepositoryWorkspace } from '~/composables/useRepositoryWorkspace'
import { useCommitGraph } from '~/composables/useCommitGraph'
import { useStagingController } from '~/composables/useStagingController'
import { useBranchController } from '~/composables/useBranchController'
import { useNotificationService } from '~/composables/useNotificationService'
import { useConfirmationDialog } from '~/composables/useConfirmationDialog'

export type ResetCommitMode = 'soft' | 'mixed' | 'hard'

export function useCommitOperations() {
  const { currentRepository } = useRepositoryWorkspace()
  const { fetchCommitGraph } = useCommitGraph()
  const { fetchStatus } = useStagingController()
  const { fetchReferences } = useBranchController()
  const { notifySuccess, notifyError } = useNotificationService()
  const { confirmStandard } = useConfirmationDialog()
  const { t } = useI18n()

  const isOperatingCommit = useState<boolean>('workspace-is-operating-commit', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function cherryPickCommit(commitHash: string): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isOperatingCommit.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('cherry_pick_commit', {
          commitHash,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('commit.cherryPickSuccess'))
      await fetchCommitGraph()
      await fetchStatus()
      await fetchReferences()
      return true
    } catch (error) {
      console.error('Failed to cherry-pick commit:', error)
      notifyError(t('commit.cherryPickFailed'))
      await fetchStatus()
      return false
    } finally {
      isOperatingCommit.value = false
    }
  }

  async function revertCommit(commitHash: string): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isOperatingCommit.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('revert_commit', {
          commitHash,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('commit.revertSuccess'))
      await fetchCommitGraph()
      await fetchStatus()
      await fetchReferences()
      return true
    } catch (error) {
      console.error('Failed to revert commit:', error)
      notifyError(t('commit.revertFailed'))
      await fetchStatus()
      return false
    } finally {
      isOperatingCommit.value = false
    }
  }

  async function resetToCommit(
    commitHash: string,
    resetMode: ResetCommitMode = 'mixed'
  ): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    if (resetMode === 'hard') {
      const isConfirmed = await confirmStandard({
        title: t('commit.resetHardConfirmTitle'),
        description: t('commit.resetHardConfirmDescription', {
          hash: commitHash.slice(0, 7),
        }),
        confirmLabel: t('commit.resetHardConfirmButton'),
        variant: 'danger',
      })
      if (!isConfirmed) {
        return false
      }
    }

    try {
      isOperatingCommit.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('reset_to_commit', {
          commitHash,
          resetMode,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('commit.resetSuccess', { mode: resetMode }))
      await fetchCommitGraph()
      await fetchStatus()
      await fetchReferences()
      return true
    } catch (error) {
      console.error('Failed to reset to commit:', error)
      notifyError(t('commit.resetFailed'))
      return false
    } finally {
      isOperatingCommit.value = false
    }
  }

  return {
    isOperatingCommit,
    cherryPickCommit,
    revertCommit,
    resetToCommit,
  }
}
