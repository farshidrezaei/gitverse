import type {
  BranchSummary,
  CommitChangedFile,
  CommitDetails,
  CommitGraphHistoryResponse,
  DetailedRepositoryStatus,
  FileDiff,
  GitBinaryInformation,
  RecentRepositoryRecord,
  RepositoryOverview,
  RepositoryReferences,
} from '~/types/repository'

export interface SelectedDiffTarget {
  filePath: string
  isStaged: boolean
}

const LAST_OPENED_REPO_STORAGE_KEY = 'gitverse_last_opened_repository_path'

export function useWorkspaceManager() {
  const currentRepository = useState<RepositoryOverview | null>('workspace-current-repository', () => null)
  const repositoryStatus = useState<DetailedRepositoryStatus | null>('workspace-repository-status', () => null)
  const repositoryReferences = useState<RepositoryReferences | null>('workspace-references', () => null)
  const recentRepositories = useState<RecentRepositoryRecord[]>('workspace-recent-repositories', () => [])
  const gitBinary = useState<GitBinaryInformation | null>('workspace-git-binary', () => null)
  const selectedDiffTarget = useState<SelectedDiffTarget | null>('workspace-selected-diff-target', () => null)
  const currentDiff = useState<FileDiff | null>('workspace-current-diff', () => null)
  const commitGraph = useState<CommitGraphHistoryResponse | null>('workspace-commit-graph', () => null)
  const selectedCommitHash = useState<string | null>('workspace-selected-commit-hash', () => null)
  const selectedCommitDetails = useState<CommitDetails | null>('workspace-selected-commit-details', () => null)
  const selectedCommitFilePath = useState<string | null>('workspace-selected-commit-file-path', () => null)
  const favoriteBranches = useState<string[]>('workspace-favorite-branches', () => [])
  const isLoading = ref<boolean>(false)
  const isLoadingDiff = ref<boolean>(false)
  const isLoadingGraph = ref<boolean>(false)
  const isLoadingCommitDetails = ref<boolean>(false)
  const isOperating = ref<boolean>(false)
  const lastError = ref<string | null>(null)

  let unlistenStatusWatcher: (() => void) | null = null

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function fetchStatus(): Promise<DetailedRepositoryStatus | null> {
    if (!currentRepository.value) {
      repositoryStatus.value = null
      return null
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const status = await invoke<DetailedRepositoryStatus>('get_repository_status', {
          directoryPath: currentRepository.value.path,
        })
        repositoryStatus.value = status

        // Keep current repository sync counts updated
        if (currentRepository.value) {
          currentRepository.value.ahead_count = status.ahead_count
          currentRepository.value.behind_count = status.behind_count
          currentRepository.value.working_tree_is_clean =
            status.staged_files.length === 0 &&
            status.unstaged_files.length === 0 &&
            status.conflicted_files.length === 0
          if (status.branch_name) {
            currentRepository.value.current_branch_name = status.branch_name
          }
        }
        if (selectedDiffTarget.value) {
          await fetchFileDiff(selectedDiffTarget.value.filePath, selectedDiffTarget.value.isStaged)
        }
        return status
      } else {
        // Mock status in dev preview
        const mock: DetailedRepositoryStatus = {
          branch_name: currentRepository.value.current_branch_name || 'master',
          head_commit_hash: 'dab7678166d',
          upstream_branch_name: 'origin/master',
          ahead_count: 0,
          behind_count: 0,
          is_detached_head: false,
          staged_files: [
            { path: 'src-tauri/Cargo.toml', original_path: null, status: 'modified', is_staged: true },
          ],
          unstaged_files: [
            { path: 'app/pages/index.vue', original_path: null, status: 'modified', is_staged: false },
            { path: 'scratch/notes.txt', original_path: null, status: 'untracked', is_staged: false },
          ],
          conflicted_files: [],
        }
        repositoryStatus.value = mock
        if (selectedDiffTarget.value) {
          await fetchFileDiff(selectedDiffTarget.value.filePath, selectedDiffTarget.value.isStaged)
        }
        return mock
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Failed to fetch status'
      return null
    }
  }

  async function setupRealtimeListener(): Promise<void> {
    if (await isTauriEnvironment()) {
      const { listen } = await import('@tauri-apps/api/event')
      if (unlistenStatusWatcher) {
        unlistenStatusWatcher()
      }
      const unlisten = await listen('repository://status-changed', async () => {
        await fetchStatus()
      })
      unlistenStatusWatcher = unlisten
    }
  }

  async function stageFile(filePath: string): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('stage_file', {
        filePath,
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function unstageFile(filePath: string): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('unstage_file', {
        filePath,
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function stageAll(): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('stage_all', {
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function unstageAll(): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('unstage_all', {
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function discardFile(filePath: string, isUntracked: boolean = false): Promise<void> {
    if (!currentRepository.value) return
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('discard_file_changes', {
        filePath,
        isUntracked,
        directoryPath: currentRepository.value.path,
      })
    }
    await fetchStatus()
  }

  async function commitChanges(message: string, isAmend: boolean): Promise<boolean> {
    if (!currentRepository.value || !message.trim()) return false
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('commit', {
          message,
          isAmend,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()
      await fetchCommitGraph()
      return true
    } catch (error: any) {
      lastError.value = error?.message || 'Commit failed'
      return false
    }
  }

  async function discoverGit(customPath?: string): Promise<GitBinaryInformation | null> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<GitBinaryInformation>('discover_git_binary', {
          customPath: customPath || null,
        })
        gitBinary.value = result
        return result
      } else {
        // Web / dev mode mock
        const mock: GitBinaryInformation = {
          executable_path: 'git',
          version_string: 'git version 2.43.0 (Mock/Web Preview)',
          is_valid: true,
          has_porcelain_v2_support: true,
        }
        gitBinary.value = mock
        return mock
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Failed to discover Git'
      return null
    }
  }

  async function fetchRecentRepositories(): Promise<RecentRepositoryRecord[]> {
    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const list = await invoke<RecentRepositoryRecord[]>('get_recent_repositories', { limit: 20 })
        recentRepositories.value = list
        return list
      } else {
        // Fallback or demo repos
        recentRepositories.value = [
          {
            id: 'mock-gitverse',
            name: 'gitverse',
            path: '/home/farshid/Desktop/self/gitverse',
            last_opened_timestamp: Date.now(),
            is_pinned: true,
          },
        ]
        return recentRepositories.value
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Failed to fetch recents'
      return []
    }
  }

  async function openRepository(directoryPath: string): Promise<RepositoryOverview | null> {
    isLoading.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const overview = await invoke<RepositoryOverview>('open_repository', {
          directoryPath,
        })
        currentRepository.value = overview
        loadFavoriteBranches()
        await fetchRecentRepositories()
        await fetchStatus()
        await fetchReferences()
        await fetchCommitGraph()
        await setupRealtimeListener()
        if (import.meta.client) {
          try {
            localStorage.setItem(LAST_OPENED_REPO_STORAGE_KEY, directoryPath)
          } catch (error) {
            console.error('Failed to save last opened repo:', error)
          }
        }
        return overview
      } else {
        // Web preview mock
        const overview: RepositoryOverview = {
          path: directoryPath,
          name: directoryPath.split('/').filter(Boolean).pop() || 'repository',
          current_branch_name: 'master',
          head_commit_hash: '3ad5d4c9f1a2',
          is_empty: false,
          is_detached_head: false,
          working_tree_is_clean: true,
          ahead_count: 0,
          behind_count: 0,
        }
        currentRepository.value = overview
        await fetchReferences()
        await fetchCommitGraph()
        if (import.meta.client) {
          try {
            localStorage.setItem(LAST_OPENED_REPO_STORAGE_KEY, directoryPath)
          } catch (error) {
            console.error('Failed to save last opened repo:', error)
          }
        }
        return overview
      }
    } catch (error: any) {
      lastError.value = error?.message || `Failed to open ${directoryPath}`
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function promptAndOpenRepository(): Promise<RepositoryOverview | null> {
    try {
      if (await isTauriEnvironment()) {
        const { open } = await import('@tauri-apps/plugin-dialog')
        const selected = await open({
          directory: true,
          multiple: false,
          title: 'Select Git Repository',
        })
        if (selected && typeof selected === 'string') {
          return await openRepository(selected)
        }
        return null
      } else {
        // Web mock
        const samplePath = '/home/farshid/Desktop/self/gitverse'
        return await openRepository(samplePath)
      }
    } catch (error: any) {
      lastError.value = error?.message || 'Directory picker failed'
      return null
    }
  }

  async function fetchFileDiff(filePath: string, isStaged: boolean): Promise<FileDiff | null> {
    selectedDiffTarget.value = { filePath, isStaged }
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
        // Mock diff for web environment preview
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
                { line_type: 'context', old_line_number: 1, new_line_number: 1, content: '// Sample file for preview' },
                { line_type: 'deletion', old_line_number: 2, new_line_number: null, content: '-export const isFeatureEnabled = false' },
                { line_type: 'addition', old_line_number: null, new_line_number: 2, content: '+export const isFeatureEnabled = true' },
                { line_type: 'addition', old_line_number: null, new_line_number: 3, content: '+export const mode = "modern"' },
                { line_type: 'addition', old_line_number: null, new_line_number: 4, content: '+export const version = 2' },
                { line_type: 'context', old_line_number: 3, new_line_number: 5, content: '' },
                { line_type: 'context', old_line_number: 4, new_line_number: 6, content: 'export function execute() {' },
              ],
            },
          ],
        }
        currentDiff.value = mockDiff
        return mockDiff
      }
    } catch (error: any) {
      console.error('Failed to fetch file diff:', error)
      lastError.value = error?.message || 'Failed to fetch diff'
      currentDiff.value = null
      return null
    } finally {
      isLoadingDiff.value = false
    }
  }

  async function fetchCommitGraph(skip = 0, limit = 100, branchOrReference?: string): Promise<CommitGraphHistoryResponse | null> {
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
        // Mock commits for web preview
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
                { parent_hash: 'a991385ed2ff207cba10ab91977e8a6230644992', from_lane: 0, to_lane: 0, is_merge_edge: false },
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
                { parent_hash: '85ec25a99fc173af066eb513685e13e306b72dbc', from_lane: 0, to_lane: 0, is_merge_edge: false },
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
    } catch (error: any) {
      console.error('Failed to fetch commit graph:', error)
      lastError.value = error?.message || 'Failed to fetch commit history'
      return null
    } finally {
      isLoadingGraph.value = false
    }
  }

  async function fetchReferences(): Promise<RepositoryReferences | null> {
    if (!currentRepository.value) {
      repositoryReferences.value = null
      return null
    }

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const refs = await invoke<RepositoryReferences>('get_repository_references', {
          directoryPath: currentRepository.value.path,
        })
        repositoryReferences.value = refs
        return refs
      } else {
        // Web mock
        const mockRefs: RepositoryReferences = {
          branches: [
            { name: 'master', is_head: true, upstream_name: 'origin/master', ahead_count: 0, behind_count: 0, head_commit_hash: '5897482' },
            { name: 'feature/diff-viewer', is_head: false, upstream_name: null, ahead_count: 0, behind_count: 0, head_commit_hash: 'a991385' },
          ],
          remote_branches: [
            { name: 'origin/master', remote_name: 'origin', head_commit_hash: '5897482' },
          ],
          tags: [
            { name: 'v0.1.0-alpha', commit_hash: '5897482' },
          ],
          remotes: [
            { name: 'origin', fetch_url: 'https://github.com/user/gitverse.git', push_url: 'https://github.com/user/gitverse.git' },
          ],
        }
        repositoryReferences.value = mockRefs
        return mockRefs
      }
    } catch (error: any) {
      console.error('Failed to fetch references:', error)
      lastError.value = error?.message || 'Failed to fetch references'
      return null
    }
  }

  async function checkoutBranch(branchName: string, createIfMissing = false, startPoint?: string): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperating.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('checkout_branch', {
          branchName,
          createIfMissing,
          startPoint: startPoint || null,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error: any) {
      lastError.value = error?.message || `Failed to checkout ${branchName}`
      return false
    } finally {
      isOperating.value = false
    }
  }

  async function createBranch(branchName: string, startPoint?: string): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperating.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('create_branch', {
          branchName,
          startPoint: startPoint || null,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error: any) {
      lastError.value = error?.message || `Failed to create branch ${branchName}`
      return false
    } finally {
      isOperating.value = false
    }
  }

  async function deleteBranch(branchName: string, force = false): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperating.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('delete_branch', {
          branchName,
          force,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error: any) {
      lastError.value = error?.message || `Failed to delete branch ${branchName}`
      return false
    } finally {
      isOperating.value = false
    }
  }

  async function fetchRemote(remoteName?: string): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperating.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('fetch_remote', {
          remoteName: remoteName || null,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error: any) {
      lastError.value = error?.message || 'Fetch failed'
      return false
    } finally {
      isOperating.value = false
    }
  }

  async function pullChanges(remoteName?: string, branchName?: string): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperating.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('pull_changes', {
          remoteName: remoteName || null,
          branchName: branchName || null,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error: any) {
      lastError.value = error?.message || 'Pull failed'
      return false
    } finally {
      isOperating.value = false
    }
  }

  async function pushChanges(remoteName?: string, branchName?: string, setUpstream = false): Promise<boolean> {
    if (!currentRepository.value) return false
    isOperating.value = true
    lastError.value = null

    try {
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('push_changes', {
          remoteName: remoteName || null,
          branchName: branchName || null,
          setUpstream,
          directoryPath: currentRepository.value.path,
        })
      }
      await fetchStatus()
      await fetchReferences()
      await fetchCommitGraph()
      return true
    } catch (error: any) {
      lastError.value = error?.message || 'Push failed'
      return false
    } finally {
      isOperating.value = false
    }
  }

  function getFavoritesStorageKey(): string {
    return `gitverse-favorites-${currentRepository.value?.path || 'default'}`
  }

  function loadFavoriteBranches(): void {
    if (import.meta.client && currentRepository.value) {
      try {
        const stored = localStorage.getItem(getFavoritesStorageKey())
        favoriteBranches.value = stored ? JSON.parse(stored) : []
      } catch (error) {
        console.error('Failed to load favorite branches:', error)
        favoriteBranches.value = []
      }
    }
  }

  function toggleFavoriteBranch(branchName: string): void {
    const set = new Set(favoriteBranches.value)
    if (set.has(branchName)) {
      set.delete(branchName)
    } else {
      set.add(branchName)
    }
    favoriteBranches.value = Array.from(set)
    if (import.meta.client) {
      try {
        localStorage.setItem(getFavoritesStorageKey(), JSON.stringify(favoriteBranches.value))
      } catch (error) {
        console.error('Failed to save favorite branches:', error)
      }
    }
  }

  function isFavoriteBranch(branchName: string): boolean {
    return favoriteBranches.value.includes(branchName)
  }

  function clearDiffSelection(): void {
    selectedDiffTarget.value = null
    currentDiff.value = null
  }

  function clearCommitSelection(): void {
    selectedCommitDetails.value = null
    selectedCommitFilePath.value = null
  }

  async function fetchCommitDetails(commitHash: string): Promise<CommitDetails | null> {
    if (!currentRepository.value) {
      selectedCommitDetails.value = null
      return null
    }

    isLoadingCommitDetails.value = true
    selectedCommitHash.value = commitHash
    selectedDiffTarget.value = null

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
            { path: 'src/main.rs', original_path: null, status: 'modified', additions: 12, deletions: 2 },
            { path: 'src/domain/commit.rs', original_path: null, status: 'added', additions: 3, deletions: 1 },
          ],
        }
        selectedCommitDetails.value = mockDetails
        await fetchCommitFileDiff(commitHash, mockDetails.changed_files[0]?.path)
        return mockDetails
      }
    } catch (error: any) {
      console.error('Failed to fetch commit details:', error)
      lastError.value = error?.message || 'Failed to fetch commit details'
      selectedCommitDetails.value = null
      return null
    } finally {
      isLoadingCommitDetails.value = false
    }
  }

  async function fetchCommitFileDiff(commitHash: string, filePath?: string): Promise<FileDiff | null> {
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
                { line_type: 'context', old_line_number: 1, new_line_number: 1, content: '// Commit diff preview' },
                { line_type: 'addition', old_line_number: null, new_line_number: 2, content: '+// Added in commit' },
                { line_type: 'context', old_line_number: 2, new_line_number: 3, content: '' },
              ],
            },
          ],
        }
        currentDiff.value = mockDiff
        return mockDiff
      }
    } catch (error: any) {
      console.error('Failed to fetch commit file diff:', error)
      lastError.value = error?.message || 'Failed to fetch commit diff'
      currentDiff.value = null
      return null
    } finally {
      isLoadingDiff.value = false
    }
  }

  async function closeRepository(): Promise<void> {
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('close_repository')
    }
    if (import.meta.client) {
      try {
        localStorage.removeItem(LAST_OPENED_REPO_STORAGE_KEY)
      } catch {
        // ignore
      }
    }
    currentRepository.value = null
    repositoryReferences.value = null
    commitGraph.value = null
    selectedCommitHash.value = null
    favoriteBranches.value = []
    clearDiffSelection()
    clearCommitSelection()
  }

  async function restoreLastOpenedRepository(): Promise<RepositoryOverview | null> {
    if (!import.meta.client) return null

    let savedPath: string | null = null
    try {
      savedPath = localStorage.getItem(LAST_OPENED_REPO_STORAGE_KEY)
    } catch {
      // ignore
    }

    if (savedPath) {
      try {
        const overview = await openRepository(savedPath)
        if (overview) return overview
      } catch (error) {
        console.warn('Failed to restore last opened repo from saved path:', savedPath, error)
        try {
          localStorage.removeItem(LAST_OPENED_REPO_STORAGE_KEY)
        } catch {
          // ignore
        }
      }
    }

    // Fallback: check SQLite recents
    try {
      const recents = await fetchRecentRepositories()
      if (recents.length > 0 && recents[0]?.path) {
        return await openRepository(recents[0].path)
      }
    } catch (error) {
      console.warn('Failed to restore repository from recents:', error)
    }

    return null
  }

  return {
    currentRepository,
    repositoryStatus,
    repositoryReferences,
    recentRepositories,
    gitBinary,
    selectedDiffTarget,
    currentDiff,
    commitGraph,
    selectedCommitHash,
    selectedCommitDetails,
    selectedCommitFilePath,
    favoriteBranches,
    isLoading,
    isLoadingDiff,
    isLoadingGraph,
    isLoadingCommitDetails,
    isOperating,
    lastError,
    discoverGit,
    fetchStatus,
    fetchReferences,
    fetchCommitGraph,
    fetchFileDiff,
    fetchCommitDetails,
    fetchCommitFileDiff,
    clearCommitSelection,
    toggleFavoriteBranch,
    isFavoriteBranch,
    loadFavoriteBranches,
    checkoutBranch,
    createBranch,
    deleteBranch,
    fetchRemote,
    pullChanges,
    pushChanges,
    clearDiffSelection,
    fetchRecentRepositories,
    openRepository,
    promptAndOpenRepository,
    closeRepository,
    restoreLastOpenedRepository,
    stageFile,
    unstageFile,
    stageAll,
    unstageAll,
    discardFile,
    commitChanges,
  }
}
