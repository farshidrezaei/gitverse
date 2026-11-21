<script setup lang="ts">
import type {
  BranchSummary,
  RemoteBranchSummary,
  RepositoryReferences,
  DetailedRepositoryStatus,
  FileChangeStatus,
} from '~/types/repository'
import type { RemotePullRequest, ProviderAccount } from '~/types/provider'
import type { SelectedDiffTarget } from '~/composables/useStagingController'
import ExplorerChangesSection from './ExplorerChangesSection.vue'
import ExplorerBranchesSection from './ExplorerBranchesSection.vue'
import ExplorerPullRequestsSection from './ExplorerPullRequestsSection.vue'
import ExplorerStashesSection from './ExplorerStashesSection.vue'
import ExplorerRemotesSection from './ExplorerRemotesSection.vue'
import ExplorerTagsSection from './ExplorerTagsSection.vue'
import ExplorerWorktreesSection from './ExplorerWorktreesSection.vue'
import RenameBranchModal from '../RenameBranchModal.vue'

interface WorkspaceUnifiedExplorerProps {
  repositoryStatus: DetailedRepositoryStatus | null
  references: RepositoryReferences | null
  favoriteBranches: string[]
  selectedBranchName: string | null
  selectedDiffTarget: SelectedDiffTarget | null
  stagingViewMode: 'flat' | 'tree'
  commitMessage: string
  isAmend: boolean
  isBypassHooks?: boolean
  isLoading: boolean
  isOperating?: boolean
}

const props = withDefaults(defineProps<WorkspaceUnifiedExplorerProps>(), {
  isBypassHooks: false,
  isOperating: false,
})

const emit = defineEmits<{
  'update:stagingViewMode': [mode: 'flat' | 'tree']
  'update:commitMessage': [value: string]
  'update:isAmend': [value: boolean]
  'update:isBypassHooks': [value: boolean]
  stageFile: [filePath: string]
  unstageFile: [filePath: string]
  stageAll: []
  unstageAll: []
  discardFile: [filePath: string, isUntracked: boolean]
  discardAllUnstaged: []
  selectFile: [filePath: string, isStaged: boolean]
  openDiffTab: [filePath: string, isStaged: boolean, changeStatus?: FileChangeStatus]
  commit: []
  refresh: []
  selectBranch: [branch: BranchSummary | RemoteBranchSummary]
  checkout: [branchName: string]
  createBranchFromBranch: [branchName: string]
  deleteBranch: [branchName: string]
  openCreateBranchModal: []
  openCreateWorktreeModal: []
  openWorktreeInTab: [directoryPath: string]
  toggleFavoriteBranch: [branchName: string]
  openPullRequest: [pullRequest: RemotePullRequest, account: ProviderAccount, repositoryFullName: string]
}>()

const { t } = useI18n()
const { currentRepository } = useRepositoryWorkspace()
const { renameBranch } = useWorkspaceManager()

// Accordion toggle states
const isChangesOpen = ref<boolean>(true)
const isBranchesOpen = ref<boolean>(true)
const isPullRequestsOpen = ref<boolean>(false)
const isRemotesOpen = ref<boolean>(false)
const isStashesOpen = ref<boolean>(false)
const isTagsOpen = ref<boolean>(false)
const isWorktreesOpen = ref<boolean>(false)

// Modals
const isRenameModalOpen = ref<boolean>(false)
const branchToRename = ref<string>('')

const existingBranchNames = computed(() => {
  return props.references?.branches.map((branch) => branch.name) || []
})

const hasRemotes = computed(() => (props.references?.remotes?.length ?? 0) > 0)

function openRenameBranchModal(branchName: string): void {
  branchToRename.value = branchName
  isRenameModalOpen.value = true
}

async function handleRenameBranch(oldName: string, newName: string): Promise<void> {
  await renameBranch(oldName, newName)
}

const totalChangesCount = computed(() => {
  const staged = props.repositoryStatus?.staged_files?.length || 0
  const unstaged = props.repositoryStatus?.unstaged_files?.length || 0
  const untracked = props.repositoryStatus?.untracked_files?.length || 0
  const conflicted = props.repositoryStatus?.conflicted_files?.length || 0
  return staged + unstaged + untracked + conflicted
})
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden bg-[var(--gv-sidebar)] border-e border-default/70 select-none">
    <!-- Top Explorer Title Bar -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
      <div class="flex items-center gap-2 min-w-0">
        <UIcon name="i-lucide-folder-git-2" class="h-4 w-4 text-primary shrink-0" />
        <span class="text-xs font-bold text-highlighted uppercase tracking-wider truncate">
          {{ currentRepository?.name || t('navigation.workspace') }}
        </span>
      </div>

      <div class="flex items-center gap-1">
        <UTooltip :text="t('common.refresh')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-refresh-cw"
            class="h-6 w-6"
            :loading="props.isLoading"
            :aria-label="t('common.refresh')"
            @click="emit('refresh')"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Scrollable Unified Sections -->
    <div class="flex-1 overflow-y-auto divide-y divide-default/40">
      <!-- SECTION 1: Changes / Working Tree -->
      <div>
        <button
          type="button"
          class="flex w-full items-center justify-between px-3 py-2 text-xs font-semibold text-highlighted hover:bg-muted/30 transition-colors"
          @click="isChangesOpen = !isChangesOpen"
        >
          <div class="flex items-center gap-2 min-w-0">
            <UIcon
              :name="isChangesOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="h-3.5 w-3.5 text-muted transition-transform rtl:rotate-180 shrink-0"
            />
            <UIcon name="i-lucide-file-diff" class="h-3.5 w-3.5 text-primary shrink-0" />
            <span class="uppercase tracking-wider text-[11px]">{{ t('navigation.changes') }}</span>
          </div>

          <UBadge
            v-if="totalChangesCount > 0"
            size="xs"
            color="warning"
            variant="subtle"
            class="font-mono text-[9px] px-1.5 py-0"
          >
            {{ totalChangesCount }}
          </UBadge>
        </button>

        <div v-show="isChangesOpen">
          <ExplorerChangesSection
            :repository-status="props.repositoryStatus"
            :selected-diff-target="props.selectedDiffTarget"
            :staging-view-mode="props.stagingViewMode"
            :is-loading="props.isLoading"
            :is-operating="props.isOperating"
            :commit-message="props.commitMessage"
            :is-amend="props.isAmend"
            :is-bypass-hooks="props.isBypassHooks"
            @update:staging-view-mode="emit('update:stagingViewMode', $event)"
            @update:commit-message="emit('update:commitMessage', $event)"
            @update:is-amend="emit('update:isAmend', $event)"
            @update:is-bypass-hooks="emit('update:isBypassHooks', $event)"
            @stage-file="emit('stageFile', $event)"
            @unstage-file="emit('unstageFile', $event)"
            @stage-all="emit('stageAll')"
            @unstage-all="emit('unstageAll')"
            @discard-file="(path, untracked) => emit('discardFile', path, untracked)"
            @discard-all-unstaged="emit('discardAllUnstaged')"
            @select-file="(path, staged) => emit('selectFile', path, staged)"
            @open-diff-tab="(path, staged, status) => emit('openDiffTab', path, staged, status)"
            @commit="emit('commit')"
            @refresh="emit('refresh')"
          />
        </div>
      </div>

      <!-- SECTION 2: Branches -->
      <div>
        <button
          type="button"
          class="flex w-full items-center justify-between px-3 py-2 text-xs font-semibold text-highlighted hover:bg-muted/30 transition-colors"
          @click="isBranchesOpen = !isBranchesOpen"
        >
          <div class="flex items-center gap-2 min-w-0">
            <UIcon
              :name="isBranchesOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="h-3.5 w-3.5 text-muted transition-transform rtl:rotate-180 shrink-0"
            />
            <UIcon name="i-lucide-git-branch" class="h-3.5 w-3.5 text-primary shrink-0" />
            <span class="uppercase tracking-wider text-[11px]">{{ t('git.branches') }}</span>
          </div>

          <span class="text-[10px] text-muted font-mono">
            {{ props.references?.branches?.length || 0 }}
          </span>
        </button>

        <div v-show="isBranchesOpen">
          <ExplorerBranchesSection
            :references="props.references"
            :favorite-branches="props.favoriteBranches"
            :selected-branch-name="props.selectedBranchName"
            :is-loading="props.isLoading"
            @select-branch="emit('selectBranch', $event)"
            @checkout="emit('checkout', $event)"
            @create-branch-from-branch="emit('createBranchFromBranch', $event)"
            @delete-branch="emit('deleteBranch', $event)"
            @open-create-branch-modal="emit('openCreateBranchModal')"
            @toggle-favorite-branch="emit('toggleFavoriteBranch', $event)"
            @open-rename-modal="openRenameBranchModal"
          />
        </div>
      </div>

      <!-- SECTION 3: Pull Requests -->
      <div v-if="hasRemotes">
        <button
          type="button"
          class="flex w-full items-center justify-between px-3 py-2 text-xs font-semibold text-highlighted hover:bg-muted/30 transition-colors cursor-pointer"
          @click="isPullRequestsOpen = !isPullRequestsOpen"
        >
          <div class="flex items-center gap-2 min-w-0">
            <UIcon
              :name="isPullRequestsOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="h-3.5 w-3.5 text-muted transition-transform rtl:rotate-180 shrink-0"
            />
            <UIcon name="i-lucide-git-pull-request" class="h-3.5 w-3.5 text-primary shrink-0" />
            <span class="uppercase tracking-wider text-[11px]">{{ t('provider.pullRequests') }}</span>
          </div>
        </button>

        <div v-show="isPullRequestsOpen">
          <ExplorerPullRequestsSection
            :remotes="props.references?.remotes"
            :current-repository-path="currentRepository?.path"
            @open-pull-request="(pullRequest, account, repositoryFullName) => emit('openPullRequest', pullRequest, account, repositoryFullName)"
            @checkout-branch="emit('checkout', $event)"
          />
        </div>
      </div>

      <!-- SECTION 4: Stashes -->
      <div>
        <button
          type="button"
          class="flex w-full items-center justify-between px-3 py-2 text-xs font-semibold text-highlighted hover:bg-muted/30 transition-colors"
          @click="isStashesOpen = !isStashesOpen"
        >
          <div class="flex items-center gap-2 min-w-0">
            <UIcon
              :name="isStashesOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="h-3.5 w-3.5 text-muted transition-transform rtl:rotate-180 shrink-0"
            />
            <UIcon name="i-lucide-archive" class="h-3.5 w-3.5 text-primary shrink-0" />
            <span class="uppercase tracking-wider text-[11px]">{{ t('stash.title') }}</span>
          </div>
        </button>

        <div v-show="isStashesOpen">
          <ExplorerStashesSection :is-loading="props.isLoading" />
        </div>
      </div>

      <!-- SECTION 4: Remotes -->
      <div>
        <button
          type="button"
          class="flex w-full items-center justify-between px-3 py-2 text-xs font-semibold text-highlighted hover:bg-muted/30 transition-colors"
          @click="isRemotesOpen = !isRemotesOpen"
        >
          <div class="flex items-center gap-2 min-w-0">
            <UIcon
              :name="isRemotesOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="h-3.5 w-3.5 text-muted transition-transform rtl:rotate-180 shrink-0"
            />
            <UIcon name="i-lucide-globe" class="h-3.5 w-3.5 text-primary shrink-0" />
            <span class="uppercase tracking-wider text-[11px]">{{ t('navigation.remotes') }}</span>
          </div>

          <span class="text-[10px] text-muted font-mono">
            {{ props.references?.remotes?.length || 0 }}
          </span>
        </button>

        <div v-show="isRemotesOpen">
          <ExplorerRemotesSection :remotes="props.references?.remotes" />
        </div>
      </div>

      <!-- SECTION 5: Tags -->
      <div>
        <button
          type="button"
          class="flex w-full items-center justify-between px-3 py-2 text-xs font-semibold text-highlighted hover:bg-muted/30 transition-colors"
          @click="isTagsOpen = !isTagsOpen"
        >
          <div class="flex items-center gap-2 min-w-0">
            <UIcon
              :name="isTagsOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="h-3.5 w-3.5 text-muted transition-transform rtl:rotate-180 shrink-0"
            />
            <UIcon name="i-lucide-tag" class="h-3.5 w-3.5 text-primary shrink-0" />
            <span class="uppercase tracking-wider text-[11px]">{{ t('git.tags') }}</span>
          </div>

          <span class="text-[10px] text-muted font-mono">
            {{ props.references?.tags?.length || 0 }}
          </span>
        </button>

        <div v-show="isTagsOpen">
          <ExplorerTagsSection :tags="props.references?.tags" />
        </div>
      </div>

      <!-- SECTION 6: Worktrees -->
      <div>
        <button
          type="button"
          class="flex w-full items-center justify-between px-3 py-2 text-xs font-semibold text-highlighted hover:bg-muted/30 transition-colors"
          @click="isWorktreesOpen = !isWorktreesOpen"
        >
          <div class="flex items-center gap-2 min-w-0">
            <UIcon
              :name="isWorktreesOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="h-3.5 w-3.5 text-muted transition-transform rtl:rotate-180 shrink-0"
            />
            <UIcon name="i-lucide-git-fork" class="h-3.5 w-3.5 text-primary shrink-0" />
            <span class="uppercase tracking-wider text-[11px]">{{ t('worktree.title') }}</span>
          </div>
        </button>

        <div v-show="isWorktreesOpen">
          <ExplorerWorktreesSection
            @open-create-worktree-modal="emit('openCreateWorktreeModal')"
            @open-worktree-in-tab="emit('openWorktreeInTab', $event)"
          />
        </div>
      </div>
    </div>

    <!-- Modals -->
    <RenameBranchModal
      v-model:open="isRenameModalOpen"
      :current-branch-name="branchToRename"
      :existing-branch-names="existingBranchNames"
      @rename="handleRenameBranch"
    />
  </div>
</template>
