import { formatApplicationError } from '~/utils/tauriClient'

export type SupportedExternalEditor =
  | 'vscode'
  | 'cursor'
  | 'zed'
  | 'sublime'
  | 'intellij'

export function useExternalToolsController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifySuccess, notifyError } = useNotificationService()
  const { t } = useI18n()

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function openInEditor(
    editorName: SupportedExternalEditor = 'vscode',
    filePath?: string
  ): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('open_in_external_editor', {
          directoryPath: currentRepository.value.path,
          filePath: filePath || null,
          editorName,
        })
      }
      return true
    } catch (error) {
      console.error('Failed to open in external editor:', error)
      notifyError(formatApplicationError(error))
      return false
    }
  }

  async function openInTerminal(directoryPath?: string): Promise<boolean> {
    const targetPath = directoryPath || currentRepository.value?.path
    if (!targetPath) {
      return false
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('open_in_terminal', {
          directoryPath: targetPath,
        })
      }
      return true
    } catch (error) {
      console.error('Failed to open terminal:', error)
      notifyError(formatApplicationError(error))
      return false
    }
  }

  async function openInFileManager(filePath?: string): Promise<boolean> {
    if (!currentRepository.value) {
      return false
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('open_in_file_manager', {
          directoryPath: currentRepository.value.path,
          filePath: filePath || null,
        })
      }
      return true
    } catch (error) {
      console.error('Failed to open file manager:', error)
      notifyError(formatApplicationError(error))
      return false
    }
  }

  return {
    openInEditor,
    openInTerminal,
    openInFileManager,
  }
}
