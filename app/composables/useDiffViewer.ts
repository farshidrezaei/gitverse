import type { FileBlame, FileDiff } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useDiffViewer() {
  const { currentRepository } = useRepositoryWorkspace()
  const currentDiff = useState<FileDiff | null>('workspace-current-diff', () => null)
  const isLoadingDiff = useState<boolean>('workspace-is-loading-diff', () => false)
  const diffViewMode = useState<'unified' | 'split'>('workspace-diff-view-mode', () => 'split')

  const { notifyError } = useNotificationService()
  const { t } = useI18n()

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchFileDiff(filePath: string, isStaged: boolean): Promise<FileDiff | null> {
    if (!currentRepository.value) {
      currentDiff.value = null
      return null
    }

    isLoadingDiff.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const diff = await invoke<FileDiff>('get_file_diff', {
          filePath,
          isStaged,
          directoryPath: currentRepository.value.path,
        })
        currentDiff.value = diff
        return diff
      } else {
        const mockDiff: FileDiff = {
          old_path: isStaged ? null : filePath,
          new_path: filePath,
          is_binary: false,
          is_new_file: false,
          is_deleted_file: false,
          total_additions: 3,
          total_deletions: 1,
          hunks: [
            {
              header: '@@ -1,7 +1,9 @@',
              old_start: 1,
              old_lines: 7,
              new_start: 1,
              new_lines: 9,
              lines: [
                {
                  line_type: 'context',
                  old_line_number: 1,
                  new_line_number: 1,
                  content: '// Sample file for preview',
                },
                {
                  line_type: 'deletion',
                  old_line_number: 2,
                  new_line_number: null,
                  content: '-export const isFeatureEnabled = false',
                },
                {
                  line_type: 'addition',
                  old_line_number: null,
                  new_line_number: 2,
                  content: '+export const isFeatureEnabled = true',
                },
                {
                  line_type: 'addition',
                  old_line_number: null,
                  new_line_number: 3,
                  content: '+export const mode = "modern"',
                },
                {
                  line_type: 'addition',
                  old_line_number: null,
                  new_line_number: 4,
                  content: '+export const version = 2',
                },
                {
                  line_type: 'context',
                  old_line_number: 3,
                  new_line_number: 5,
                  content: '',
                },
                {
                  line_type: 'context',
                  old_line_number: 4,
                  new_line_number: 6,
                  content: 'export function execute() {',
                },
              ],
            },
          ],
        }
        currentDiff.value = mockDiff
        return mockDiff
      }
    } catch (error) {
      const errorMessage = formatApplicationError(error)
      notifyError(t('common.error'), errorMessage)
      currentDiff.value = null
      return null
    } finally {
      isLoadingDiff.value = false
    }
  }

  async function fetchFileBlame(filePath: string, revision?: string): Promise<FileBlame | null> {
    if (!currentRepository.value) {
      return null
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        return await invoke<FileBlame>('get_file_blame', {
          filePath,
          revision,
          directoryPath: currentRepository.value.path,
        })
      } else {
        return {
          file_path: filePath,
          lines: [
            {
              commit_hash: 'abc1234567890123456789012345678901234567',
              author_name: 'Developer',
              author_email: 'dev@example.com',
              author_timestamp: Math.floor(Date.now() / 1000) - 86400,
              commit_summary: 'Initial implementation of file',
              original_line_number: 1,
              final_line_number: 1,
              line_content: 'export function hello() {}',
            },
          ],
        }
      }
    } catch (error) {
      const message = formatApplicationError(error, 'Failed to fetch file blame')
      notifyError(t('common.error'), message)
      return null
    }
  }

  function setDiffViewMode(mode: 'unified' | 'split'): void {
    diffViewMode.value = mode
  }

  function clearDiff(): void {
    currentDiff.value = null
  }

  return {
    currentDiff,
    isLoadingDiff,
    diffViewMode,
    fetchFileDiff,
    fetchFileBlame,
    setDiffViewMode,
    clearDiff,
  }
}
