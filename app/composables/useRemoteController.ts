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
    setUpstream = false
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

  return {
    isOperatingRemote,
    fetchRemote,
    pullChanges,
    pushChanges,
  }
}
