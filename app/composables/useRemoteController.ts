import { useRepositoryWorkspace } from '~/composables/useRepositoryWorkspace'
import { useNotificationService } from '~/composables/useNotificationService'

export function useRemoteController() {
  const { currentRepository } = useRepositoryWorkspace()
  const isOperatingRemote = useState<boolean>('workspace-is-operating-remote', () => false)

  const { notifySuccess, notifyError } = useNotificationService()
  const { t } = useI18n()

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchRemote(remoteName?: string): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperatingRemote.value = true

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('fetch_remote', {
          remoteName: remoteName || null,
          directoryPath: currentRepository.value.path,
        })
      }
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Fetch failed'
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingRemote.value = false
    }
  }

  async function pullChanges(remoteName?: string, branchName?: string): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperatingRemote.value = true

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('pull_changes', {
          remoteName: remoteName || null,
          branchName: branchName || null,
          directoryPath: currentRepository.value.path,
        })
      }
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Pull failed'
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingRemote.value = false
    }
  }

  async function pushChanges(
    remoteName?: string,
    branchName?: string,
    setUpstream = false,
  ): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperatingRemote.value = true

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('push_changes', {
          remoteName: remoteName || null,
          branchName: branchName || null,
          setUpstream,
          directoryPath: currentRepository.value.path,
        })
      }
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Push failed'
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingRemote.value = false
    }
  }

  async function addRemote(
    remoteName: string,
    remoteUrl: string,
    shouldFetchImmediately = false,
  ): Promise<boolean> {
    const trimmedName = remoteName.trim()
    const trimmedUrl = remoteUrl.trim()

    if (!trimmedName) {
      notifyError(t('common.error'), t('remote.validationEmptyName'))
      return false
    }

    if (!trimmedUrl) {
      notifyError(t('common.error'), t('remote.validationEmptyUrl'))
      return false
    }

    if (/\s/.test(trimmedName)) {
      notifyError(t('common.error'), t('remote.validationInvalidName'))
      return false
    }

    isOperatingRemote.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('add_remote', {
          name: trimmedName,
          url: trimmedUrl,
          directoryPath: currentRepository.value?.path || null,
        })
      }

      notifySuccess(t('remote.addSuccess', { name: trimmedName }))

      if (shouldFetchImmediately) {
        await fetchRemote(trimmedName)
      }

      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingRemote.value = false
    }
  }

  async function removeRemote(remoteName: string): Promise<boolean> {
    const trimmedName = remoteName.trim()
    if (!trimmedName) return false

    isOperatingRemote.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('remove_remote', {
          name: trimmedName,
          directoryPath: currentRepository.value?.path || null,
        })
      }

      notifySuccess(t('remote.removeSuccess', { name: trimmedName }))
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingRemote.value = false
    }
  }

  async function renameRemote(oldName: string, newName: string): Promise<boolean> {
    const trimmedOld = oldName.trim()
    const trimmedNew = newName.trim()

    if (!trimmedNew) {
      notifyError(t('common.error'), t('remote.validationEmptyName'))
      return false
    }

    if (/\s/.test(trimmedNew)) {
      notifyError(t('common.error'), t('remote.validationInvalidName'))
      return false
    }

    isOperatingRemote.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('rename_remote', {
          oldName: trimmedOld,
          newName: trimmedNew,
          directoryPath: currentRepository.value?.path || null,
        })
      }

      notifySuccess(t('remote.renameSuccess', { name: trimmedNew }))
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingRemote.value = false
    }
  }

  async function setRemoteUrl(
    remoteName: string,
    remoteUrl: string,
    isPushOnly = false,
  ): Promise<boolean> {
    const trimmedName = remoteName.trim()
    const trimmedUrl = remoteUrl.trim()

    if (!trimmedUrl) {
      notifyError(t('common.error'), t('remote.validationEmptyUrl'))
      return false
    }

    isOperatingRemote.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('set_remote_url', {
          name: trimmedName,
          url: trimmedUrl,
          isPushOnly,
          directoryPath: currentRepository.value?.path || null,
        })
      }

      notifySuccess(t('remote.editUrlSuccess'))
      return true
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      return false
    } finally {
      isOperatingRemote.value = false
    }
  }

  async function copyRemoteUrl(remoteUrl: string): Promise<void> {
    if (!navigator?.clipboard) return
    try {
      await navigator.clipboard.writeText(remoteUrl)
      notifySuccess(t('remote.copySuccess'))
    } catch (error) {
      console.error('Failed to copy remote URL:', error)
    }
  }

  return {
    isOperatingRemote,
    fetchRemote,
    pullChanges,
    pushChanges,
    addRemote,
    removeRemote,
    renameRemote,
    setRemoteUrl,
    copyRemoteUrl,
  }
}
