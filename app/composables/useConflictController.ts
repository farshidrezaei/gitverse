import type { ConflictFileDetails } from '~/types/repository'
import { useRepositoryWorkspace } from '~/composables/useRepositoryWorkspace'
import { useStagingController } from '~/composables/useStagingController'
import { useCommitGraph } from '~/composables/useCommitGraph'
import { useBranchController } from '~/composables/useBranchController'
import { useNotificationService } from '~/composables/useNotificationService'
import { useConfirmationDialog } from '~/composables/useConfirmationDialog'

export type ConflictResolutionStrategy = 'ours' | 'theirs' | 'mark_resolved'

export function useConflictController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { fetchStatus } = useStagingController()
  const { fetchCommitGraph } = useCommitGraph()
  const { fetchReferences } = useBranchController()
  const { notifySuccess, notifyError } = useNotificationService()
  const { confirmStandard } = useConfirmationDialog()
  const { t } = useI18n()

  const activeConflictFile = useState<ConflictFileDetails | null>(
    'workspace-active-conflict-file',
    () => null
  )
  const isLoadingConflict = useState<boolean>('workspace-is-loading-conflict', () => false)
  const isResolvingConflict = useState<boolean>('workspace-is-resolving-conflict', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function loadConflictDetails(filePath: string): Promise<ConflictFileDetails | null> {
    if (!currentRepository.value) {
      return null
    }

    try {
      isLoadingConflict.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const details = await invoke<ConflictFileDetails>('get_conflict_file_details', {
          filePath,
          directoryPath: currentRepository.value.path,
        })
        activeConflictFile.value = details
        return details
      }
      return null
    } catch (error) {
      console.error('Failed to load conflict file details:', error)
      notifyError(t('conflict.loadFailed'))
      return null
    } finally {
      isLoadingConflict.value = false
    }
  }

  async function resolveConflict(
    filePath: string,
    strategy: ConflictResolutionStrategy
  ): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isResolvingConflict.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('resolve_conflict_file', {
          filePath,
          resolutionStrategy: strategy,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('conflict.resolvedSuccess'))
      activeConflictFile.value = null
      await fetchStatus()
      return true
    } catch (error) {
      console.error('Failed to resolve conflict:', error)
      notifyError(t('conflict.resolveFailed'))
      return false
    } finally {
      isResolvingConflict.value = false
    }
  }

  async function writeConflictResolvedContent(
    filePath: string,
    resolvedContent: string
  ): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isResolvingConflict.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('write_conflict_resolved_content', {
          filePath,
          resolvedContent,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('conflict.resolvedSuccess'))
      activeConflictFile.value = null
      await fetchStatus()
      return true
    } catch (error) {
      console.error('Failed to write resolved content:', error)
      notifyError(t('conflict.resolveFailed'))
      return false
    } finally {
      isResolvingConflict.value = false
    }
  }

  async function abortOperation(): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    const isConfirmed = await confirmStandard({
      title: t('conflict.abortConfirmTitle'),
      description: t('conflict.abortConfirmDescription'),
      confirmLabel: t('conflict.abortConfirmButton'),
      variant: 'danger',
    })
    if (!isConfirmed) {
      return false
    }

    try {
      isResolvingConflict.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('abort_repository_operation', {
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('conflict.abortedSuccess'))
      activeConflictFile.value = null
      await fetchStatus()
      await fetchCommitGraph()
      await fetchReferences()
      return true
    } catch (error) {
      console.error('Failed to abort operation:', error)
      notifyError(t('conflict.abortFailed'))
      return false
    } finally {
      isResolvingConflict.value = false
    }
  }

  async function continueOperation(): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isResolvingConflict.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('continue_repository_operation', {
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('conflict.continuedSuccess'))
      activeConflictFile.value = null
      await fetchStatus()
      await fetchCommitGraph()
      await fetchReferences()
      return true
    } catch (error) {
      console.error('Failed to continue operation:', error)
      notifyError(t('conflict.continueFailed'))
      await fetchStatus()
      return false
    } finally {
      isResolvingConflict.value = false
    }
  }

  return {
    activeConflictFile,
    isLoadingConflict,
    isResolvingConflict,
    loadConflictDetails,
    resolveConflict,
    writeConflictResolvedContent,
    abortOperation,
    continueOperation,
  }
}
