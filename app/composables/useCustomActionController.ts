import type { CustomActionDefinition, CustomActionExecutionResult } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useCustomActionController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifySuccess, notifyError } = useNotificationService()
  const { t } = useI18n()

  const customActions = useState<CustomActionDefinition[]>('workspace-custom-actions', () => [])
  const isLoadingActions = useState<boolean>('workspace-is-loading-actions', () => false)
  const isExecutingAction = useState<boolean>('workspace-is-executing-action', () => false)
  const lastExecutionResult = useState<CustomActionExecutionResult | null>(
    'workspace-last-action-result',
    () => null,
  )

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchCustomActions(): Promise<CustomActionDefinition[]> {
    try {
      isLoadingActions.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const results = await invoke<CustomActionDefinition[]>('list_custom_actions')
        customActions.value = results
        return results
      }

      // Browser Mock
      const mockList: CustomActionDefinition[] = [
        {
          action_identifier: 'git_status',
          display_name: 'Status Quick Check',
          command_string: 'git status',
          icon_name: 'i-lucide-check-square',
          description_text: 'Display concise git status in terminal',
        },
      ]
      customActions.value = mockList
      return mockList
    } catch (error) {
      console.error('Failed to list custom actions:', error)
      notifyError(formatApplicationError(error))
      return []
    } finally {
      isLoadingActions.value = false
    }
  }

  async function saveAction(action: CustomActionDefinition): Promise<boolean> {
    try {
      isLoadingActions.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('save_custom_action', { action })
      }
      notifySuccess(t('actions.actionSavedSuccess'))
      await fetchCustomActions()
      return true
    } catch (error) {
      console.error('Failed to save custom action:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isLoadingActions.value = false
    }
  }

  async function deleteAction(actionIdentifier: string): Promise<boolean> {
    try {
      isLoadingActions.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('delete_custom_action', { actionIdentifier })
      }
      notifySuccess(t('actions.actionDeletedSuccess'))
      await fetchCustomActions()
      return true
    } catch (error) {
      console.error('Failed to delete custom action:', error)
      notifyError(formatApplicationError(error))
      return false
    } finally {
      isLoadingActions.value = false
    }
  }

  async function executeAction(actionIdentifier: string): Promise<CustomActionExecutionResult | null> {
    if (!currentRepository.value) return null

    try {
      isExecutingAction.value = true
      lastExecutionResult.value = null
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<CustomActionExecutionResult>('execute_custom_action', {
          actionIdentifier,
          directoryPath: currentRepository.value.path,
        })
        lastExecutionResult.value = result
        if (result.is_successful) {
          notifySuccess(t('actions.actionExecutedSuccess'))
        } else {
          notifyError(t('actions.actionFailed'))
        }
        return result
      }

      // Mock
      const mockResult: CustomActionExecutionResult = {
        action_identifier: actionIdentifier,
        is_successful: true,
        exit_code: 0,
        standard_output: 'On branch main\nnothing to commit, working tree clean',
        standard_error: '',
      }
      lastExecutionResult.value = mockResult
      notifySuccess(t('actions.actionExecutedSuccess'))
      return mockResult
    } catch (error) {
      console.error('Failed to execute custom action:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isExecutingAction.value = false
    }
  }

  return {
    customActions,
    isLoadingActions,
    isExecutingAction,
    lastExecutionResult,
    fetchCustomActions,
    saveAction,
    deleteAction,
    executeAction,
  }
}
