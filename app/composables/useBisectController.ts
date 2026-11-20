import type { BisectState } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useBisectController() {
  const { currentRepository, openRepository } = useRepositoryWorkspace()
  const { notifyError, notifySuccess } = useNotificationService()
  const { t } = useI18n()

  const bisectState = useState<BisectState | null>('workspace-bisect-state', () => null)
  const isLoadingBisect = useState<boolean>('workspace-is-loading-bisect', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchBisectStatus(): Promise<BisectState | null> {
    if (!currentRepository.value) {
      bisectState.value = null
      return null
    }

    try {
      isLoadingBisect.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const state = await invoke<BisectState>('get_bisect_status', {
          directoryPath: currentRepository.value.path,
        })
        bisectState.value = state
        return state
      }
      return bisectState.value
    } catch (error) {
      console.error('Failed to query Git bisect status:', error)
      return null
    } finally {
      isLoadingBisect.value = false
    }
  }

  async function startBisect(badCommitHash: string, goodCommitHash: string): Promise<BisectState | null> {
    if (!currentRepository.value) {
      return null
    }

    try {
      isLoadingBisect.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const state = await invoke<BisectState>('bisect_start', {
          badCommitHash,
          goodCommitHash,
          directoryPath: currentRepository.value.path,
        })
        bisectState.value = state
        await openRepository(currentRepository.value.path)
        notifySuccess(t('bisect.notifications.started'))
        return state
      }
      return null
    } catch (error) {
      console.error('Failed to start Git bisect:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isLoadingBisect.value = false
    }
  }

  async function stepBisect(decision: 'good' | 'bad' | 'skip'): Promise<BisectState | null> {
    if (!currentRepository.value) {
      return null
    }

    try {
      isLoadingBisect.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const state = await invoke<BisectState>('bisect_step', {
          decision,
          directoryPath: currentRepository.value.path,
        })
        bisectState.value = state
        await openRepository(currentRepository.value.path)
        if (state.culprit_commit_hash) {
          notifySuccess(t('bisect.notifications.culpritFound'))
        }
        return state
      }
      return null
    } catch (error) {
      console.error('Failed to advance Git bisect:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isLoadingBisect.value = false
    }
  }

  async function resetBisect(): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isLoadingBisect.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('bisect_reset', {
          directoryPath: currentRepository.value.path,
        })
        bisectState.value = null
        await openRepository(currentRepository.value.path)
        notifySuccess(t('bisect.notifications.reset'))
        return true
      }
      return false
    } catch (error) {
      console.error('Failed to reset Git bisect:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isLoadingBisect.value = false
    }
  }

  return {
    bisectState,
    isLoadingBisect,
    fetchBisectStatus,
    startBisect,
    stepBisect,
    resetBisect,
  }
}
