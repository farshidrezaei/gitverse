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
  const activeTabId = safeState<string>('workspace-active-tab-id', () => '')

  const activeTab = computed<WorkspaceTab | undefined>(() => {
    return tabs.value.find(tab => tab.id === activeTabId.value) || tabs.value[0]
  })

  function getRepositoryTabIdentifier(repositoryPath: string): string {
    return `workspace:${repositoryPath}`
  }

  function getDiffTabIdentifier(options: OpenDiffTabOptions): string {
    const repositoryPrefix = options.repositoryPath ? `${options.repositoryPath}:` : ''
    if (options.comparisonBaseHash && options.comparisonTargetHash) {
      return `diff:compare:${repositoryPrefix}${options.comparisonBaseHash}:${options.comparisonTargetHash}:${options.filePath}`
    }
    if (options.commitHash) {
      return `diff:commit:${repositoryPrefix}${options.commitHash}:${options.filePath}`
    }
    if (options.isStaged) {
      return `diff:staged:${repositoryPrefix}${options.filePath}`
    }
    return `diff:working:${repositoryPrefix}${options.filePath}`
  }

  function getFileNameFromPath(filePath: string): string {
    const parts = filePath.split('/')
    return parts[parts.length - 1] || filePath
  }

  function openWorkspaceTab(repositoryPath: string, repositoryName: string): WorkspaceMainTab {
    const tabIdentifier = getRepositoryTabIdentifier(repositoryPath)
    const existingIndex = tabs.value.findIndex(tab => tab.id === tabIdentifier)

    if (existingIndex !== -1) {
      const existingTab = tabs.value[existingIndex] as WorkspaceMainTab
      existingTab.repositoryName = repositoryName
      activeTabId.value = existingTab.id
      return existingTab
    }

    const mainTab: WorkspaceMainTab = {
      id: tabIdentifier,
      type: 'workspace',
      title: repositoryName || 'Workspace',
      repositoryPath,
      repositoryName,
      isClosable: true,
      isPinned: true,
    }

    tabs.value.push(mainTab)
    activeTabId.value = tabIdentifier
    return mainTab
  }

  function initializeWorkspaceTab(repositoryPath: string, repositoryName: string): void {
    openWorkspaceTab(repositoryPath, repositoryName)
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
          repositoryPath: options.repositoryPath,
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
      repositoryPath: options.repositoryPath,
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
      const nextActiveIndex = Math.max(0, targetIndex - 1)
      const nextTab = tabs.value[nextActiveIndex]
      activeTabId.value = nextTab ? nextTab.id : ''
    }
  }

  function closeAllDiffTabs(): void {
    tabs.value = tabs.value.filter(tab => tab.type === 'workspace')
    if (!tabs.value.some(tab => tab.id === activeTabId.value)) {
      activeTabId.value = tabs.value[0]?.id || ''
    }
  }

  function closeOtherTabs(tabIdentifier: string): void {
    const targetTab = tabs.value.find(tab => tab.id === tabIdentifier)
    if (!targetTab) return

    tabs.value = tabs.value.filter(
      tab => tab.type === 'workspace' || tab.id === tabIdentifier,
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
      openWorkspaceTab(repositoryPath, repositoryName)
    } else {
      tabs.value = []
      activeTabId.value = ''
    }
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    openWorkspaceTab,
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
