import type { FileHistoryTimeline } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useFileHistoryController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifyError } = useNotificationService()

  const timeline = useState<FileHistoryTimeline | null>('workspace-file-history-timeline', () => null)
  const isLoadingTimeline = useState<boolean>('workspace-is-loading-file-history', () => false)
  const revisionContentCache = useState<Record<string, string>>('workspace-revision-content-cache', () => ({}))

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchFileHistory(filePath: string, limit: number = 100): Promise<FileHistoryTimeline | null> {
    if (!currentRepository.value) {
      timeline.value = null
      return null
    }

    try {
      isLoadingTimeline.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<FileHistoryTimeline>('get_file_history', {
          filePath,
          limit,
          directoryPath: currentRepository.value.path,
        })
        timeline.value = result
        return result
      }
      const mockResult: FileHistoryTimeline = {
        file_path: filePath,
        revisions: [
          {
            commit_hash: 'a2d87c0',
            author_name: 'Farshid Rezaei',
            author_email: 'farshid.net1@gmail.com',
            author_timestamp: Date.now() / 1000,
            commit_summary: 'feat(insights): implement repository analytics',
            change_status: 'M',
          },
          {
            commit_hash: '4ba8dfc',
            author_name: 'Farshid Rezaei',
            author_email: 'farshid.net1@gmail.com',
            author_timestamp: Date.now() / 1000 - 3600,
            commit_summary: 'feat(commit): implement conventional commit builder',
            change_status: 'A',
          },
        ],
        total_revisions: 2,
      }
      timeline.value = mockResult
      return mockResult
    } catch (error) {
      console.error('Failed to fetch file history:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isLoadingTimeline.value = false
    }
  }

  async function fetchFileContentAtCommit(commitHash: string, filePath: string): Promise<string> {
    const cacheKey = `${commitHash}:${filePath}`
    if (revisionContentCache.value[cacheKey]) {
      return revisionContentCache.value[cacheKey]
    }

    if (!currentRepository.value) return ''

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const content = await invoke<string>('get_file_content_at_commit', {
          commitHash,
          filePath,
          directoryPath: currentRepository.value.path,
        })
        revisionContentCache.value[cacheKey] = content
        return content
      }
      return `// Mock content of ${filePath} at ${commitHash}\nconsole.log("Historical file content");\n`
    } catch (error) {
      console.error('Failed to fetch file content at commit:', error)
      return ''
    }
  }

  return {
    timeline,
    isLoadingTimeline,
    fetchFileHistory,
    fetchFileContentAtCommit,
  }
}
