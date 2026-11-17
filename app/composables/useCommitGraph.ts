import type { CommitGraphHistoryResponse } from '~/types/repository'

export function useCommitGraph() {
  const { currentRepository } = useRepositoryWorkspace()
  const commitGraph = useState<CommitGraphHistoryResponse | null>(
    'workspace-commit-graph',
    () => null
  )
  const selectedCommitHash = useState<string | null>(
    'workspace-selected-commit-hash',
    () => null
  )
  const isLoadingGraph = useState<boolean>('workspace-is-loading-graph', () => false)

  const { notifyError } = useNotificationService()
  const { t } = useI18n()

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchCommitGraph(
    skip = 0,
    limit = 100,
    branchOrReference?: string
  ): Promise<CommitGraphHistoryResponse | null> {
    if (!currentRepository.value) {
      commitGraph.value = null
      return null
    }

    isLoadingGraph.value = true
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const response = await invoke<CommitGraphHistoryResponse>('get_commit_graph', {
          directoryPath: currentRepository.value.path,
          skip,
          limit,
          branchOrReference: branchOrReference || null,
        })
        commitGraph.value = response
        if (!selectedCommitHash.value && response.nodes.length > 0) {
          selectedCommitHash.value = response.nodes[0]?.commit.hash || null
        }
        return response
      } else {
        const mockResponse: CommitGraphHistoryResponse = {
          nodes: [
            {
              commit: {
                hash: '5897482c17022d926b55574e2dc4c1b92d07b5d4',
                parent_hashes: ['a991385ed2ff207cba10ab91977e8a6230644992'],
                author_name: 'Farshid',
                author_email: 'farshid.net1@gmail.com',
                author_timestamp: Math.floor(Date.now() / 1000) - 300,
                committer_name: 'Farshid',
                committer_email: 'farshid.net1@gmail.com',
                committer_timestamp: Math.floor(Date.now() / 1000) - 300,
                subject: 'feat(diff): implement unified and side-by-side diff viewer with hunk pairing',
                body: '',
                decorations: [
                  { name: 'HEAD', reference_type: 'head', is_head: true },
                  { name: 'master', reference_type: 'localbranch', is_head: true },
                ],
              },
              lane: 0,
              edges: [
                {
                  parent_hash: 'a991385ed2ff207cba10ab91977e8a6230644992',
                  from_lane: 0,
                  to_lane: 0,
                  is_merge_edge: false,
                },
              ],
              is_head: true,
              is_merge: false,
            },
            {
              commit: {
                hash: 'a991385ed2ff207cba10ab91977e8a6230644992',
                parent_hashes: ['85ec25a99fc173af066eb513685e13e306b72dbc'],
                author_name: 'Farshid',
                author_email: 'farshid.net1@gmail.com',
                author_timestamp: Math.floor(Date.now() / 1000) - 3600,
                committer_name: 'Farshid',
                committer_email: 'farshid.net1@gmail.com',
                committer_timestamp: Math.floor(Date.now() / 1000) - 3600,
                subject: 'fix(ui): use custom segmented pill for tree/flat staging view toggle',
                body: '',
                decorations: [],
              },
              lane: 0,
              edges: [
                {
                  parent_hash: '85ec25a99fc173af066eb513685e13e306b72dbc',
                  from_lane: 0,
                  to_lane: 0,
                  is_merge_edge: false,
                },
              ],
              is_head: false,
              is_merge: false,
            },
          ],
          max_lane_index: 0,
          total_commits_count: 2,
          has_more_commits: false,
        }
        commitGraph.value = mockResponse
        selectedCommitHash.value = mockResponse.nodes[0]?.commit.hash || null
        return mockResponse
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      notifyError(t('common.error'), errorMessage)
      return null
    } finally {
      isLoadingGraph.value = false
    }
  }

  function selectCommit(commitHash: string): void {
    selectedCommitHash.value = commitHash
  }

  function selectNextCommit(): void {
    if (!commitGraph.value || commitGraph.value.nodes.length === 0) return
    const currentIndex = commitGraph.value.nodes.findIndex(
      (node) => node.commit.hash === selectedCommitHash.value
    )
    if (currentIndex >= 0 && currentIndex < commitGraph.value.nodes.length - 1) {
      selectedCommitHash.value = commitGraph.value.nodes[currentIndex + 1]?.commit.hash || null
    }
  }

  function selectPreviousCommit(): void {
    if (!commitGraph.value || commitGraph.value.nodes.length === 0) return
    const currentIndex = commitGraph.value.nodes.findIndex(
      (node) => node.commit.hash === selectedCommitHash.value
    )
    if (currentIndex > 0) {
      selectedCommitHash.value = commitGraph.value.nodes[currentIndex - 1]?.commit.hash || null
    }
  }

  return {
    commitGraph,
    selectedCommitHash,
    isLoadingGraph,
    fetchCommitGraph,
    selectCommit,
    selectNextCommit,
    selectPreviousCommit,
  }
}
