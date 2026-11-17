import type { CommitDetails, FileDiff } from '~/types/repository'

export function useCommitInspector() {
  const { currentRepository } = useRepositoryWorkspace()
  const { currentDiff, isLoadingDiff } = useDiffViewer()
  const selectedCommitDetails = useState<CommitDetails | null>(
    'workspace-selected-commit-details',
    () => null
  )
  const selectedCommitFilePath = useState<string | null>(
    'workspace-selected-commit-file-path',
    () => null
  )
  const isLoadingCommitDetails = useState<boolean>(
    'workspace-is-loading-commit-details',
    () => false
  )

  const { notifyError } = useNotificationService()
  const { t } = useI18n()

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchCommitDetails(commitHash: string): Promise<CommitDetails | null> {
    if (!currentRepository.value) {
      selectedCommitDetails.value = null
      return null
    }

    isLoadingCommitDetails.value = true

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const details = await invoke<CommitDetails>('get_commit_details', {
          commitHash,
          directoryPath: currentRepository.value.path,
        })
        selectedCommitDetails.value = details
        if (details.changed_files.length > 0 && details.changed_files[0]) {
          await fetchCommitFileDiff(commitHash, details.changed_files[0].path)
        } else {
          await fetchCommitFileDiff(commitHash)
        }
        return details
      } else {
        const mockDetails: CommitDetails = {
          hash: commitHash,
          parent_hashes: ['parent123456'],
          author_name: 'Farshid Rezaei',
          author_email: 'farshid.net1@gmail.com',
          author_timestamp: Math.floor(Date.now() / 1000) - 3600,
          committer_name: 'Farshid Rezaei',
          committer_email: 'farshid.net1@gmail.com',
          committer_timestamp: Math.floor(Date.now() / 1000) - 3600,
          subject: 'Commit details preview subject',
          body: 'Detailed commit body explaining the changes made in this commit.',
          decorations: [],
          total_additions: 15,
          total_deletions: 3,
          changed_files: [
            {
              path: 'src/main.rs',
              original_path: null,
              status: 'modified',
              additions: 12,
              deletions: 2,
            },
            {
              path: 'src/domain/commit.rs',
              original_path: null,
              status: 'added',
              additions: 3,
              deletions: 1,
            },
          ],
        }
        selectedCommitDetails.value = mockDetails
        await fetchCommitFileDiff(commitHash, mockDetails.changed_files[0]?.path)
        return mockDetails
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      selectedCommitDetails.value = null
      return null
    } finally {
      isLoadingCommitDetails.value = false
    }
  }

  async function fetchCommitFileDiff(
    commitHash: string,
    filePath?: string
  ): Promise<FileDiff | null> {
    if (!currentRepository.value) {
      currentDiff.value = null
      return null
    }

    isLoadingDiff.value = true
    selectedCommitFilePath.value = filePath || null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const diff = await invoke<FileDiff>('get_commit_file_diff', {
          commitHash,
          filePath: filePath || null,
          directoryPath: currentRepository.value.path,
        })
        currentDiff.value = diff
        return diff
      } else {
        const mockDiff: FileDiff = {
          old_path: filePath ? `a/${filePath}` : null,
          new_path: filePath || commitHash,
          is_binary: false,
          is_new_file: false,
          is_deleted_file: false,
          total_additions: 5,
          total_deletions: 1,
          hunks: [
            {
              header: '@@ -1,5 +1,9 @@',
              old_start: 1,
              old_lines: 5,
              new_start: 1,
              new_lines: 9,
              lines: [
                {
                  line_type: 'context',
                  old_line_number: 1,
                  new_line_number: 1,
                  content: '// Commit diff preview',
                },
                {
                  line_type: 'addition',
                  old_line_number: null,
                  new_line_number: 2,
                  content: '+// Added in commit',
                },
                {
                  line_type: 'context',
                  old_line_number: 2,
                  new_line_number: 3,
                  content: '',
                },
              ],
            },
          ],
        }
        currentDiff.value = mockDiff
        return mockDiff
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      currentDiff.value = null
      return null
    } finally {
      isLoadingDiff.value = false
    }
  }

  function clearCommitSelection(): void {
    selectedCommitDetails.value = null
    selectedCommitFilePath.value = null
  }

  return {
    selectedCommitDetails,
    selectedCommitFilePath,
    isLoadingCommitDetails,
    fetchCommitDetails,
    fetchCommitFileDiff,
    clearCommitSelection,
  }
}
