import type { GitLfsOverview } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useLfsController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifySuccess, notifyError } = useNotificationService()
  const { t } = useI18n()

  const lfsOverview = useState<GitLfsOverview | null>('workspace-lfs-overview', () => null)
  const isLoadingLfs = useState<boolean>('workspace-is-loading-lfs', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchLfsOverview(): Promise<GitLfsOverview | null> {
    if (!currentRepository.value) return null

    try {
      isLoadingLfs.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<GitLfsOverview>('get_lfs_overview', {
          directoryPath: currentRepository.value.path,
        })
        lfsOverview.value = result
        return result
      }

      // Mock
      const mockResult: GitLfsOverview = {
        is_lfs_installed: false,
        tracked_patterns: ['*.psd', '*.zip'],
        tracked_files: [],
        locks: [],
        total_lfs_size_in_bytes: 0,
      }
      lfsOverview.value = mockResult
      return mockResult
    } catch (error) {
      console.error('Failed to load Git LFS overview:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isLoadingLfs.value = false
    }
  }

  async function trackPattern(pattern: string): Promise<boolean> {
    if (!currentRepository.value || !pattern.trim()) return false

    try {
      isLoadingLfs.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke<string>('lfs_track_pattern', {
          directoryPath: currentRepository.value.path,
          pattern: pattern.trim(),
        })
      }
      notifySuccess(t('lfs.patternTrackedSuccess'))
      await fetchLfsOverview()
      return true
    } catch (error) {
      console.error('Failed to track LFS pattern:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isLoadingLfs.value = false
    }
  }

  async function untrackPattern(pattern: string): Promise<boolean> {
    if (!currentRepository.value || !pattern.trim()) return false

    try {
      isLoadingLfs.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke<string>('lfs_untrack_pattern', {
          directoryPath: currentRepository.value.path,
          pattern: pattern.trim(),
        })
      }
      notifySuccess(t('lfs.patternUntrackedSuccess'))
      await fetchLfsOverview()
      return true
    } catch (error) {
      console.error('Failed to untrack LFS pattern:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isLoadingLfs.value = false
    }
  }

  return {
    lfsOverview,
    isLoadingLfs,
    fetchLfsOverview,
    trackPattern,
    untrackPattern,
  }
}
