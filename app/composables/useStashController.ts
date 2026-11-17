import type { StashRecord } from '~/types/repository'

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
      notifyError(t('stash.saveFailed'))
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
      notifyError(t('stash.dropFailed'))
      return false
    } finally {
      isLoadingStashes.value = false
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
  }
}
