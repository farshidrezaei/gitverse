<script setup lang="ts">
import type { SplitterItem } from '@nuxt/ui'
import type { BranchSummary, RemoteBranchSummary, FileChangeStatus } from '~/types/repository'
import type { WorkspaceDiffTab } from '~/types/workspaceTab'
import WorkspaceTabBar from '~/components/workspace/tabs/WorkspaceTabBar.vue'
import WorkspaceDiffTabContent from '~/components/workspace/tabs/WorkspaceDiffTabContent.vue'
import WorkspaceBranchSidebar from '~/components/workspace/branch/WorkspaceBranchSidebar.vue'
import WorkspaceStagingSidebar from '~/components/workspace/staging/WorkspaceStagingSidebar.vue'
import WorkspaceCommitGraphViewport from '~/components/workspace/CommitGraphViewport.vue'
import WorkspaceDiffViewerPanel from '~/components/workspace/DiffViewerPanel.vue'
import WorkspaceCommitInspectorPanel from '~/components/workspace/CommitInspectorPanel.vue'
import WelcomeWorkspaceState from '~/components/workspace/WelcomeWorkspaceState.vue'
import CreateBranchModal from '~/components/workspace/CreateBranchModal.vue'
import { useWorkspaceTabController } from '~/composables/useWorkspaceTabController'

const { t } = useI18n()
const { confirmStandard, confirmHold, confirmKeyword } = useConfirmationDialog()

const {
  currentRepository,
  repositoryStatus,
  repositoryReferences,
  selectedDiffTarget,
  currentDiff,
  commitGraph,
  selectedCommitHash,
  selectedCommitDetails,
  selectedCommitFilePath,
  activeComparison,
  isComparing,
  favoriteBranches,
  stagingViewMode,
  isLoading,
  isLoadingDiff,
  isLoadingGraph,
  isLoadingCommitDetails,
  isOperating,
  fetchStatus,
  fetchReferences,
  fetchCommitGraph,
  fetchFileDiff,
  fetchCommitDetails,
  fetchCommitFileDiff,
  compareCommits,
  fetchComparisonFileDiff,
  swapComparison,
  clearComparison,
  clearCommitSelection,
  openRepository,
  closeRepository,
  promptAndOpenRepository,
  toggleFavoriteBranch,
  loadFavoriteBranches,
  restoreLastOpenedRepository,
  checkoutBranch,
  createBranch,
  deleteBranch,
  fetchRemote,
  pullChanges,
  pushChanges,
  stageFile,
  unstageFile,
  stageAll,
  unstageAll,
  discardFile,
  discardAllUnstaged,
  commitChanges,
  fetchStashes,
  cherryPickCommit,
  revertCommit,
  resetToCommit,
} = useWorkspaceManager()

const {
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
} = useWorkspaceTabController()

const isCreateBranchModalOpen = ref<boolean>(false)
const commitMessage = ref<string>('')
const isAmend = ref<boolean>(false)
const selectedBranchName = ref<string | null>(null)
const activeGraphReference = ref<string | null>(null)

watch(
  currentRepository,
  (repository) => {
    if (repository) {
      openWorkspaceTab(repository.path, repository.name)
    } else {
      resetTabs()
    }
  },
  { immediate: true },
)

async function handleSelectTab(tabId: string): Promise<void> {
  const targetTab = tabs.value.find(tab => tab.id === tabId)
  if (!targetTab) return

  activateTab(tabId)

  if (targetTab.type === 'workspace') {
    const mainTab = targetTab as WorkspaceMainTab
    if (currentRepository.value?.path !== mainTab.repositoryPath) {
      await openRepository(mainTab.repositoryPath)
    }
  } else if (targetTab.type === 'diff') {
    const diffTab = targetTab as WorkspaceDiffTab
    if (diffTab.repositoryPath && currentRepository.value?.path !== diffTab.repositoryPath) {
      await openRepository(diffTab.repositoryPath)
    }
  }
}

async function handleCloseTab(tabId: string): Promise<void> {
  const targetTab = tabs.value.find(tab => tab.id === tabId)
  closeTab(tabId)

  // If no workspace tabs left, close active repository
  const remainingWorkspaceTabs = tabs.value.filter(tab => tab.type === 'workspace')
  if (remainingWorkspaceTabs.length === 0) {
    await closeRepository()
  } else if (activeTab.value?.type === 'workspace') {
    const activeMainTab = activeTab.value as WorkspaceMainTab
    if (currentRepository.value?.path !== activeMainTab.repositoryPath) {
      await openRepository(activeMainTab.repositoryPath)
    }
  }
}

function handleKeyboardShortcuts(event: KeyboardEvent): void {
  const isModifierKeyActive = event.ctrlKey || event.metaKey
  if (isModifierKeyActive && event.key.toLowerCase() === 'w') {
    if (activeTab.value?.isClosable) {
      event.preventDefault()
      closeTab(activeTabId.value)
    }
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeyboardShortcuts)

  if (currentRepository.value) {
    loadFavoriteBranches()
    await fetchStatus()
    await fetchReferences()
    await fetchCommitGraph()
    await fetchStashes()
  } else {
    await restoreLastOpenedRepository()
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyboardShortcuts)
})

// Main horizontal layout with floating inset panels (Island-style)
const mainSplitterItems = ref<SplitterItem[]>([
  {
    id: 'sidebar-island',
    slot: 'sidebar',
    minSize: 14,
    defaultSize: 19,
    maxSize: 32,
    collapsible: true,
    collapsedSize: 0,
    class: 'p-1.5 overflow-hidden flex flex-col',
  },
  {
    id: 'center-island',
    slot: 'center',
    minSize: 35,
    defaultSize: 53,
    class: 'p-1.5 overflow-hidden flex flex-col',
  },
  {
    id: 'staging-island',
    slot: 'staging',
    minSize: 18,
    defaultSize: 28,
    maxSize: 45,
    collapsible: true,
    collapsedSize: 0,
    class: 'p-1.5 overflow-hidden flex flex-col',
  },
])

// Nested vertical layout inside center panel: Commit Graph (top) | Diff Viewer (bottom)
const centerSplitterItems = ref<SplitterItem[]>([
  {
    id: 'graph-top-panel',
    slot: 'graph',
    minSize: 30,
    defaultSize: 55,
    class: 'pb-1 overflow-hidden flex flex-col',
  },
  {
    id: 'diff-bottom-panel',
    slot: 'diff',
    minSize: 20,
    defaultSize: 45,
    class: 'pt-1 overflow-hidden flex flex-col',
  },
])

async function selectCommit(sha: string): Promise<void> {
  await fetchCommitDetails(sha)
}

function handleSelectCommitFile(filePath: string): void {
  if (activeComparison.value) {
    void fetchComparisonFileDiff(
      activeComparison.value.base_hash,
      activeComparison.value.target_hash,
      filePath,
    )
  } else if (selectedCommitHash.value) {
    void fetchCommitFileDiff(selectedCommitHash.value, filePath)
  }
}

function handleCreateBranchFromCommit(commitSha: string): void {
  selectedCommitHash.value = commitSha
  isCreateBranchModalOpen.value = true
}

function handleCreateBranchFromBranch(): void {
  isCreateBranchModalOpen.value = true
}

async function handleCheckoutCommit(commitSha: string): Promise<void> {
  const isConfirmed = await confirmStandard({
    title: t('git.checkoutDetachedTitle'),
    description: t('git.checkoutDetachedDescription'),
    variant: 'warning',
    confirmLabel: t('git.checkoutCommit'),
    details: [
      { label: t('git.commit'), value: commitSha.substring(0, 8), isMonospace: true },
    ],
  })

  if (isConfirmed) {
    await checkoutBranch(commitSha)
  }
}

async function handleDiscard(filePath: string, isUntracked: boolean = false): Promise<void> {
  const isConfirmed = await confirmStandard({
    title: t('git.discardFileTitle'),
    description: t('git.discardFileDescription'),
    variant: 'danger',
    confirmLabel: t('git.discard'),
    details: [
      { label: t('git.file'), value: filePath, isMonospace: true },
    ],
  })

  if (isConfirmed) {
    await discardFile(filePath, isUntracked)
  }
}

async function handleDiscardAllUnstaged(): Promise<void> {
  const unstagedCount = repositoryStatus.value?.unstaged_files?.length || 0
  if (unstagedCount === 0) return

  const isConfirmed = await confirmHold({
    title: t('git.discardAllTitle'),
    description: t('git.discardAllDescription'),
    variant: 'danger',
    confirmLabel: t('git.discardAll'),
    holdDurationMilliseconds: 3000,
    details: [
      { label: t('git.affectedFiles'), value: `${unstagedCount} files` },
    ],
  })

  if (isConfirmed) {
    await discardAllUnstaged()
  }
}

function handleSelectFile(filePath: string, isStaged: boolean): void {
  clearCommitSelection()
  void fetchFileDiff(filePath, isStaged)
}

function handleOpenStagingDiffTab(
  filePath: string,
  isStaged: boolean,
  changeStatus?: FileChangeStatus,
): void {
  openDiffTab({
    filePath,
    repositoryPath: currentRepository.value?.path,
    isStaged,
    changeStatus,
    isPinned: true,
  })
}

function handleOpenCommitDiffTab(filePath: string): void {
  if (activeComparison.value) {
    openDiffTab({
      filePath,
      repositoryPath: currentRepository.value?.path,
      comparisonBaseHash: activeComparison.value.base_hash,
      comparisonTargetHash: activeComparison.value.target_hash,
      isPinned: true,
    })
  } else if (selectedCommitHash.value) {
    openDiffTab({
      filePath,
      repositoryPath: currentRepository.value?.path,
      commitHash: selectedCommitHash.value,
      isPinned: true,
    })
  }
}

async function handleCommit(): Promise<void> {
  if (!commitMessage.value.trim()) return
  const success = await commitChanges(commitMessage.value, isAmend.value)
  if (success) {
    commitMessage.value = ''
    isAmend.value = false
  }
}

async function handleCreateBranch(branchName: string, checkout: boolean): Promise<void> {
  const success = checkout
    ? await checkoutBranch(branchName, true)
    : await createBranch(branchName)
  if (success) {
    isCreateBranchModalOpen.value = false
  }
}

async function handleSelectBranch(branch: BranchSummary | RemoteBranchSummary): Promise<void> {
  selectedBranchName.value = branch.name
  activeGraphReference.value = branch.name
  await fetchCommitGraph(0, 100, branch.name)
  if (branch.head_commit_hash) {
    await fetchCommitDetails(branch.head_commit_hash)
  }
}

async function handleResetGraphFilter(): Promise<void> {
  activeGraphReference.value = null
  selectedBranchName.value = null
  await fetchCommitGraph(0, 100)
}

async function handleRefreshGraph(): Promise<void> {
  await fetchCommitGraph(0, 100, activeGraphReference.value || undefined)
}

async function handleCheckout(branchName: string): Promise<void> {
  activeGraphReference.value = null
  selectedBranchName.value = branchName
  await checkoutBranch(branchName)
}

async function handleDeleteBranch(branchName: string): Promise<void> {
  const isConfirmed = await confirmKeyword({
    title: t('git.deleteBranchTitle', { name: branchName }),
    description: t('git.deleteBranchDescription'),
    variant: 'danger',
    confirmLabel: t('git.deleteBranch'),
    confirmationKeyword: branchName,
    details: [
      { label: t('git.branch'), value: branchName, isMonospace: true },
    ],
  })

  if (isConfirmed) {
    await deleteBranch(branchName)
  }
}

async function handleFetch(): Promise<void> {
  await fetchRemote()
}

async function handlePull(): Promise<void> {
  await pullChanges()
}

async function handlePush(): Promise<void> {
  await pushChanges(undefined, undefined, true)
}
</script>

<template>
  <div class="h-full w-full flex-1 overflow-hidden select-none bg-[var(--gv-background)]">
    <!-- Zero Repository Open State: Show Welcome screen with recent repos -->
    <WelcomeWorkspaceState v-if="!currentRepository" />

    <!-- Active Repository: Tab Bar + Content Area -->
    <div v-else class="flex h-full w-full flex-col overflow-hidden">
      <!-- Workspace Tab Strip -->
      <WorkspaceTabBar
        :tabs="tabs"
        :active-tab-id="activeTabId"
        @select-tab="handleSelectTab"
        @close-tab="handleCloseTab"
        @close-other-tabs="closeOtherTabs"
        @close-all-diff-tabs="closeAllDiffTabs"
        @pin-tab="pinTab"
        @open-new-repository="promptAndOpenRepository"
      />

      <!-- Content Area -->
      <div class="relative flex-1 overflow-hidden">
        <!-- Main Workspace Workbench Island (Preserved with v-show for instantaneous tab switching) -->
        <div
          v-show="activeTab?.type === 'workspace'"
          class="h-full w-full"
        >
          <USplitter
            id="gitverse-workbench-splitter"
            auto-save-id="gitverse-workbench-layout"
            :items="mainSplitterItems"
            class="h-full w-full"
            :ui="{
              handle: 'w-1.5 bg-transparent hover:bg-primary/50 transition-colors data-[state=drag]:bg-primary cursor-col-resize',
            }"
          >
            <!-- PANEL 1: INSET SIDEBAR ISLAND (Branches, Tags, Remotes) -->
            <template #sidebar>
              <WorkspaceBranchSidebar
                :references="repositoryReferences"
                :favorite-branches="favoriteBranches"
                :selected-branch-name="selectedBranchName"
                :is-loading="isLoading"
                @select-branch="handleSelectBranch"
                @checkout="handleCheckout"
                @create-branch-from-branch="handleCreateBranchFromBranch"
                @delete-branch="handleDeleteBranch"
                @open-create-branch-modal="isCreateBranchModalOpen = true"
                @toggle-favorite-branch="toggleFavoriteBranch"
              />
            </template>

            <!-- PANEL 2: INSET CENTER ISLANDS (Commit Graph + Diff Viewer) -->
            <template #center>
              <USplitter
                id="center-vertical-splitter"
                orientation="vertical"
                auto-save-id="gitverse-center-vertical-layout"
                :items="centerSplitterItems"
                class="h-full w-full"
                :ui="{
                  handle: 'h-1.5 bg-transparent hover:bg-primary/50 transition-colors data-[state=drag]:bg-primary cursor-row-resize',
                }"
              >
                <!-- Top Island: Commit Graph & History -->
                <template #graph>
                  <WorkspaceCommitGraphViewport
                    :graph="commitGraph"
                    :selected-commit-hash="selectedCommitHash"
                    :comparison-base-hash="activeComparison?.base_hash"
                    :is-loading="isLoadingGraph"
                    :is-operating="isOperating"
                    :active-reference="activeGraphReference"
                    @select-commit="selectCommit"
                    @compare-commits="(base, target) => compareCommits(base, target)"
                    @refresh="handleRefreshGraph"
                    @reset-graph-filter="handleResetGraphFilter"
                    @push="handlePush"
                    @pull="handlePull"
                    @fetch="handleFetch"
                    @create-branch="isCreateBranchModalOpen = true"
                    @create-branch-at-commit="handleCreateBranchFromCommit"
                    @checkout-commit="handleCheckoutCommit"
                    @cherry-pick-commit="cherryPickCommit"
                    @revert-commit="revertCommit"
                    @reset-to-commit="resetToCommit"
                  />
                </template>

                <!-- Bottom Island: Diff & Patch Viewer -->
                <template #diff>
                  <WorkspaceDiffViewerPanel
                    :diff="currentDiff"
                    :selected-target="selectedDiffTarget"
                    :is-loading="isLoadingDiff"
                    @stage="stageFile"
                    @unstage="unstageFile"
                    @discard="handleDiscard"
                  />
                </template>
              </USplitter>
            </template>

            <!-- PANEL 3: INSET STAGING ISLAND & COMMIT INSPECTOR -->
            <template #staging>
              <!-- Commit Inspector Island -->
              <WorkspaceCommitInspectorPanel
                v-if="selectedCommitDetails || activeComparison || isLoadingCommitDetails"
                :commit-details="selectedCommitDetails"
                :active-comparison="activeComparison"
                :selected-file-path="selectedCommitFilePath"
                :is-loading="isLoadingCommitDetails"
                @select-file="handleSelectCommitFile"
                @open-diff-tab="handleOpenCommitDiffTab"
                @create-branch="handleCreateBranchFromCommit"
                @checkout-commit="handleCheckoutCommit"
                @cherry-pick="cherryPickCommit"
                @revert="revertCommit"
                @reset="resetToCommit"
                @close="clearCommitSelection"
                @swap-comparison="swapComparison"
                @close-comparison="clearComparison"
              />

              <!-- Working Tree Changes Island -->
              <WorkspaceStagingSidebar
                v-else
                :repository-status="repositoryStatus"
                :selected-diff-target="selectedDiffTarget"
                :staging-view-mode="stagingViewMode"
                :is-loading="isLoading"
                :commit-message="commitMessage"
                :is-amend="isAmend"
                @update:staging-view-mode="stagingViewMode = $event"
                @update:commit-message="commitMessage = $event"
                @update:is-amend="isAmend = $event"
                @stage-file="stageFile"
                @unstage-file="unstageFile"
                @stage-all="stageAll"
                @unstage-all="unstageAll"
                @discard-file="handleDiscard"
                @discard-all-unstaged="handleDiscardAllUnstaged"
                @select-file="handleSelectFile"
                @open-diff-tab="handleOpenStagingDiffTab"
                @commit="handleCommit"
                @refresh="fetchStatus"
              />
            </template>
          </USplitter>
        </div>

        <!-- Full-Page Dedicated Diff Tab -->
        <WorkspaceDiffTabContent
          v-if="activeTab?.type === 'diff'"
          :tab="(activeTab as WorkspaceDiffTab)"
          @stage="stageFile"
          @unstage="unstageFile"
          @discard="handleDiscard"
          @close="closeTab"
        />
      </div>
    </div>

    <!-- Branch creation modal -->
    <CreateBranchModal
      v-model:open="isCreateBranchModalOpen"
      :source-branch="currentRepository?.current_branch_name"
      :source-commit-sha="selectedCommitHash"
      :existing-branch-names="repositoryReferences?.branches.map((b) => b.name) || []"
      @create="handleCreateBranch"
    />
  </div>
</template>
