import type {
  InteractiveRebaseRequest,
  RebaseBranchOutcome,
  RebasePlanItem,
  RepositoryOverview,
} from '~/types/repository'

function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useRebaseController() {
  const currentRepository = tryUseNuxtApp()
    ? useRepositoryWorkspace().currentRepository
    : ref<RepositoryOverview | null>(null)

  const isRebasing = safeState<boolean>('workspace-is-interactive-rebasing', () => false)

  // Capture composables at the top level of setup
  const notificationService = tryUseNuxtApp() ? useNotificationService() : null
  const i18n = tryUseNuxtApp() ? useI18n() : null

  function notifySuccess(message: string): void {
    notificationService?.notifySuccess(message)
  }

  function notifyError(message: string): void {
    notificationService?.notifyError(message)
  }

  function translate(key: string, parameters?: Record<string, unknown>): string {
    if (i18n) {
      return i18n.t(key, parameters || {})
    }
    return key
  }

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchRebaseCommits(baseCommitHash: string): Promise<RebasePlanItem[]> {
    if (!currentRepository.value) {
      return []
    }

    try {
      isRebasing.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const items = await invoke<RebasePlanItem[]>('get_rebase_commits', {
          directoryPath: currentRepository.value.path,
          baseCommitHash,
        })
        return items
      } else {
        // Browser mock mode
        return [
          {
            action: 'pick',
            commit_hash: '1111111111111111111111111111111111111111',
            short_hash: '1111111',
            original_subject: 'feat: add first feature',
            new_message: null,
            author_name: 'Farshid Rezaei',
          },
          {
            action: 'pick',
            commit_hash: '2222222222222222222222222222222222222222',
            short_hash: '2222222',
            original_subject: 'fix: typo in documentation',
            new_message: null,
            author_name: 'Farshid Rezaei',
          },
        ]
      }
    } catch (error) {
      console.error('Failed to load commits for interactive rebase:', error)
      notifyError(translate('rebase.fetchCommitsFailed'))
      return []
    } finally {
      isRebasing.value = false
    }
  }

  async function executeRebase(request: InteractiveRebaseRequest): Promise<RebaseBranchOutcome | null> {
    if (!currentRepository.value) {
      return null
    }

    try {
      isRebasing.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const outcome = await invoke<RebaseBranchOutcome>('execute_interactive_rebase', {
          directoryPath: currentRepository.value.path,
          request,
        })

        if (outcome.is_successful) {
          notifySuccess(translate('rebase.rebaseSuccessful'))
        } else if (outcome.has_conflicts) {
          notifyError(translate('rebase.rebaseConflicts'))
        }
        return outcome
      } else {
        // Browser mock mode
        notifySuccess(translate('rebase.rebaseSuccessful'))
        return {
          is_successful: true,
          has_conflicts: false,
          message: 'Interactive rebase executed successfully.',
        }
      }
    } catch (error) {
      console.error('Failed to execute interactive rebase:', error)
      notifyError(translate('rebase.rebaseFailed', { error: String(error) }))
      return null
    } finally {
      isRebasing.value = false
    }
  }

  return {
    isRebasing,
    fetchRebaseCommits,
    executeRebase,
  }
}
