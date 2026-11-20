import type { ReflogRecord } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useReflogController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifyError } = useNotificationService()
  const { t } = useI18n()

  const reflogRecords = useState<ReflogRecord[]>('workspace-reflog-records', () => [])
  const isLoadingReflog = useState<boolean>('workspace-is-loading-reflog', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchReflog(
    reference?: string,
    limit: number = 100
  ): Promise<ReflogRecord[]> {
    if (!currentRepository.value) {
      reflogRecords.value = []
      return []
    }

    try {
      isLoadingReflog.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<ReflogRecord[]>('get_reflog', {
          reference: reference || null,
          limit,
          directoryPath: currentRepository.value.path,
        })
        reflogRecords.value = records
        return records
      } else {
        return reflogRecords.value
      }
    } catch (error) {
      console.error('Failed to fetch reflog records:', error)
      notifyError(formatApplicationError(error))
      return []
    } finally {
      isLoadingReflog.value = false
    }
  }

  return {
    reflogRecords,
    isLoadingReflog,
    fetchReflog,
  }
}
