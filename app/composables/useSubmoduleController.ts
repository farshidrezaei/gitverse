import type { GitSubmoduleSummary } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useSubmoduleController() {
  const { currentRepository, openRepository } = useRepositoryWorkspace()
  const { notifyError, notifySuccess } = useNotificationService()
  const { t } = useI18n()

  const submodules = useState<GitSubmoduleSummary[]>('workspace-submodules', () => [])
  const isLoadingSubmodules = useState<boolean>('workspace-is-loading-submodules', () => false)
  const isOperatingSubmodules = useState<boolean>('workspace-is-operating-submodules', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchSubmodules(): Promise<GitSubmoduleSummary[]> {
    if (!currentRepository.value) {
      submodules.value = []
      return []
    }

    try {
      isLoadingSubmodules.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<GitSubmoduleSummary[]>('get_submodules', {
          directoryPath: currentRepository.value.path,
        })
        submodules.value = records
        return records
      }
      return submodules.value
    } catch (error) {
      console.error('Failed to fetch Git submodules:', error)
      notifyError(formatApplicationError(error))
      return []
    } finally {
      isLoadingSubmodules.value = false
    }
  }

  async function updateSubmodules(
    submodulePath?: string,
    isInit: boolean = true,
    isRecursive: boolean = true
  ): Promise<boolean> {
    if (!currentRepository.value) return false

    try {
      isOperatingSubmodules.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('update_submodules', {
          submodulePath: submodulePath || null,
          isInit,
          isRecursive,
          directoryPath: currentRepository.value.path,
        })
        await fetchSubmodules()
        notifySuccess(t('submodule.notifications.updated'))
        return true
      }
      return false
    } catch (error) {
      console.error('Failed to update submodules:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isOperatingSubmodules.value = false
    }
  }

  async function syncSubmodules(
    submodulePath?: string,
    isRecursive: boolean = true
  ): Promise<boolean> {
    if (!currentRepository.value) return false

    try {
      isOperatingSubmodules.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('sync_submodules', {
          submodulePath: submodulePath || null,
          isRecursive,
          directoryPath: currentRepository.value.path,
        })
        await fetchSubmodules()
        notifySuccess(t('submodule.notifications.synced'))
        return true
      }
      return false
    } catch (error) {
      console.error('Failed to sync submodules:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isOperatingSubmodules.value = false
    }
  }

  async function openSubmoduleAsRepository(submodule: GitSubmoduleSummary): Promise<void> {
    if (!currentRepository.value) return
    const targetPath = `${currentRepository.value.path}/${submodule.path}`
    await openRepository(targetPath)
  }

  return {
    submodules,
    isLoadingSubmodules,
    isOperatingSubmodules,
    fetchSubmodules,
    updateSubmodules,
    syncSubmodules,
    openSubmoduleAsRepository,
  }
}
