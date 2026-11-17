import type {
  CommitDetails,
  CommitGraphHistoryResponse,
  DetailedRepositoryStatus,
  FileDiff,
  GitBinaryInformation,
  RecentRepositoryRecord,
  RepositoryOverview,
  RepositoryReferences,
} from '~/types/repository'

export function useWorkspaceManager() {
  const repositoryWorkspace = useRepositoryWorkspace()
  const stagingController = useStagingController()
  const branchController = useBranchController()
  const commitGraphController = useCommitGraph()
  const diffViewerController = useDiffViewer()
  const commitInspectorController = useCommitInspector()
  const remoteController = useRemoteController()

  // Combined operational and loading states for backward compatibility
  const isLoading = computed(
    () => repositoryWorkspace.isLoadingRepository.value || stagingController.isOperatingStaging.value
  )
  const isOperating = computed(
    () =>
      repositoryWorkspace.isOperating.value ||
      stagingController.isOperatingStaging.value ||
      branchController.isOperatingBranches.value ||
      remoteController.isOperatingRemote.value
  )

  async function openRepository(directoryPath: string): Promise<RepositoryOverview | null> {
    const overview = await repositoryWorkspace.openRepository(directoryPath)
    if (overview) {
      branchController.loadFavoriteBranches()
      await stagingController.fetchStatus()
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
      await stagingController.setupRealtimeListener()
    }
    return overview
  }

  async function closeRepository(): Promise<void> {
    await repositoryWorkspace.closeRepository()
    stagingController.repositoryStatus.value = null
    branchController.repositoryReferences.value = null
    commitGraphController.commitGraph.value = null
    commitGraphController.selectedCommitHash.value = null
    branchController.favoriteBranches.value = []
    diffViewerController.clearDiff()
    commitInspectorController.clearCommitSelection()
  }

  async function restoreLastOpenedRepository(): Promise<RepositoryOverview | null> {
    const overview = await repositoryWorkspace.restoreLastOpenedRepository()
    if (overview) {
      branchController.loadFavoriteBranches()
      await stagingController.fetchStatus()
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
      await stagingController.setupRealtimeListener()
    }
    return overview
  }

  async function checkoutBranch(
    branchName: string,
    createIfMissing = false,
    startPoint?: string
  ): Promise<boolean> {
    const success = await branchController.checkoutBranch(branchName, createIfMissing, startPoint)
    if (success) {
      await stagingController.fetchStatus()
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
    }
    return success
  }

  async function createBranch(branchName: string, startPoint?: string): Promise<boolean> {
    const success = await branchController.createBranch(branchName, startPoint)
    if (success) {
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
    }
    return success
  }

  async function deleteBranch(branchName: string, force = false): Promise<boolean> {
    const success = await branchController.deleteBranch(branchName, force)
    if (success) {
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
    }
    return success
  }

  async function fetchRemote(remoteName?: string): Promise<boolean> {
    const success = await remoteController.fetchRemote(remoteName)
    if (success) {
      await stagingController.fetchStatus()
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
    }
    return success
  }

  async function pullChanges(remoteName?: string, branchName?: string): Promise<boolean> {
    const success = await remoteController.pullChanges(remoteName, branchName)
    if (success) {
      await stagingController.fetchStatus()
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
    }
    return success
  }

  async function pushChanges(
    remoteName?: string,
    branchName?: string,
    setUpstream = false
  ): Promise<boolean> {
    const success = await remoteController.pushChanges(remoteName, branchName, setUpstream)
    if (success) {
      await stagingController.fetchStatus()
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
    }
    return success
  }

  async function fetchFileDiff(filePath: string, isStaged: boolean): Promise<FileDiff | null> {
    stagingController.selectedDiffTarget.value = { filePath, isStaged }
    return await diffViewerController.fetchFileDiff(filePath, isStaged)
  }

  async function fetchCommitDetails(commitHash: string): Promise<CommitDetails | null> {
    commitGraphController.selectCommit(commitHash)
    stagingController.clearDiffSelection()
    return await commitInspectorController.fetchCommitDetails(commitHash)
  }

  return {
    // Repository
    currentRepository: repositoryWorkspace.currentRepository,
    recentRepositories: repositoryWorkspace.recentRepositories,
    gitBinary: repositoryWorkspace.gitBinary,
    isLoading,
    isOperating,
    lastError: repositoryWorkspace.lastError,
    discoverGit: repositoryWorkspace.discoverGit,
    openRepository,
    promptAndOpenRepository: repositoryWorkspace.promptAndOpenRepository,
    closeRepository,
    restoreLastOpenedRepository,
    fetchRecentRepositories: repositoryWorkspace.fetchRecentRepositories,
    removeRecentRepository: repositoryWorkspace.removeRecentRepository,

    // Staging
    repositoryStatus: stagingController.repositoryStatus,
    selectedDiffTarget: stagingController.selectedDiffTarget,
    stagingViewMode: stagingController.stagingViewMode,
    fetchStatus: stagingController.fetchStatus,
    stageFile: stagingController.stageFile,
    unstageFile: stagingController.unstageFile,
    stageAll: stagingController.stageAll,
    unstageAll: stagingController.unstageAll,
    discardFile: stagingController.discardFile,
    discardAllUnstaged: stagingController.discardAllUnstaged,
    commitChanges: stagingController.commitChanges,
    clearDiffSelection: stagingController.clearDiffSelection,

    // Branches & References
    repositoryReferences: branchController.repositoryReferences,
    favoriteBranches: branchController.favoriteBranches,
    fetchReferences: branchController.fetchReferences,
    checkoutBranch,
    createBranch,
    deleteBranch,
    loadFavoriteBranches: branchController.loadFavoriteBranches,
    toggleFavoriteBranch: branchController.toggleFavoriteBranch,
    isFavoriteBranch: branchController.isFavoriteBranch,

    // Commit Graph
    commitGraph: commitGraphController.commitGraph,
    selectedCommitHash: commitGraphController.selectedCommitHash,
    isLoadingGraph: commitGraphController.isLoadingGraph,
    fetchCommitGraph: commitGraphController.fetchCommitGraph,
    selectCommit: commitGraphController.selectCommit,

    // Commit Inspector
    selectedCommitDetails: commitInspectorController.selectedCommitDetails,
    selectedCommitFilePath: commitInspectorController.selectedCommitFilePath,
    isLoadingCommitDetails: commitInspectorController.isLoadingCommitDetails,
    fetchCommitDetails,
    fetchCommitFileDiff: commitInspectorController.fetchCommitFileDiff,
    clearCommitSelection: commitInspectorController.clearCommitSelection,

    // Diff Viewer
    currentDiff: diffViewerController.currentDiff,
    isLoadingDiff: diffViewerController.isLoadingDiff,
    diffViewMode: diffViewerController.diffViewMode,
    fetchFileDiff,

    // Remotes
    fetchRemote,
    pullChanges,
    pushChanges,
  }
}
