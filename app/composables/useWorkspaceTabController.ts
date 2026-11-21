import type {
  WorkspaceRepositoryTab,
  WorkspaceDiffTab,
  WorkspacePullRequestTab,
  WorkspaceWorkflowRunTab,
  OpenDiffTabOptions,
} from '~/types/workspaceTab'
import type { RemotePullRequest, ProviderAccount, WorkflowRun } from '~/types/provider'

export const LAST_OPENED_REPOSITORY_STORAGE_KEY = 'gitverse_last_opened_repository_path'
export const OPEN_REPOSITORY_TABS_STORAGE_KEY = 'gitverse_open_repository_tabs'

function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useWorkspaceTabController() {
  // Repository-level tabs (Top level)
  const repositoryTabs = safeState<WorkspaceRepositoryTab[]>('workspace-repository-tabs-list', () => [])
  const activeRepositoryPath = safeState<string>('workspace-active-repository-path', () => '')

  // File diff tabs (Scoped by repository path)
  const diffTabs = safeState<WorkspaceDiffTab[]>('workspace-diff-tabs-list', () => [])
  const activeDiffTabId = safeState<string | null>('workspace-active-diff-tab-id', () => null)

  // Dedicated Pull Request tabs (Scoped by repository path)
  const pullRequestTabs = safeState<WorkspacePullRequestTab[]>('workspace-pull-request-tabs-list', () => [])
  const activePullRequestTabId = safeState<string | null>('workspace-active-pr-tab-id', () => null)

  // Dedicated Workflow Run / Actions tabs (Scoped by repository path)
  const workflowRunTabs = safeState<WorkspaceWorkflowRunTab[]>('workspace-workflow-run-tabs-list', () => [])
  const activeWorkflowRunTabId = safeState<string | null>('workspace-active-workflow-run-tab-id', () => null)

  const activeRepositoryTab = computed<WorkspaceRepositoryTab | undefined>(() => {
    return repositoryTabs.value.find(tab => tab.repositoryPath === activeRepositoryPath.value) || repositoryTabs.value[0]
  })

  const currentRepositoryDiffTabs = computed<WorkspaceDiffTab[]>(() => {
    if (!activeRepositoryPath.value) return []
    return diffTabs.value.filter(tab => tab.repositoryPath === activeRepositoryPath.value)
  })

  const currentRepositoryPullRequestTabs = computed<WorkspacePullRequestTab[]>(() => {
    if (!activeRepositoryPath.value) return []
    return pullRequestTabs.value.filter(tab => tab.repositoryPath === activeRepositoryPath.value)
  })

  const currentRepositoryWorkflowRunTabs = computed<WorkspaceWorkflowRunTab[]>(() => {
    if (!activeRepositoryPath.value) return []
    return workflowRunTabs.value.filter(tab => tab.repositoryPath === activeRepositoryPath.value)
  })

  const activeDiffTab = computed<WorkspaceDiffTab | undefined>(() => {
    if (!activeDiffTabId.value) return undefined
    return currentRepositoryDiffTabs.value.find(tab => tab.id === activeDiffTabId.value)
  })

  const activePullRequestTab = computed<WorkspacePullRequestTab | undefined>(() => {
    if (!activePullRequestTabId.value) return undefined
    return currentRepositoryPullRequestTabs.value.find(tab => tab.id === activePullRequestTabId.value)
  })

  const activeWorkflowRunTab = computed<WorkspaceWorkflowRunTab | undefined>(() => {
    if (!activeWorkflowRunTabId.value) return undefined
    return currentRepositoryWorkflowRunTabs.value.find(tab => tab.id === activeWorkflowRunTabId.value)
  })


  function getRepositoryTabIdentifier(repositoryPath: string): string {
    return `workspace:${repositoryPath}`
  }

  function getDiffTabIdentifier(options: OpenDiffTabOptions, repoPath: string): string {
    if (options.comparisonBaseHash && options.comparisonTargetHash) {
      return `diff:compare:${repoPath}:${options.comparisonBaseHash}:${options.comparisonTargetHash}:${options.filePath}`
    }
    if (options.commitHash) {
      return `diff:commit:${repoPath}:${options.commitHash}:${options.filePath}`
    }
    if (options.isStaged) {
      return `diff:staged:${repoPath}:${options.filePath}`
    }
    return `diff:working:${repoPath}:${options.filePath}`
  }

  function getFileNameFromPath(filePath: string): string {
    const parts = filePath.split('/')
    return parts[parts.length - 1] || filePath
  }

  function persistTabsToStorage(): void {
    if (typeof window === 'undefined') return
    try {
      if (repositoryTabs.value.length === 0) {
        window.localStorage.removeItem(OPEN_REPOSITORY_TABS_STORAGE_KEY)
        window.localStorage.removeItem(LAST_OPENED_REPOSITORY_STORAGE_KEY)
      } else {
        window.localStorage.setItem(
          OPEN_REPOSITORY_TABS_STORAGE_KEY,
          JSON.stringify(repositoryTabs.value.map(tab => ({ path: tab.repositoryPath, name: tab.repositoryName }))),
        )
        if (activeRepositoryPath.value) {
          window.localStorage.setItem(LAST_OPENED_REPOSITORY_STORAGE_KEY, activeRepositoryPath.value)
        }
      }
    } catch {
      // ignore
    }
  }

  function openRepositoryTab(repositoryPath: string, repositoryName: string): WorkspaceRepositoryTab {
    const tabIdentifier = getRepositoryTabIdentifier(repositoryPath)
    const existingIndex = repositoryTabs.value.findIndex(tab => tab.repositoryPath === repositoryPath)

    if (existingIndex !== -1) {
      const existingTab = repositoryTabs.value[existingIndex] as WorkspaceRepositoryTab
      existingTab.repositoryName = repositoryName
      activeRepositoryPath.value = repositoryPath
      activeDiffTabId.value = null // Reset active diff view to workbench
      persistTabsToStorage()
      return existingTab
    }

    const newTab: WorkspaceRepositoryTab = {
      id: tabIdentifier,
      repositoryPath,
      repositoryName: repositoryName || 'Repository',
      isClosable: true,
    }

    repositoryTabs.value.push(newTab)
    activeRepositoryPath.value = repositoryPath
    activeDiffTabId.value = null
    persistTabsToStorage()
    return newTab
  }

  function closeRepositoryTab(repositoryPath: string): void {
    const targetIndex = repositoryTabs.value.findIndex(tab => tab.repositoryPath === repositoryPath)
    if (targetIndex === -1) return

    // Clean up all diff tabs belonging to this repository
    diffTabs.value = diffTabs.value.filter(tab => tab.repositoryPath !== repositoryPath)

    const isClosingActive = activeRepositoryPath.value === repositoryPath
    repositoryTabs.value.splice(targetIndex, 1)

    if (isClosingActive) {
      const nextActiveIndex = Math.max(0, targetIndex - 1)
      const nextTab = repositoryTabs.value[nextActiveIndex]
      activeRepositoryPath.value = nextTab ? nextTab.repositoryPath : ''
      activeDiffTabId.value = null
    }

    persistTabsToStorage()
  }

  function closeOtherRepositoryTabs(repositoryPath: string): void {
    repositoryTabs.value = repositoryTabs.value.filter(tab => tab.repositoryPath === repositoryPath)
    diffTabs.value = diffTabs.value.filter(tab => tab.repositoryPath === repositoryPath)
    activeRepositoryPath.value = repositoryPath
    activeDiffTabId.value = null
    persistTabsToStorage()
  }

  function openDiffTab(options: OpenDiffTabOptions, shouldActivate = true): WorkspaceDiffTab {
    const repoPath = options.repositoryPath || activeRepositoryPath.value
    const tabIdentifier = getDiffTabIdentifier(options, repoPath)
    const existingTabIndex = diffTabs.value.findIndex(tab => tab.id === tabIdentifier)

    if (existingTabIndex !== -1) {
      const existingTab = diffTabs.value[existingTabIndex] as WorkspaceDiffTab
      if (options.isPinned) {
        existingTab.isPinned = true
      }
      if (shouldActivate) {
        activeDiffTabId.value = existingTab.id
      }
      return existingTab
    }

    const isExplicitlyPinned = options.isPinned ?? true

    // If an unpinned preview tab exists in this repository and this new open is NOT pinned, reuse it
    if (!isExplicitlyPinned) {
      const previewTabIndex = diffTabs.value.findIndex(
        tab => tab.repositoryPath === repoPath && !tab.isPinned,
      )

      if (previewTabIndex !== -1) {
        const replacementTab: WorkspaceDiffTab = {
          id: tabIdentifier,
          title: getFileNameFromPath(options.filePath),
          filePath: options.filePath,
          repositoryPath: repoPath,
          isStaged: options.isStaged,
          commitHash: options.commitHash,
          comparisonBaseHash: options.comparisonBaseHash,
          comparisonTargetHash: options.comparisonTargetHash,
          changeStatus: options.changeStatus,
          isClosable: true,
          isPinned: false,
        }

        diffTabs.value[previewTabIndex] = replacementTab
        if (shouldActivate) {
          activeDiffTabId.value = replacementTab.id
        }
        return replacementTab
      }
    }

    const newDiffTab: WorkspaceDiffTab = {
      id: tabIdentifier,
      title: getFileNameFromPath(options.filePath),
      filePath: options.filePath,
      repositoryPath: repoPath,
      isStaged: options.isStaged,
      commitHash: options.commitHash,
      comparisonBaseHash: options.comparisonBaseHash,
      comparisonTargetHash: options.comparisonTargetHash,
      changeStatus: options.changeStatus,
      isClosable: true,
      isPinned: isExplicitlyPinned,
    }

    diffTabs.value.push(newDiffTab)

    if (shouldActivate) {
      activeDiffTabId.value = newDiffTab.id
    }

    return newDiffTab
  }

  function pinDiffTab(tabIdentifier: string): void {
    const targetTab = diffTabs.value.find(tab => tab.id === tabIdentifier)
    if (targetTab) {
      targetTab.isPinned = true
    }
  }

  function closeDiffTab(tabIdentifier: string): void {
    const repoDiffs = currentRepositoryDiffTabs.value
    const targetIndex = repoDiffs.findIndex(tab => tab.id === tabIdentifier)
    if (targetIndex === -1) return

    const isClosingActive = activeDiffTabId.value === tabIdentifier

    diffTabs.value = diffTabs.value.filter(tab => tab.id !== tabIdentifier)

    if (isClosingActive) {
      const remainingRepoDiffs = diffTabs.value.filter(tab => tab.repositoryPath === activeRepositoryPath.value)
      if (remainingRepoDiffs.length > 0) {
        const nextActiveIndex = Math.max(0, targetIndex - 1)
        activeDiffTabId.value = remainingRepoDiffs[nextActiveIndex]?.id || null
      } else {
        activeDiffTabId.value = null
      }
    }
  }

  function closeAllDiffTabs(): void {
    diffTabs.value = diffTabs.value.filter(tab => tab.repositoryPath !== activeRepositoryPath.value)
    activeDiffTabId.value = null
  }

  function closeOtherDiffTabs(tabIdentifier: string): void {
    diffTabs.value = diffTabs.value.filter(
      tab => tab.repositoryPath !== activeRepositoryPath.value || tab.id === tabIdentifier,
    )
    activeDiffTabId.value = tabIdentifier
  }

  function activateDiffTab(tabIdentifier: string | null): void {
    activeDiffTabId.value = tabIdentifier
    if (tabIdentifier) {
      activePullRequestTabId.value = null
    }
  }

  function openPullRequestTab(
    pullRequest: RemotePullRequest,
    account: ProviderAccount,
    repositoryFullName: string,
    repositoryPath: string,
  ): WorkspacePullRequestTab {
    const tabIdentifier = `pr:${repositoryPath}:${repositoryFullName}:${pullRequest.number}`
    const existingIndex = pullRequestTabs.value.findIndex(tab => tab.id === tabIdentifier)

    if (existingIndex !== -1) {
      const existingTab = pullRequestTabs.value[existingIndex] as WorkspacePullRequestTab
      existingTab.pullRequest = pullRequest
      existingTab.account = account
      existingTab.repositoryFullName = repositoryFullName
      activePullRequestTabId.value = tabIdentifier
      activeDiffTabId.value = null
      return existingTab
    }

    const newTab: WorkspacePullRequestTab = {
      id: tabIdentifier,
      title: `#${pullRequest.number} ${pullRequest.title}`,
      pullRequest,
      account,
      repositoryFullName,
      repositoryPath,
      isClosable: true,
      isPinned: false,
      activeSubView: 'conversation',
    }

    pullRequestTabs.value.push(newTab)
    activePullRequestTabId.value = newTab.id
    activeDiffTabId.value = null
    return newTab
  }

  function activatePullRequestTab(tabIdentifier: string | null): void {
    activePullRequestTabId.value = tabIdentifier
    if (tabIdentifier) {
      activeDiffTabId.value = null
      activeWorkflowRunTabId.value = null
    }
  }

  function closePullRequestTab(tabIdentifier: string): void {
    const prTabs = currentRepositoryPullRequestTabs.value
    const targetIndex = prTabs.findIndex(tab => tab.id === tabIdentifier)
    if (targetIndex === -1) return

    const isClosingActive = activePullRequestTabId.value === tabIdentifier
    pullRequestTabs.value = pullRequestTabs.value.filter(tab => tab.id !== tabIdentifier)

    if (isClosingActive) {
      const remainingTabs = pullRequestTabs.value.filter(tab => tab.repositoryPath === activeRepositoryPath.value)
      if (remainingTabs.length > 0) {
        const nextActiveIndex = Math.max(0, targetIndex - 1)
        activePullRequestTabId.value = remainingTabs[nextActiveIndex]?.id || null
      } else {
        activePullRequestTabId.value = null
      }
    }
  }

  function openWorkflowRunTab(
    workflowRun: WorkflowRun,
    account: ProviderAccount,
    repositoryFullName: string,
    targetRepositoryPath?: string,
  ): WorkspaceWorkflowRunTab {
    const repositoryPath = targetRepositoryPath || activeRepositoryPath.value
    const tabIdentifier = `workflow-run-${repositoryPath}-${workflowRun.id}`

    const existingIndex = workflowRunTabs.value.findIndex(tab => tab.id === tabIdentifier)
    if (existingIndex !== -1) {
      const existingTab = workflowRunTabs.value[existingIndex] as WorkspaceWorkflowRunTab
      existingTab.workflowRun = workflowRun
      existingTab.account = account
      existingTab.repositoryFullName = repositoryFullName
      activeWorkflowRunTabId.value = tabIdentifier
      activeDiffTabId.value = null
      activePullRequestTabId.value = null
      return existingTab
    }

    const newTab: WorkspaceWorkflowRunTab = {
      id: tabIdentifier,
      title: `${workflowRun.workflow_name} #${workflowRun.run_number}`,
      workflowRun,
      account,
      repositoryFullName,
      repositoryPath,
      isClosable: true,
      isPinned: false,
    }

    workflowRunTabs.value.push(newTab)
    activeWorkflowRunTabId.value = newTab.id
    activeDiffTabId.value = null
    activePullRequestTabId.value = null
    return newTab
  }

  function activateWorkflowRunTab(tabIdentifier: string | null): void {
    activeWorkflowRunTabId.value = tabIdentifier
    if (tabIdentifier) {
      activeDiffTabId.value = null
      activePullRequestTabId.value = null
    }
  }

  function closeWorkflowRunTab(tabIdentifier: string): void {
    const runTabs = currentRepositoryWorkflowRunTabs.value
    const targetIndex = runTabs.findIndex(tab => tab.id === tabIdentifier)
    if (targetIndex === -1) return

    const isClosingActive = activeWorkflowRunTabId.value === tabIdentifier
    workflowRunTabs.value = workflowRunTabs.value.filter(tab => tab.id !== tabIdentifier)

    if (isClosingActive) {
      const remainingTabs = workflowRunTabs.value.filter(tab => tab.repositoryPath === activeRepositoryPath.value)
      if (remainingTabs.length > 0) {
        const nextActiveIndex = Math.max(0, targetIndex - 1)
        activeWorkflowRunTabId.value = remainingTabs[nextActiveIndex]?.id || null
      } else {
        activeWorkflowRunTabId.value = null
      }
    }
  }

  function activateRepositoryTab(repositoryPath: string): void {
    activeRepositoryPath.value = repositoryPath
    activeDiffTabId.value = null
    activePullRequestTabId.value = null
    activeWorkflowRunTabId.value = null
    persistTabsToStorage()
  }

  function resetTabs(): void {
    repositoryTabs.value = []
    diffTabs.value = []
    pullRequestTabs.value = []
    workflowRunTabs.value = []
    activeRepositoryPath.value = ''
    activeDiffTabId.value = null
    activePullRequestTabId.value = null
    activeWorkflowRunTabId.value = null
    persistTabsToStorage()
  }

  return {
    repositoryTabs,
    activeRepositoryPath,
    activeRepositoryTab,
    diffTabs,
    activeDiffTabId,
    activeDiffTab,
    currentRepositoryDiffTabs,
    pullRequestTabs,
    activePullRequestTabId,
    activePullRequestTab,
    currentRepositoryPullRequestTabs,
    workflowRunTabs,
    activeWorkflowRunTabId,
    activeWorkflowRunTab,
    currentRepositoryWorkflowRunTabs,
    openRepositoryTab,
    closeRepositoryTab,
    closeOtherRepositoryTabs,
    activateRepositoryTab,
    openDiffTab,
    pinDiffTab,
    closeDiffTab,
    closeAllDiffTabs,
    closeOtherDiffTabs,
    activateDiffTab,
    openPullRequestTab,
    activatePullRequestTab,
    closePullRequestTab,
    openWorkflowRunTab,
    activateWorkflowRunTab,
    closeWorkflowRunTab,
    resetTabs,
  }
}
