import type { RepositoryInsights } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useInsightsController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifyError } = useNotificationService()

  const insights = useState<RepositoryInsights | null>('workspace-repository-insights', () => null)
  const isLoadingInsights = useState<boolean>('workspace-is-loading-insights', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchInsights(): Promise<RepositoryInsights | null> {
    if (!currentRepository.value) {
      insights.value = null
      return null
    }

    try {
      isLoadingInsights.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<RepositoryInsights>('get_repository_insights', {
          directoryPath: currentRepository.value.path,
        })
        insights.value = result
        return result
      }
      // Browser fallback mock
      const mockResult: RepositoryInsights = {
        total_commits: 120,
        total_contributors: 3,
        top_contributors: [
          { name: 'Farshid Rezaei', email: 'farshid.net1@gmail.com', commit_count: 100, percentage: 83.3 },
          { name: 'Alice Developer', email: 'alice@example.com', commit_count: 15, percentage: 12.5 },
          { name: 'Bob Engineer', email: 'bob@example.com', commit_count: 5, percentage: 4.2 },
        ],
        activity_by_weekday: [
          { day_of_week: 0, count: 12 },
          { day_of_week: 1, count: 25 },
          { day_of_week: 2, count: 30 },
          { day_of_week: 3, count: 18 },
          { day_of_week: 4, count: 22 },
          { day_of_week: 5, count: 10 },
          { day_of_week: 6, count: 3 },
        ],
        lines_added: 62988,
        lines_deleted: 1205,
      }
      insights.value = mockResult
      return mockResult
    } catch (error) {
      console.error('Failed to fetch repository insights:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isLoadingInsights.value = false
    }
  }

  return {
    insights,
    isLoadingInsights,
    fetchInsights,
  }
}
