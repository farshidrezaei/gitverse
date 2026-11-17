import { describe, expect, it } from 'vitest'
import { useWorkspaceTabController } from '~/composables/useWorkspaceTabController'

describe('useWorkspaceTabController', () => {
  it('initializes workspace main tab properly', () => {
    const {
      tabs,
      activeTabId,
      activeTab,
      initializeWorkspaceTab,
    } = useWorkspaceTabController()

    initializeWorkspaceTab('/path/to/project', 'project')

    expect(tabs.value.length).toBe(1)
    expect(tabs.value[0]?.id).toBe('workspace-main')
    expect(tabs.value[0]?.type).toBe('workspace')
    expect(tabs.value[0]?.isClosable).toBe(false)
    expect(activeTabId.value).toBe('workspace-main')
    expect(activeTab.value?.id).toBe('workspace-main')
  })

  it('opens a new diff tab and activates it', () => {
    const {
      tabs,
      activeTabId,
      openDiffTab,
      initializeWorkspaceTab,
    } = useWorkspaceTabController()

    initializeWorkspaceTab('/path/to/project', 'project')

    const newTab = openDiffTab({
      filePath: 'src/main.rs',
      isStaged: true,
      isPinned: true,
    })

    expect(tabs.value.length).toBe(2)
    expect(newTab.id).toBe('diff:staged:src/main.rs')
    expect(newTab.type).toBe('diff')
    expect(newTab.title).toBe('main.rs')
    expect(newTab.isClosable).toBe(true)
    expect(newTab.isPinned).toBe(true)
    expect(activeTabId.value).toBe(newTab.id)
  })

  it('reuses unpinned preview diff tab when opening another preview file', () => {
    const {
      tabs,
      activeTabId,
      openDiffTab,
      initializeWorkspaceTab,
    } = useWorkspaceTabController()

    initializeWorkspaceTab('/path/to/project', 'project')

    // Open first preview tab (unpinned)
    openDiffTab({
      filePath: 'src/first.rs',
      isPinned: false,
    })

    expect(tabs.value.length).toBe(2)
    expect(tabs.value[1]?.title).toBe('first.rs')
    expect(tabs.value[1]?.isPinned).toBe(false)

    // Open second preview tab (unpinned) -> should replace first
    openDiffTab({
      filePath: 'src/second.rs',
      isPinned: false,
    })

    expect(tabs.value.length).toBe(2)
    expect(tabs.value[1]?.title).toBe('second.rs')
    expect(tabs.value[1]?.isPinned).toBe(false)
    expect(activeTabId.value).toBe('diff:working:src/second.rs')
  })

  it('pins an unpinned tab', () => {
    const {
      tabs,
      openDiffTab,
      pinTab,
      initializeWorkspaceTab,
    } = useWorkspaceTabController()

    initializeWorkspaceTab('/path/to/project', 'project')

    const tab = openDiffTab({
      filePath: 'src/preview.rs',
      isPinned: false,
    })

    expect(tab.isPinned).toBe(false)
    pinTab(tab.id)
    expect(tabs.value.find(item => item.id === tab.id)?.isPinned).toBe(true)
  })

  it('closes a diff tab and activates the preceding tab', () => {
    const {
      tabs,
      activeTabId,
      openDiffTab,
      closeTab,
      initializeWorkspaceTab,
    } = useWorkspaceTabController()

    initializeWorkspaceTab('/path/to/project', 'project')

    const tabA = openDiffTab({ filePath: 'fileA.txt', isPinned: true })
    const tabB = openDiffTab({ filePath: 'fileB.txt', isPinned: true })

    expect(tabs.value.length).toBe(3)
    expect(activeTabId.value).toBe(tabB.id)

    // Close tabB -> should fallback to tabA
    closeTab(tabB.id)
    expect(tabs.value.length).toBe(2)
    expect(activeTabId.value).toBe(tabA.id)

    // Close tabA -> should fallback to workspace-main
    closeTab(tabA.id)
    expect(tabs.value.length).toBe(1)
    expect(activeTabId.value).toBe('workspace-main')

    // Attempt to close non-closable main tab -> should not close
    closeTab('workspace-main')
    expect(tabs.value.length).toBe(1)
  })

  it('closes other tabs and closes all diff tabs', () => {
    const {
      tabs,
      activeTabId,
      openDiffTab,
      closeOtherTabs,
      closeAllDiffTabs,
      initializeWorkspaceTab,
    } = useWorkspaceTabController()

    initializeWorkspaceTab('/path/to/project', 'project')

    openDiffTab({ filePath: 'file1.txt', isPinned: true })
    const tab2 = openDiffTab({ filePath: 'file2.txt', isPinned: true })
    openDiffTab({ filePath: 'file3.txt', isPinned: true })

    expect(tabs.value.length).toBe(4)

    // Close others with respect to tab2 -> keep main and tab2
    closeOtherTabs(tab2.id)
    expect(tabs.value.length).toBe(2)
    expect(tabs.value.map(item => item.id)).toEqual(['workspace-main', tab2.id])
    expect(activeTabId.value).toBe(tab2.id)

    // Close all diff tabs -> only main remains
    closeAllDiffTabs()
    expect(tabs.value.length).toBe(1)
    expect(tabs.value[0]?.id).toBe('workspace-main')
    expect(activeTabId.value).toBe('workspace-main')
  })
})
