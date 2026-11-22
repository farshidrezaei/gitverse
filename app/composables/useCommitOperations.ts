import { useRepositoryWorkspace } from '~/composables/useRepositoryWorkspace'
import { useCommitGraph } from '~/composables/useCommitGraph'
import { useStagingController } from '~/composables/useStagingController'
import { useBranchController } from '~/composables/useBranchController'
import { useNotificationService } from '~/composables/useNotificationService'
import { useConfirmationDialog } from '~/composables/useConfirmationDialog'
import { formatApplicationError } from '~/utils/tauriClient'
import { useActionGuard } from '~/composables/useActionGuard'

export type ResetCommitMode = 'soft' | 'mixed' | 'hard'

export function useCommitOperations() {
  const { currentRepository } = useRepositoryWorkspace()
  const { fetchCommitGraph } = useCommitGraph()
  const { fetchStatus } = useStagingController()
  const { fetchReferences } = useBranchController()
  const { notifySuccess, notifyError, notifyUndoable } = useNotificationService()
  const { confirmStandard, confirmHold } = useConfirmationDialog()
  const { t } = useI18n()

  const isOperatingCommit = useState<boolean>('workspace-is-operating-commit', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function cherryPickCommit(commitHash: string): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    const { evaluateAction } = useActionGuard()
    const preflight = evaluateAction('cherry_pick')
    if (!preflight.canExecute) {
      notifyError(t('commit.cherryPickFailed'), t(preflight.reasonKey!))
      return false
    }

    const isConfirmed = await confirmStandard({
      title: t('commit.cherryPickConfirmTitle'),
      description: t('commit.cherryPickConfirmDescription', {
        hash: commitHash.slice(0, 7),
      }),
      confirmLabel: t('commit.cherryPickConfirmButton'),
      variant: 'primary',
      details: [
        { label: t('git.commit'), value: commitHash.slice(0, 8), isMonospace: true },
      ],
    })
    if (!isConfirmed) {
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
      notifyError(t('commit.cherryPickFailed'), formatApplicationError(error))
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

    const { evaluateAction } = useActionGuard()
    const preflight = evaluateAction('revert')
    if (!preflight.canExecute) {
      notifyError(t('commit.revertFailed'), t(preflight.reasonKey!))
      return false
    }

    const isConfirmed = await confirmStandard({
      title: t('commit.revertConfirmTitle'),
      description: t('commit.revertConfirmDescription', {
        hash: commitHash.slice(0, 7),
      }),
      confirmLabel: t('commit.revertConfirmButton'),
      variant: 'warning',
      details: [
        { label: t('git.commit'), value: commitHash.slice(0, 8), isMonospace: true },
      ],
    })
    if (!isConfirmed) {
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

      await fetchCommitGraph()
      await fetchStatus()
      await fetchReferences()

      // Provide immediate toast with Undo capability (soft/mixed reset of the revert commit)
      notifyUndoable(
        t('commit.revertSuccess'),
        t('notifications.undoHint', { hash: commitHash.slice(0, 7) }),
        t('notifications.undo'),
        async () => {
          try {
            if (await isTauriEnvironment()) {
              const { invoke } = await import('@tauri-apps/api/core')
              await invoke('reset_to_commit', {
                commitHash: 'HEAD~1',
                resetMode: 'mixed',
                directoryPath: currentRepository.value?.path,
              })
            }
            notifySuccess(t('commit.revertUndone'))
            await fetchCommitGraph()
            await fetchStatus()
            await fetchReferences()
          } catch (undoError) {
            console.error('Failed to undo revert commit:', undoError)
            notifyError(t('notifications.undoFailed'), formatApplicationError(undoError))
          }
        }
      )

      return true
    } catch (error) {
      console.error('Failed to revert commit:', error)
      notifyError(t('commit.revertFailed'), formatApplicationError(error))
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
      const isConfirmed = await confirmHold({
        title: t('commit.resetHardConfirmTitle'),
        description: t('commit.resetHardConfirmDescription', {
          hash: commitHash.slice(0, 7),
        }),
        confirmLabel: t('commit.resetHardConfirmButton'),
        variant: 'danger',
        holdDurationMilliseconds: 2500,
        details: [
          { label: t('git.commit'), value: commitHash.slice(0, 8), isMonospace: true },
          { label: t('commit.resetModeLabel'), value: t('commit.resetHard') },
        ],
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
