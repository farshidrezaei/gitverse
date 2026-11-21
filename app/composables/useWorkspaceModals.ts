function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useWorkspaceModals() {
  const isFolderPickerModalOpen = safeState<boolean>('modal-folder-picker-open', () => false)
  const isReflogModalOpen = safeState<boolean>('modal-reflog-open', () => false)
  const isBisectModalOpen = safeState<boolean>('modal-bisect-open', () => false)
  const isExportPatchModalOpen = safeState<boolean>('modal-export-patch-open', () => false)
  const isApplyPatchModalOpen = safeState<boolean>('modal-apply-patch-open', () => false)
  const isSubmoduleModalOpen = safeState<boolean>('modal-submodule-open', () => false)
  const isInsightsModalOpen = safeState<boolean>('modal-insights-open', () => false)
  const isFileHistoryModalOpen = safeState<boolean>('modal-file-history-open', () => false)
  const isSearchModalOpen = safeState<boolean>('modal-search-open', () => false)
  const isCommitQueueModalOpen = safeState<boolean>('modal-commit-queue-open', () => false)
  const commitQueueMode = safeState<'cherryPick' | 'revert'>('modal-commit-queue-mode', () => 'cherryPick')
  const isGitBundleModalOpen = safeState<boolean>('modal-git-bundle-open', () => false)
  const isExportArchiveModalOpen = safeState<boolean>('modal-export-archive-open', () => false)
  const isLfsManagerModalOpen = safeState<boolean>('modal-lfs-manager-open', () => false)
  const isCustomActionsModalOpen = safeState<boolean>('modal-custom-actions-open', () => false)

  function openFolderPicker(): void {
    isFolderPickerModalOpen.value = true
  }

  function openReflog(): void {
    isReflogModalOpen.value = true
  }

  function openBisect(): void {
    isBisectModalOpen.value = true
  }

  function openExportPatch(): void {
    isExportPatchModalOpen.value = true
  }

  function openApplyPatch(): void {
    isApplyPatchModalOpen.value = true
  }

  function openSubmodules(): void {
    isSubmoduleModalOpen.value = true
  }

  function openInsights(): void {
    isInsightsModalOpen.value = true
  }

  function openFileHistory(): void {
    isFileHistoryModalOpen.value = true
  }

  function openSearch(): void {
    isSearchModalOpen.value = true
  }

  function openCommitQueue(mode: 'cherryPick' | 'revert' = 'cherryPick'): void {
    commitQueueMode.value = mode
    isCommitQueueModalOpen.value = true
  }

  function openGitBundle(): void {
    isGitBundleModalOpen.value = true
  }

  function openExportArchive(): void {
    isExportArchiveModalOpen.value = true
  }

  function openLfsManager(): void {
    isLfsManagerModalOpen.value = true
  }

  function openCustomActions(): void {
    isCustomActionsModalOpen.value = true
  }

  return {
    isFolderPickerModalOpen,
    isReflogModalOpen,
    isBisectModalOpen,
    isExportPatchModalOpen,
    isApplyPatchModalOpen,
    isSubmoduleModalOpen,
    isInsightsModalOpen,
    isFileHistoryModalOpen,
    isSearchModalOpen,
    isCommitQueueModalOpen,
    commitQueueMode,
    isGitBundleModalOpen,
    isExportArchiveModalOpen,
    isLfsManagerModalOpen,
    isCustomActionsModalOpen,
    openFolderPicker,
    openReflog,
    openBisect,
    openExportPatch,
    openApplyPatch,
    openSubmodules,
    openInsights,
    openFileHistory,
    openSearch,
    openCommitQueue,
    openGitBundle,
    openExportArchive,
    openLfsManager,
    openCustomActions,
  }
}
