import type { PatchCheckResult, PatchExportScope } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function usePatchController() {
  const { currentRepository, openRepository } = useRepositoryWorkspace()
  const { notifyError, notifySuccess } = useNotificationService()
  const { t } = useI18n()

  const isOperating = useState<boolean>('workspace-patch-operating', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function exportPatch(
    scope: PatchExportScope,
    commitHash?: string
  ): Promise<string | null> {
    if (!currentRepository.value) return null

    try {
      isOperating.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const patch = await invoke<string>('export_patch', {
          scope,
          commitHash: commitHash || null,
          directoryPath: currentRepository.value.path,
        })
        return patch
      }
      return '# Sample unified diff patch\ndiff --git a/file.txt b/file.txt\n'
    } catch (error) {
      console.error('Failed to export patch:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isOperating.value = false
    }
  }

  async function checkPatch(patchContent: string): Promise<PatchCheckResult | null> {
    if (!currentRepository.value) return null

    try {
      isOperating.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<PatchCheckResult>('check_patch', {
          patchContent,
          directoryPath: currentRepository.value.path,
        })
        return result
      }
      return {
        can_apply: true,
        error_message: null,
        affected_files: ['file.txt'],
      }
    } catch (error) {
      console.error('Failed to validate patch:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isOperating.value = false
    }
  }

  async function applyPatch(
    patchContent: string,
    isCached: boolean = false,
    isReverse: boolean = false
  ): Promise<boolean> {
    if (!currentRepository.value) return false

    try {
      isOperating.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('apply_patch_file', {
          patchContent,
          isCached,
          isReverse,
          directoryPath: currentRepository.value.path,
        })
        await openRepository(currentRepository.value.path)
        notifySuccess(t('patch.notifications.applied'))
        return true
      }
      return false
    } catch (error) {
      console.error('Failed to apply patch:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isOperating.value = false
    }
  }

  function downloadPatchFile(patchContent: string, filename: string = 'changes.patch') {
    const blob = new Blob([patchContent], { type: 'text/x-diff;charset=utf-8' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(link.href)
    notifySuccess(t('patch.notifications.downloaded'))
  }

  return {
    isOperating,
    exportPatch,
    checkPatch,
    applyPatch,
    downloadPatchFile,
  }
}
