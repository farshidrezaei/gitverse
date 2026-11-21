import { describe, expect, it } from 'vitest'
import { useWorkspaceModals } from '~/composables/useWorkspaceModals'

describe('useWorkspaceModals Composable', () => {
  it('initializes with all modals closed', () => {
    const modals = useWorkspaceModals()
    expect(modals.isReflogModalOpen.value).toBe(false)
    expect(modals.isBisectModalOpen.value).toBe(false)
    expect(modals.isInsightsModalOpen.value).toBe(false)
    expect(modals.isSubmoduleModalOpen.value).toBe(false)
    expect(modals.isExportPatchModalOpen.value).toBe(false)
    expect(modals.isApplyPatchModalOpen.value).toBe(false)
    expect(modals.isGitBundleModalOpen.value).toBe(false)
    expect(modals.isFolderPickerModalOpen.value).toBe(false)
  })

  it('opens modals via designated trigger methods', () => {
    const modals = useWorkspaceModals()

    modals.openReflog()
    expect(modals.isReflogModalOpen.value).toBe(true)

    modals.openBisect()
    expect(modals.isBisectModalOpen.value).toBe(true)

    modals.openInsights()
    expect(modals.isInsightsModalOpen.value).toBe(true)

    modals.openSubmodules()
    expect(modals.isSubmoduleModalOpen.value).toBe(true)

    modals.openExportPatch()
    expect(modals.isExportPatchModalOpen.value).toBe(true)

    modals.openApplyPatch()
    expect(modals.isApplyPatchModalOpen.value).toBe(true)

    modals.openGitBundle()
    expect(modals.isGitBundleModalOpen.value).toBe(true)

    modals.openFolderPicker()
    expect(modals.isFolderPickerModalOpen.value).toBe(true)
  })
})
