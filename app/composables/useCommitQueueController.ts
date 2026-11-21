import type { CommitQueueOperationResult } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useCommitQueueController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifySuccess, notifyError, notifyWarning } = useNotificationService()
  const { t } = useI18n()

  const isExecutingQueue = useState<boolean>('workspace-is-executing-commit-queue', () => false)
  const queueResult = useState<CommitQueueOperationResult | null>('workspace-commit-queue-result', () => null)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function executeCherryPickQueue(
    commitHashes: string[],
    shouldCommit: boolean = true,
  ): Promise<CommitQueueOperationResult | null> {
    if (!currentRepository.value || commitHashes.length === 0) return null

    try {
      isExecutingQueue.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<CommitQueueOperationResult>('execute_cherry_pick_queue', {
          directoryPath: currentRepository.value.path,
          commitHashes,
          shouldCommit,
        })
        queueResult.value = result
        if (result.is_successful) {
          notifySuccess(t('commitQueue.cherryPickSuccess', { count: result.completed_commits.length }))
        } else {
          notifyWarning(t('commitQueue.conflictDetected', { commit: result.failed_commit_hash?.slice(0, 7) }))
        }
        return result
      }

      // Browser Mock
      const mockResult: CommitQueueOperationResult = {
        is_successful: true,
        completed_commits: [...commitHashes],
        failed_commit_hash: null,
        conflict_files: [],
        message: 'Mock cherry-pick applied',
      }
      queueResult.value = mockResult
      notifySuccess(t('commitQueue.cherryPickSuccess', { count: commitHashes.length }))
      return mockResult
    } catch (error) {
      console.error('Failed to execute cherry-pick queue:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isExecutingQueue.value = false
    }
  }

  async function executeRevertQueue(
    commitHashes: string[],
    shouldCommit: boolean = true,
  ): Promise<CommitQueueOperationResult | null> {
    if (!currentRepository.value || commitHashes.length === 0) return null

    try {
      isExecutingQueue.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<CommitQueueOperationResult>('execute_revert_queue', {
          directoryPath: currentRepository.value.path,
          commitHashes,
          shouldCommit,
        })
        queueResult.value = result
        if (result.is_successful) {
          notifySuccess(t('commitQueue.revertSuccess', { count: result.completed_commits.length }))
        } else {
          notifyWarning(t('commitQueue.conflictDetected', { commit: result.failed_commit_hash?.slice(0, 7) }))
        }
        return result
      }

      // Browser Mock
      const mockResult: CommitQueueOperationResult = {
        is_successful: true,
        completed_commits: [...commitHashes],
        failed_commit_hash: null,
        conflict_files: [],
        message: 'Mock revert applied',
      }
      queueResult.value = mockResult
      notifySuccess(t('commitQueue.revertSuccess', { count: commitHashes.length }))
      return mockResult
    } catch (error) {
      console.error('Failed to execute revert queue:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isExecutingQueue.value = false
    }
  }

  return {
    isExecutingQueue,
    queueResult,
    executeCherryPickQueue,
    executeRevertQueue,
  }
}
