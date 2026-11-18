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
import { useStashController } from '~/composables/useStashController'
import { useCommitOperations } from '~/composables/useCommitOperations'
import { useConflictController } from '~/composables/useConflictController'
import { buildHunkPatch, buildSelectedLinesPatch } from '~/utils/patchGenerator'

export function useWorkspaceManager() {
  const repositoryWorkspace = useRepositoryWorkspace()
  const stagingController = useStagingController()
  const branchController = useBranchController()
  const commitGraphController = useCommitGraph()
  const diffViewerController = useDiffViewer()
  const commitInspectorController = useCommitInspector()
  const remoteController = useRemoteController()
  const stashController = useStashController()
  const commitOperations = useCommitOperations()
  const conflictController = useConflictController()

  // Combined operational and loading states for backward compatibility
  const isLoading = computed(
    () => repositoryWorkspace.isLoadingRepository.value || stagingController.isOperatingStaging.value
  )
  const isOperating = computed(
    () =>
      repositoryWorkspace.isOperating.value ||
      stagingController.isOperatingStaging.value ||
      branchController.isOperatingBranches.value ||
      remoteController.isOperatingRemote.value ||
      commitOperations.isOperatingCommit.value ||
      conflictController.isResolvingConflict.value
  )

  async function openRepository(directoryPath: string): Promise<RepositoryOverview | null> {
    const overview = await repositoryWorkspace.openRepository(directoryPath)
    if (overview) {
      branchController.loadFavoriteBranches()
      await stagingController.fetchStatus()
      await branchController.fetchReferences()
      await commitGraphController.fetchCommitGraph()
      await stashController.fetchStashes()
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
    stashController.stashRecords.value = []
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
      await stashController.fetchStashes()
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

  async function stageHunk(hunkIndex: number): Promise<boolean> {
    if (!diffViewerController.currentDiff.value) return false
    const patch = buildHunkPatch(diffViewerController.currentDiff.value, hunkIndex)
    if (!patch) return false
    const success = await stagingController.stagePatch(patch)
    if (success && stagingController.selectedDiffTarget.value) {
      await fetchFileDiff(
        stagingController.selectedDiffTarget.value.filePath,
        stagingController.selectedDiffTarget.value.isStaged,
      )
    }
    return success
  }

  async function unstageHunk(hunkIndex: number): Promise<boolean> {
    if (!diffViewerController.currentDiff.value) return false
    const patch = buildHunkPatch(diffViewerController.currentDiff.value, hunkIndex)
    if (!patch) return false
    const success = await stagingController.unstagePatch(patch)
    if (success && stagingController.selectedDiffTarget.value) {
      await fetchFileDiff(
        stagingController.selectedDiffTarget.value.filePath,
        stagingController.selectedDiffTarget.value.isStaged,
      )
    }
    return success
  }

  async function discardHunk(hunkIndex: number): Promise<boolean> {
    if (!diffViewerController.currentDiff.value) return false
    const patch = buildHunkPatch(diffViewerController.currentDiff.value, hunkIndex)
    if (!patch) return false
    const success = await stagingController.discardPatch(patch)
    if (success && stagingController.selectedDiffTarget.value) {
      await fetchFileDiff(
        stagingController.selectedDiffTarget.value.filePath,
        stagingController.selectedDiffTarget.value.isStaged,
      )
    }
    return success
  }

  async function stageSelectedLines(hunkIndex: number, lineIndices: number[]): Promise<boolean> {
    if (!diffViewerController.currentDiff.value) return false
    const patch = buildSelectedLinesPatch(diffViewerController.currentDiff.value, hunkIndex, lineIndices, 'stage')
    if (!patch) return false
    const success = await stagingController.stagePatch(patch)
    if (success && stagingController.selectedDiffTarget.value) {
      await fetchFileDiff(
        stagingController.selectedDiffTarget.value.filePath,
        stagingController.selectedDiffTarget.value.isStaged,
      )
    }
    return success
  }

  async function unstageSelectedLines(hunkIndex: number, lineIndices: number[]): Promise<boolean> {
    if (!diffViewerController.currentDiff.value) return false
    const patch = buildSelectedLinesPatch(diffViewerController.currentDiff.value, hunkIndex, lineIndices, 'unstage')
    if (!patch) return false
    const success = await stagingController.unstagePatch(patch)
    if (success && stagingController.selectedDiffTarget.value) {
      await fetchFileDiff(
        stagingController.selectedDiffTarget.value.filePath,
        stagingController.selectedDiffTarget.value.isStaged,
      )
    }
    return success
  }

  async function discardSelectedLines(hunkIndex: number, lineIndices: number[]): Promise<boolean> {
    if (!diffViewerController.currentDiff.value) return false
    const patch = buildSelectedLinesPatch(diffViewerController.currentDiff.value, hunkIndex, lineIndices, 'discard')
    if (!patch) return false
    const success = await stagingController.discardPatch(patch)
    if (success && stagingController.selectedDiffTarget.value) {
      await fetchFileDiff(
        stagingController.selectedDiffTarget.value.filePath,
        stagingController.selectedDiffTarget.value.isStaged,
      )
    }
    return success
  }

  async function fetchCommitDetails(commitHash: string): Promise<CommitDetails | null> {
    commitGraphController.selectCommit(commitHash)
    stagingController.clearDiffSelection()
    return await commitInspectorController.fetchCommitDetails(commitHash)
  }

  async function compareCommits(baseHash: string, targetHash: string) {
    stagingController.clearDiffSelection()
    return await commitInspectorController.compareCommits(baseHash, targetHash)
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
    listDirectoryFolders: repositoryWorkspace.listDirectoryFolders,
    closeRepository,
    restoreLastOpenedRepository,
    fetchRecentRepositories: repositoryWorkspace.fetchRecentRepositories,
    removeRecentRepository: repositoryWorkspace.removeRecentRepository,
    clearRecentRepositories: repositoryWorkspace.clearRecentRepositories,

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
    currentBranchName: branchController.currentBranchName,
    fetchReferences: branchController.fetchReferences,
    checkoutBranch,
    createBranch,
    deleteBranch,
    renameBranch: branchController.renameBranch,
    mergeBranch: branchController.mergeBranch,
    rebaseBranch: branchController.rebaseBranch,
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
    activeComparison: commitInspectorController.activeComparison,
    isComparing: commitInspectorController.isComparing,
    selectedCommitFilePath: commitInspectorController.selectedCommitFilePath,
    isLoadingCommitDetails: commitInspectorController.isLoadingCommitDetails,
    fetchCommitDetails,
    fetchCommitFileDiff: commitInspectorController.fetchCommitFileDiff,
    compareCommits,
    fetchComparisonFileDiff: commitInspectorController.fetchComparisonFileDiff,
    swapComparison: commitInspectorController.swapComparison,
    clearComparison: commitInspectorController.clearComparison,
    clearCommitSelection: commitInspectorController.clearCommitSelection,

    // Diff Viewer
    currentDiff: diffViewerController.currentDiff,
    isLoadingDiff: diffViewerController.isLoadingDiff,
    diffViewMode: diffViewerController.diffViewMode,
    fetchFileDiff,
    stageHunk,
    unstageHunk,
    discardHunk,
    stageSelectedLines,
    unstageSelectedLines,
    discardSelectedLines,

    // Remotes
    fetchRemote,
    pullChanges,
    pushChanges,
    addRemote: remoteController.addRemote,
    removeRemote: remoteController.removeRemote,
    renameRemote: remoteController.renameRemote,
    setRemoteUrl: remoteController.setRemoteUrl,
    copyRemoteUrl: remoteController.copyRemoteUrl,

    // Stashes
    stashRecords: stashController.stashRecords,
    isLoadingStashes: stashController.isLoadingStashes,
    fetchStashes: stashController.fetchStashes,
    saveStash: stashController.saveStash,
    applyStash: stashController.applyStash,
    popStash: stashController.popStash,
    dropStash: stashController.dropStash,

    // Commit Operations
    cherryPickCommit: commitOperations.cherryPickCommit,
    revertCommit: commitOperations.revertCommit,
    resetToCommit: commitOperations.resetToCommit,

    // Conflict & Operation Controls
    activeConflictFile: conflictController.activeConflictFile,
    isLoadingConflict: conflictController.isLoadingConflict,
    isResolvingConflict: conflictController.isResolvingConflict,
    loadConflictDetails: conflictController.loadConflictDetails,
    resolveConflict: conflictController.resolveConflict,
    writeConflictResolvedContent: conflictController.writeConflictResolvedContent,
    abortOperation: conflictController.abortOperation,
    continueOperation: conflictController.continueOperation,
  }
}
