import { describe, expect, it } from 'vitest'
import { useWorkspaceTabController } from '~/composables/useWorkspaceTabController'

describe('useWorkspaceTabController', () => {
  it('initializes and manages repository tabs independently', () => {
    const {
      repositoryTabs,
      activeRepositoryPath,
      activeRepositoryTab,
      openRepositoryTab,
    } = useWorkspaceTabController()

    openRepositoryTab('/path/to/project1', 'project1')
    expect(repositoryTabs.value.length).toBe(1)
    expect(repositoryTabs.value[0]?.repositoryPath).toBe('/path/to/project1')
    expect(activeRepositoryPath.value).toBe('/path/to/project1')
    expect(activeRepositoryTab.value?.repositoryName).toBe('project1')

    openRepositoryTab('/path/to/project2', 'project2')
    expect(repositoryTabs.value.length).toBe(2)
    expect(activeRepositoryPath.value).toBe('/path/to/project2')
    expect(activeRepositoryTab.value?.repositoryName).toBe('project2')
  })

  it('opens a diff tab scoped to active repository', () => {
    const {
      openRepositoryTab,
      openDiffTab,
      currentRepositoryDiffTabs,
      activeDiffTabId,
      activeDiffTab,
    } = useWorkspaceTabController()

    openRepositoryTab('/path/to/project1', 'project1')

    const newTab = openDiffTab({
      filePath: 'src/main.rs',
      isStaged: true,
      isPinned: true,
    })

    expect(currentRepositoryDiffTabs.value.length).toBe(1)
    expect(newTab.title).toBe('main.rs')
    expect(newTab.repositoryPath).toBe('/path/to/project1')
    expect(activeDiffTabId.value).toBe(newTab.id)
    expect(activeDiffTab.value?.id).toBe(newTab.id)
  })

  it('reuses unpinned preview diff tab within the same repository', () => {
    const {
      openRepositoryTab,
      openDiffTab,
      currentRepositoryDiffTabs,
      activeDiffTabId,
    } = useWorkspaceTabController()

    openRepositoryTab('/path/to/project1', 'project1')

    openDiffTab({
      filePath: 'src/first.rs',
      isPinned: false,
    })

    expect(currentRepositoryDiffTabs.value.length).toBe(1)
    expect(currentRepositoryDiffTabs.value[0]?.title).toBe('first.rs')
    expect(currentRepositoryDiffTabs.value[0]?.isPinned).toBe(false)

    openDiffTab({
      filePath: 'src/second.rs',
      isPinned: false,
    })

    expect(currentRepositoryDiffTabs.value.length).toBe(1)
    expect(currentRepositoryDiffTabs.value[0]?.title).toBe('second.rs')
    expect(currentRepositoryDiffTabs.value[0]?.isPinned).toBe(false)
    expect(activeDiffTabId.value).toContain('second.rs')
  })

  it('pins an unpinned diff tab', () => {
    const {
      openRepositoryTab,
      openDiffTab,
      pinDiffTab,
      currentRepositoryDiffTabs,
    } = useWorkspaceTabController()

    openRepositoryTab('/path/to/project1', 'project1')

    const tab = openDiffTab({
      filePath: 'src/preview.rs',
      isPinned: false,
    })

    expect(tab.isPinned).toBe(false)
    pinDiffTab(tab.id)
    expect(currentRepositoryDiffTabs.value.find(item => item.id === tab.id)?.isPinned).toBe(true)
  })

  it('closes a diff tab and activates preceding diff tab or workbench', () => {
    const {
      openRepositoryTab,
      openDiffTab,
      closeDiffTab,
      currentRepositoryDiffTabs,
      activeDiffTabId,
    } = useWorkspaceTabController()

    openRepositoryTab('/path/to/project1', 'project1')

    const tabA = openDiffTab({ filePath: 'fileA.txt', isPinned: true })
    const tabB = openDiffTab({ filePath: 'fileB.txt', isPinned: true })

    expect(currentRepositoryDiffTabs.value.length).toBe(2)
    expect(activeDiffTabId.value).toBe(tabB.id)

    // Close tabB -> fallback to tabA
    closeDiffTab(tabB.id)
    expect(currentRepositoryDiffTabs.value.length).toBe(1)
    expect(activeDiffTabId.value).toBe(tabA.id)

    // Close tabA -> fallback to null (workbench view)
    closeDiffTab(tabA.id)
    expect(currentRepositoryDiffTabs.value.length).toBe(0)
    expect(activeDiffTabId.value).toBeNull()
  })

  it('closes other diff tabs and all diff tabs', () => {
    const {
      openRepositoryTab,
      openDiffTab,
      closeOtherDiffTabs,
      closeAllDiffTabs,
      currentRepositoryDiffTabs,
      activeDiffTabId,
    } = useWorkspaceTabController()

    openRepositoryTab('/path/to/project1', 'project1')

    openDiffTab({ filePath: 'file1.txt', isPinned: true })
    const tab2 = openDiffTab({ filePath: 'file2.txt', isPinned: true })
    openDiffTab({ filePath: 'file3.txt', isPinned: true })

    expect(currentRepositoryDiffTabs.value.length).toBe(3)

    closeOtherDiffTabs(tab2.id)
    expect(currentRepositoryDiffTabs.value.length).toBe(1)
    expect(currentRepositoryDiffTabs.value[0]?.id).toBe(tab2.id)
    expect(activeDiffTabId.value).toBe(tab2.id)

    closeAllDiffTabs()
    expect(currentRepositoryDiffTabs.value.length).toBe(0)
    expect(activeDiffTabId.value).toBeNull()
  })

  it('closes repository tab and removes associated diff tabs', () => {
    const {
      openRepositoryTab,
      openDiffTab,
      closeRepositoryTab,
      repositoryTabs,
      diffTabs,
      activeRepositoryPath,
    } = useWorkspaceTabController()

    openRepositoryTab('/path/to/repo1', 'repo1')
    openDiffTab({ filePath: 'repo1_file.rs', isPinned: true })

    openRepositoryTab('/path/to/repo2', 'repo2')
    openDiffTab({ filePath: 'repo2_file.rs', isPinned: true })

    expect(repositoryTabs.value.length).toBe(2)
    expect(diffTabs.value.length).toBe(2)
    expect(activeRepositoryPath.value).toBe('/path/to/repo2')

    closeRepositoryTab('/path/to/repo2')
    expect(repositoryTabs.value.length).toBe(1)
    expect(activeRepositoryPath.value).toBe('/path/to/repo1')
    expect(diffTabs.value.length).toBe(1)
    expect(diffTabs.value[0]?.filePath).toBe('repo1_file.rs')
  })

  it('persists tabs and active path to storage and clears when all tabs are closed', () => {
    const {
      openRepositoryTab,
      closeRepositoryTab,
      resetTabs,
    } = useWorkspaceTabController()

    window.localStorage.clear()

    openRepositoryTab('/path/to/project_alpha', 'project_alpha')
    expect(window.localStorage.getItem('gitverse_last_opened_repository_path')).toBe('/path/to/project_alpha')
    expect(window.localStorage.getItem('gitverse_open_repository_tabs')).toContain('project_alpha')

    openRepositoryTab('/path/to/project_beta', 'project_beta')
    expect(window.localStorage.getItem('gitverse_last_opened_repository_path')).toBe('/path/to/project_beta')
    expect(window.localStorage.getItem('gitverse_open_repository_tabs')).toContain('project_beta')

    closeRepositoryTab('/path/to/project_beta')
    expect(window.localStorage.getItem('gitverse_last_opened_repository_path')).toBe('/path/to/project_alpha')

    closeRepositoryTab('/path/to/project_alpha')
    expect(window.localStorage.getItem('gitverse_last_opened_repository_path')).toBeNull()
    expect(window.localStorage.getItem('gitverse_open_repository_tabs')).toBeNull()

    openRepositoryTab('/path/to/project_gamma', 'project_gamma')
    expect(window.localStorage.getItem('gitverse_last_opened_repository_path')).toBe('/path/to/project_gamma')
    resetTabs()
    expect(window.localStorage.getItem('gitverse_last_opened_repository_path')).toBeNull()
    expect(window.localStorage.getItem('gitverse_open_repository_tabs')).toBeNull()
  })
})
