<script setup lang="ts">
import type { RepositoryOverview, RepositoryReferences } from '~/types/repository'
import CreateBranchModal from '~/components/workspace/CreateBranchModal.vue'
import CreateWorktreeModal from '~/components/workspace/CreateWorktreeModal.vue'
import InteractiveRebaseModal from '~/components/workspace/commit/InteractiveRebaseModal.vue'
import FolderPickerModal from '~/components/workspace/FolderPickerModal.vue'
import ReflogExplorerModal from '~/components/workspace/history/ReflogExplorerModal.vue'
import BisectModal from '~/components/workspace/bisect/BisectModal.vue'
import ExportPatchModal from '~/components/workspace/patch/ExportPatchModal.vue'
import ApplyPatchModal from '~/components/workspace/patch/ApplyPatchModal.vue'
import SubmoduleManagerModal from '~/components/workspace/submodule/SubmoduleManagerModal.vue'
import RepositoryInsightsModal from '~/components/workspace/insights/RepositoryInsightsModal.vue'
import FileHistoryModal from '~/components/workspace/history/FileHistoryModal.vue'
import RepositorySearchModal from '~/components/workspace/search/RepositorySearchModal.vue'
import CommitQueueModal from '~/components/workspace/commit/CommitQueueModal.vue'
import GitBundleModal from '~/components/workspace/archive/GitBundleModal.vue'
import ExportArchiveModal from '~/components/workspace/archive/ExportArchiveModal.vue'
import LfsManagerModal from '~/components/workspace/lfs/LfsManagerModal.vue'
import CustomActionsModal from '~/components/workspace/actions/CustomActionsModal.vue'
import { useWorkspaceModals } from '~/composables/useWorkspaceModals'

interface WorkspaceModalsContainerProps {
  currentRepository: RepositoryOverview | null
  repositoryReferences: RepositoryReferences | null
  selectedCommitHash: string | null
  selectedFileHistoryPath?: string
  isCreateBranchModalOpen: boolean
  isCreateWorktreeModalOpen: boolean
  isInteractiveRebaseModalOpen: boolean
  interactiveRebaseBaseHash: string
}

const props = defineProps<WorkspaceModalsContainerProps>()

const emit = defineEmits<{
  'update:isCreateBranchModalOpen': [value: boolean]
  'update:isCreateWorktreeModalOpen': [value: boolean]
  'update:isInteractiveRebaseModalOpen': [value: boolean]
  'update:selectedCommitHash': [value: string | null]
  createBranch: [branchName: string, checkout: boolean]
  createWorktree: [path: string, branchName: string | undefined, isNewBranch: boolean]
  interactiveRebaseComplete: []
  openRepository: [path: string]
  checkoutBranch: [branchName: string]
  resetToCommit: [commitHash: string, mode: 'soft' | 'mixed' | 'hard']
  selectCommit: [commitHash: string]
  refreshState: []
}>()

const {
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
} = useWorkspaceModals()
</script>

<template>
  <div>
    <!-- Branch creation modal -->
    <CreateBranchModal
      :open="props.isCreateBranchModalOpen"
      :source-branch="props.currentRepository?.current_branch_name"
      :source-commit-sha="props.selectedCommitHash"
      :existing-branch-names="props.repositoryReferences?.branches.map((b) => b.name) || []"
      @update:open="emit('update:isCreateBranchModalOpen', $event)"
      @create="(name, checkout) => emit('createBranch', name, checkout)"
    />

    <!-- Worktree creation modal -->
    <CreateWorktreeModal
      :open="props.isCreateWorktreeModalOpen"
      :default-directory-path="props.currentRepository?.path"
      :existing-branch-names="props.repositoryReferences?.branches.map((b) => b.name) || []"
      @update:open="emit('update:isCreateWorktreeModalOpen', $event)"
      @create="(path, branch, isNew) => emit('createWorktree', path, branch, isNew)"
    />

    <!-- Interactive Rebase Modal -->
    <InteractiveRebaseModal
      :open="props.isInteractiveRebaseModalOpen"
      :base-commit-hash="props.interactiveRebaseBaseHash"
      @update:open="emit('update:isInteractiveRebaseModalOpen', $event)"
      @complete="emit('interactiveRebaseComplete')"
    />

    <!-- Custom Folder Picker Modal -->
    <FolderPickerModal
      v-model:open="isFolderPickerModalOpen"
      @select-folder="(path) => emit('openRepository', path)"
    />

    <!-- Git Reflog Explorer Modal -->
    <ReflogExplorerModal
      v-model:open="isReflogModalOpen"
      @checkout-commit="(sha) => emit('checkoutBranch', sha)"
      @create-branch="(sha) => { emit('update:selectedCommitHash', sha); emit('update:isCreateBranchModalOpen', true) }"
      @reset-to-commit="(sha, mode) => emit('resetToCommit', sha, mode)"
    />

    <!-- Interactive Git Bisect Assistant Modal -->
    <BisectModal
      v-model:open="isBisectModalOpen"
      @locate-commit="(sha) => emit('selectCommit', sha)"
    />

    <!-- Export Patch Modal -->
    <ExportPatchModal
      v-model:open="isExportPatchModalOpen"
    />

    <!-- Apply Patch Modal -->
    <ApplyPatchModal
      v-model:open="isApplyPatchModalOpen"
    />

    <!-- Git Submodules Manager Modal -->
    <SubmoduleManagerModal
      v-model:open="isSubmoduleModalOpen"
    />

    <!-- Repository Insights & Velocity Dashboard Modal -->
    <RepositoryInsightsModal
      v-model:open="isInsightsModalOpen"
    />

    <!-- File History & Revision Scrubber Modal -->
    <FileHistoryModal
      v-model:open="isFileHistoryModalOpen"
      :initial-file-path="props.selectedFileHistoryPath"
      @locate-commit="(sha) => emit('selectCommit', sha)"
    />

    <!-- Global Code & History Search Modal -->
    <RepositorySearchModal
      v-model:open="isSearchModalOpen"
      @locate-commit="(sha: string) => emit('selectCommit', sha)"
    />

    <!-- Multi-Commit Cherry-Pick & Revert Queue Modal -->
    <CommitQueueModal
      v-model:open="isCommitQueueModalOpen"
      :mode="commitQueueMode"
      @complete="emit('refreshState')"
    />

    <!-- Git Bundle Modal -->
    <GitBundleModal
      v-model:open="isGitBundleModalOpen"
    />

    <!-- Export Source Archive Modal -->
    <ExportArchiveModal
      v-model:open="isExportArchiveModalOpen"
    />

    <!-- Git LFS Manager Modal -->
    <LfsManagerModal
      v-model:open="isLfsManagerModalOpen"
    />

    <!-- Custom Action Runners & Script Automation Modal -->
    <CustomActionsModal
      v-model:open="isCustomActionsModalOpen"
    />
  </div>
</template>
