import type {
  WorkspaceTab,
  WorkspaceMainTab,
  WorkspaceDiffTab,
  OpenDiffTabOptions,
} from '~/types/workspaceTab'

function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useWorkspaceTabController() {
  const tabs = safeState<WorkspaceTab[]>('workspace-tabs-list', () => [])
  const activeTabId = safeState<string>('workspace-active-tab-id', () => 'workspace-main')

  const activeTab = computed<WorkspaceTab | undefined>(() => {
    return tabs.value.find(tab => tab.id === activeTabId.value) || tabs.value[0]
  })

  function getDiffTabIdentifier(options: OpenDiffTabOptions): string {
    if (options.comparisonBaseHash && options.comparisonTargetHash) {
      return `diff:compare:${options.comparisonBaseHash}:${options.comparisonTargetHash}:${options.filePath}`
    }
    if (options.commitHash) {
      return `diff:commit:${options.commitHash}:${options.filePath}`
    }
    if (options.isStaged) {
      return `diff:staged:${options.filePath}`
    }
    return `diff:working:${options.filePath}`
  }

  function getFileNameFromPath(filePath: string): string {
    const parts = filePath.split('/')
    return parts[parts.length - 1] || filePath
  }

  function initializeWorkspaceTab(repositoryPath: string, repositoryName: string): void {
    const mainTabId = 'workspace-main'
    const existingMainTab = tabs.value.find(tab => tab.id === mainTabId)

    if (existingMainTab && existingMainTab.type === 'workspace') {
      const typedMainTab = existingMainTab as WorkspaceMainTab
      typedMainTab.repositoryPath = repositoryPath
      typedMainTab.repositoryName = repositoryName
      return
    }

    const mainTab: WorkspaceMainTab = {
      id: mainTabId,
      type: 'workspace',
      title: repositoryName || 'Workspace',
      repositoryPath,
      repositoryName,
      isClosable: false,
      isPinned: true,
    }

    tabs.value = [mainTab]
    activeTabId.value = mainTabId
  }

  function openDiffTab(options: OpenDiffTabOptions, shouldActivate = true): WorkspaceDiffTab {
    const tabIdentifier = getDiffTabIdentifier(options)
    const existingTabIndex = tabs.value.findIndex(tab => tab.id === tabIdentifier)

    if (existingTabIndex !== -1) {
      const existingTab = tabs.value[existingTabIndex] as WorkspaceDiffTab
      if (options.isPinned) {
        existingTab.isPinned = true
      }
      if (shouldActivate) {
        activeTabId.value = existingTab.id
      }
      return existingTab
    }

    const isExplicitlyPinned = options.isPinned ?? true

    // If an unpinned preview tab exists and this new open is NOT pinned, reuse the preview tab
    if (!isExplicitlyPinned) {
      const previewTabIndex = tabs.value.findIndex(
        tab => tab.type === 'diff' && !tab.isPinned,
      )

      if (previewTabIndex !== -1) {
        const replacementTab: WorkspaceDiffTab = {
          id: tabIdentifier,
          type: 'diff',
          title: getFileNameFromPath(options.filePath),
          filePath: options.filePath,
          isStaged: options.isStaged,
          commitHash: options.commitHash,
          comparisonBaseHash: options.comparisonBaseHash,
          comparisonTargetHash: options.comparisonTargetHash,
          changeStatus: options.changeStatus,
          isClosable: true,
          isPinned: false,
        }

        tabs.value[previewTabIndex] = replacementTab
        if (shouldActivate) {
          activeTabId.value = replacementTab.id
        }
        return replacementTab
      }
    }

    const newDiffTab: WorkspaceDiffTab = {
      id: tabIdentifier,
      type: 'diff',
      title: getFileNameFromPath(options.filePath),
      filePath: options.filePath,
      isStaged: options.isStaged,
      commitHash: options.commitHash,
      comparisonBaseHash: options.comparisonBaseHash,
      comparisonTargetHash: options.comparisonTargetHash,
      changeStatus: options.changeStatus,
      isClosable: true,
      isPinned: isExplicitlyPinned,
    }

    tabs.value.push(newDiffTab)

    if (shouldActivate) {
      activeTabId.value = newDiffTab.id
    }

    return newDiffTab
  }

  function pinTab(tabIdentifier: string): void {
    const targetTab = tabs.value.find(tab => tab.id === tabIdentifier)
    if (targetTab) {
      targetTab.isPinned = true
    }
  }

  function closeTab(tabIdentifier: string): void {
    const targetIndex = tabs.value.findIndex(tab => tab.id === tabIdentifier)
    if (targetIndex === -1) return

    const targetTab = tabs.value[targetIndex]
    if (!targetTab || !targetTab.isClosable) return

    const isClosingActive = activeTabId.value === tabIdentifier

    tabs.value.splice(targetIndex, 1)

    if (isClosingActive) {
      // Switch to previous tab if available, else next, else main workspace
      const nextActiveIndex = Math.max(0, targetIndex - 1)
      const nextTab = tabs.value[nextActiveIndex]
      activeTabId.value = nextTab ? nextTab.id : 'workspace-main'
    }
  }

  function closeAllDiffTabs(): void {
    tabs.value = tabs.value.filter(tab => !tab.isClosable)
    activeTabId.value = tabs.value[0]?.id || 'workspace-main'
  }

  function closeOtherTabs(tabIdentifier: string): void {
    tabs.value = tabs.value.filter(
      tab => !tab.isClosable || tab.id === tabIdentifier,
    )
    activeTabId.value = tabIdentifier
  }

  function activateTab(tabIdentifier: string): void {
    const exists = tabs.value.some(tab => tab.id === tabIdentifier)
    if (exists) {
      activeTabId.value = tabIdentifier
    }
  }

  function resetTabs(repositoryPath?: string, repositoryName?: string): void {
    if (repositoryPath && repositoryName) {
      initializeWorkspaceTab(repositoryPath, repositoryName)
    } else {
      tabs.value = []
      activeTabId.value = 'workspace-main'
    }
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    initializeWorkspaceTab,
    openDiffTab,
    pinTab,
    closeTab,
    closeAllDiffTabs,
    closeOtherTabs,
    activateTab,
    resetTabs,
  }
}
