import type { FileDiff, StashDetails, StashRecord } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'
import { useActionGuard } from '~/composables/useActionGuard'

export function useStashController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { fetchStatus } = useStagingController()
  const { notifySuccess, notifyError } = useNotificationService()
  const { t } = useI18n()

  const stashRecords = useState<StashRecord[]>('workspace-stash-records', () => [])
  const isLoadingStashes = useState<boolean>('workspace-is-loading-stashes', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchStashes(): Promise<StashRecord[]> {
    if (!currentRepository.value) {
      stashRecords.value = []
      return []
    }

    try {
      isLoadingStashes.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<StashRecord[]>('get_stash_list', {
          directoryPath: currentRepository.value.path,
        })
        stashRecords.value = records
        return records
      } else {
        // Fallback for browser mock mode
        return stashRecords.value
      }
    } catch (error) {
      console.error('Failed to fetch stash records:', error)
      notifyError(t('stash.fetchFailed'))
      return []
    } finally {
      isLoadingStashes.value = false
    }
  }

  async function saveStash(
    customMessage?: string,
    includeUntracked: boolean = false
  ): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    const { evaluateAction } = useActionGuard()
    const preflight = evaluateAction('stash_save')
    if (!preflight.canExecute) {
      notifyError(t(preflight.reasonKey!))
      return false
    }

    try {
      isLoadingStashes.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('save_stash', {
          message: customMessage || null,
          includeUntracked,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('stash.savedSuccess'))
      await fetchStashes()
      await fetchStatus()
      return true
    } catch (error) {
      console.error('Failed to save stash:', error)
      notifyError(formatApplicationError(error, t('stash.saveFailed')))
      return false
    } finally {
      isLoadingStashes.value = false
    }
  }

  async function applyStash(stashIndex: number): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isLoadingStashes.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('apply_stash', {
          stashIndex,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('stash.appliedSuccess'))
      await fetchStatus()
      return true
    } catch (error) {
      console.error('Failed to apply stash:', error)
      notifyError(t('stash.applyFailed'))
      return false
    } finally {
      isLoadingStashes.value = false
    }
  }

  async function popStash(stashIndex: number): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isLoadingStashes.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('pop_stash', {
          stashIndex,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('stash.poppedSuccess'))
      await fetchStashes()
      await fetchStatus()
      return true
    } catch (error) {
      console.error('Failed to pop stash:', error)
      notifyError(t('stash.popFailed'))
      return false
    } finally {
      isLoadingStashes.value = false
    }
  }

  async function dropStash(stashIndex: number): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      isLoadingStashes.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('drop_stash', {
          stashIndex,
          directoryPath: currentRepository.value.path,
        })
      }
      notifySuccess(t('stash.droppedSuccess'))
      await fetchStashes()
      return true
    } catch (error) {
      console.error('Failed to drop stash:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isLoadingStashes.value = false
    }
  }

  async function fetchStashDetails(stashIndex: number): Promise<StashDetails | null> {
    if (!currentRepository.value) {
      return null
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        return await invoke<StashDetails>('get_stash_details', {
          stashIndex,
          directoryPath: currentRepository.value.path,
        })
      }
      return null
    } catch (error) {
      console.error('Failed to fetch stash details:', error)
      notifyError(formatApplicationError(error))
      return null
    }
  }

  async function fetchStashFileDiff(
    stashIndex: number,
    filePath: string
  ): Promise<FileDiff | null> {
    if (!currentRepository.value) {
      return null
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        return await invoke<FileDiff>('get_stash_file_diff', {
          stashIndex,
          filePath,
          directoryPath: currentRepository.value.path,
        })
      }
      return null
    } catch (error) {
      console.error('Failed to fetch stash file diff:', error)
      notifyError(formatApplicationError(error))
      return null
    }
  }

  return {
    stashRecords,
    isLoadingStashes,
    fetchStashes,
    saveStash,
    applyStash,
    popStash,
    dropStash,
    fetchStashDetails,
    fetchStashFileDiff,
  }
}
