<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type {
  BranchSummary,
  RemoteBranchSummary,
  RemoteSummary,
  RepositoryReferences,
} from '~/types/repository'
import { buildBranchTree, buildRemoteBranchTree } from '~/utils/branchTree'
import BranchTreeNodeItem from '../BranchTreeNodeItem.vue'
import RemoteBranchTreeNodeItem from '../RemoteBranchTreeNodeItem.vue'
import RenameBranchModal from '../RenameBranchModal.vue'
import AddRemoteModal from '../remote/AddRemoteModal.vue'
import EditRemoteModal from '../remote/EditRemoteModal.vue'
import WorkspacePullRequestsModal from '../provider/WorkspacePullRequestsModal.vue'

interface WorkspaceBranchSidebarProps {
  references: RepositoryReferences | null
  favoriteBranches: string[]
  selectedBranchName: string | null
  isLoading: boolean
}

const props = defineProps<WorkspaceBranchSidebarProps>()

const emit = defineEmits<{
  selectBranch: [branch: BranchSummary | RemoteBranchSummary]
  checkout: [branchName: string]
  createBranchFromBranch: [branchName: string]
  deleteBranch: [branchName: string]
  openCreateBranchModal: []
  openCreateWorktreeModal: []
  openWorktreeInTab: [directoryPath: string]
  toggleFavoriteBranch: [branchName: string]
}>()

const { t } = useI18n()
const { currentRepository } = useRepositoryWorkspace()
const { renameBranch } = useWorkspaceManager()
const { worktreeList, isLoadingWorktrees, fetchWorktrees, removeWorktree } = useWorktreeController()
const {
  fetchRemote,
  addRemote,
  removeRemote,
  renameRemote,
  setRemoteUrl,
  copyRemoteUrl,
} = useRemoteController()
const { confirmStandard } = useConfirmationDialog()

watch(
  () => currentRepository.value?.path,
  () => {
    void fetchWorktrees()
  },
  { immediate: true },
)

async function handleRemoveWorktree(worktreePath: string): Promise<void> {
  await removeWorktree(worktreePath)
}

const isRenameModalOpen = ref(false)
const branchToRename = ref('')
const isPullRequestsModalOpen = ref(false)
const hasRemotes = computed(() => (props.references?.remotes?.length ?? 0) > 0)

function openRenameBranchModal(branchName: string): void {
  branchToRename.value = branchName
  isRenameModalOpen.value = true
}

async function handleRenameBranch(oldName: string, newName: string): Promise<void> {
  await renameBranch(oldName, newName)
}

// Remote Management Modals & Handlers
const isAddRemoteModalOpen = ref(false)
const isEditRemoteModalOpen = ref(false)
const selectedRemoteToEdit = ref<RemoteSummary | null>(null)

const existingRemoteNames = computed(() => {
  return props.references?.remotes?.map(remote => remote.name) || []
})

async function handleAddRemote(name: string, url: string, shouldFetchImmediately: boolean): Promise<void> {
  const isSuccess = await addRemote(name, url, shouldFetchImmediately)
  if (isSuccess) {
    isAddRemoteModalOpen.value = false
  }
}

function openEditRemoteModal(remote: RemoteSummary): void {
  selectedRemoteToEdit.value = remote
  isEditRemoteModalOpen.value = true
}

async function handleSaveEditRemote(oldName: string, newName: string, newUrl: string): Promise<void> {
  let isRenamed = true
  if (oldName !== newName) {
    isRenamed = await renameRemote(oldName, newName)
  }

  if (isRenamed) {
    const effectiveName = oldName !== newName ? newName : oldName
    const isUrlUpdated = await setRemoteUrl(effectiveName, newUrl)
    if (isUrlUpdated) {
      isEditRemoteModalOpen.value = false
    }
  }
}

async function handleRemoveRemote(remote: RemoteSummary): Promise<void> {
  const isConfirmed = await confirmStandard({
    title: t('remote.removeConfirmTitle', { name: remote.name }),
    description: t('remote.removeConfirmDescription', { name: remote.name }),
    variant: 'danger',
    confirmLabel: t('common.delete'),
    details: [
      { label: t('remote.remoteNameLabel'), value: remote.name },
      { label: t('remote.remoteUrlLabel'), value: remote.fetch_url || remote.push_url || '', isMonospace: true },
    ],
  })

  if (isConfirmed) {
    await removeRemote(remote.name)
  }
}

function getRemoteContextMenu(remote: RemoteSummary): ContextMenuItem[][] {
  const remoteUrl = remote.fetch_url || remote.push_url || ''
  return [
    [
      {
        label: t('remote.fetchFrom', { name: remote.name }),
        icon: 'i-lucide-refresh-cw',
        onSelect: () => void fetchRemote(remote.name),
      },
      {
        label: t('remote.copyUrl'),
        icon: 'i-lucide-copy',
        disabled: !remoteUrl,
        onSelect: () => void copyRemoteUrl(remoteUrl),
      },
    ],
    [
      {
        label: t('remote.editRemote'),
        icon: 'i-lucide-pencil',
        onSelect: () => openEditRemoteModal(remote),
      },
      {
        label: t('remote.removeRemote'),
        icon: 'i-lucide-trash-2',
        color: 'error' as const,
        onSelect: () => void handleRemoveRemote(remote),
      },
    ],
  ]
}

const branchTreeNodes = computed(() => {
  return buildBranchTree(props.references?.branches || [])
})

const remoteBranchTreeNodes = computed(() => {
  return buildRemoteBranchTree(props.references?.remote_branches || [])
})

const favoriteBranchSummaries = computed(() => {
  if (!props.references?.branches) return []
  return props.references.branches.filter((branch) =>
    props.favoriteBranches.includes(branch.name)
  )
})

const existingBranchNames = computed(() => {
  return props.references?.branches.map((branch) => branch.name) || []
})
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden">
    <!-- Sidebar Header -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
      <span class="text-xs font-semibold text-highlighted uppercase tracking-wider">
        {{ t('navigation.references') }}
      </span>
      <div class="flex items-center gap-1">
        <UButton
          v-if="hasRemotes"
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-git-pull-request"
          class="h-6 w-6 text-primary cursor-pointer"
          :title="t('provider.pullRequests')"
          @click="isPullRequestsModalOpen = true"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-plus"
          class="h-6 w-6"
          :title="t('git.createBranch')"
          @click="emit('openCreateBranchModal')"
        />
      </div>
    </div>

    <!-- Branch & Reference Tree -->
    <div class="flex-1 overflow-y-auto p-2 text-xs">
      <!-- References Skeleton Loading -->
      <div v-if="props.isLoading && !props.references" class="space-y-4 p-1">
        <div>
          <div class="flex items-center justify-between px-2 py-1">
            <USkeleton class="h-3 w-16" />
            <USkeleton class="h-3 w-4" />
          </div>
          <div class="space-y-1 mt-1.5">
            <div v-for="skeletonIndex in 3" :key="skeletonIndex" class="flex items-center gap-2 rounded-lg px-2 py-1.5">
              <USkeleton class="h-3.5 w-3.5 rounded-full shrink-0" />
              <USkeleton class="h-3 w-28" />
            </div>
          </div>
        </div>
        <div>
          <div class="flex items-center justify-between px-2 py-1">
            <USkeleton class="h-3 w-14" />
            <USkeleton class="h-3 w-4" />
          </div>
          <div class="space-y-1 mt-1.5">
            <div v-for="skeletonIndex in 2" :key="skeletonIndex" class="flex items-center gap-2 rounded-lg px-2 py-1.5">
              <USkeleton class="h-3.5 w-3.5 rounded shrink-0" />
              <USkeleton class="h-3 w-24" />
            </div>
          </div>
        </div>
      </div>

      <template v-else>
        <!-- Starred / Favorite Branches Section -->
        <div v-if="favoriteBranchSummaries.length > 0" class="mb-3">
          <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-warning uppercase tracking-wider">
            <span class="flex items-center gap-1.5">
              <UIcon name="i-lucide-star" class="h-3 w-3 fill-warning text-warning" />
              {{ t('navigation.favorites') }}
            </span>
            <span class="text-[10px]">{{ favoriteBranchSummaries.length }}</span>
          </div>
          <div class="flex flex-col gap-0.5 mt-1">
            <div
              v-for="branch in favoriteBranchSummaries"
              :key="branch.name"
              class="group flex w-full items-center justify-between rounded-lg px-2 py-1.5 text-start transition-all cursor-pointer relative"
              :class="[
                branch.is_head
                  ? 'bg-primary/15 font-semibold text-primary border-s-2 border-primary ps-2'
                  : props.selectedBranchName === branch.name
                    ? 'bg-muted/40 text-highlighted ring-1 ring-primary/40 font-medium'
                    : 'text-toned hover:bg-primary/10'
              ]"
              :title="branch.is_head ? `${branch.name} (${t('git.currentBranch')})` : `${branch.name} (${t('git.doubleClickToCheckout')})`"
              @click="emit('selectBranch', branch)"
              @dblclick="emit('checkout', branch.name)"
            >
              <div class="flex items-center gap-1.5 min-w-0">
                <UIcon
                  :name="branch.is_head ? 'i-lucide-check-circle-2' : 'i-lucide-git-branch'"
                  class="h-3.5 w-3.5 shrink-0"
                  :class="branch.is_head ? 'text-primary' : props.selectedBranchName === branch.name ? 'text-highlighted' : 'text-muted'"
                />
                <span class="truncate font-mono text-[11px]">{{ branch.name }}</span>

                <!-- Head Branch Pill Badge -->
                <UBadge
                  v-if="branch.is_head"
                  size="xs"
                  color="primary"
                  variant="subtle"
                  class="font-mono text-[9px] px-1 py-0 leading-none shrink-0"
                >
                  HEAD
                </UBadge>
              </div>
              <div class="flex items-center gap-1 text-[10px]">
                <span v-if="branch.ahead_count > 0" class="text-success font-mono">↑{{ branch.ahead_count }}</span>
                <span v-if="branch.behind_count > 0" class="text-warning font-mono">↓{{ branch.behind_count }}</span>
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-star-off"
                  class="h-4 w-4 opacity-0 group-hover:opacity-80 hover:text-warning"
                  :title="t('git.unstarBranch')"
                  @click.stop="emit('toggleFavoriteBranch', branch.name)"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Local Branches Tree -->
        <div class="mb-3">
          <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
            <span>{{ t('git.branches') }}</span>
            <span class="text-[10px]">{{ props.references?.branches?.length || 0 }}</span>
          </div>
          <div class="flex flex-col gap-0.5 mt-1">
            <BranchTreeNodeItem
              v-for="node in branchTreeNodes"
              :key="node.id"
              :node="node"
              :selected-branch-name="props.selectedBranchName"
              @select-branch="emit('selectBranch', $event)"
              @checkout="emit('checkout', $event)"
              @create-branch="emit('createBranchFromBranch', $event)"
              @delete="emit('deleteBranch', $event)"
              @rename="openRenameBranchModal"
            />
          </div>
        </div>

        <!-- Remotes -->
        <div class="mb-3">
          <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
            <span>{{ t('navigation.remotes') }}</span>
            <div class="flex items-center gap-1">
              <span class="text-[10px]">{{ props.references?.remotes?.length || 0 }}</span>
              <button
                type="button"
                class="rounded p-0.5 text-muted hover:bg-primary/20 hover:text-primary transition-colors cursor-pointer"
                :title="t('remote.addRemote')"
                @click="isAddRemoteModalOpen = true"
              >
                <UIcon name="i-lucide-plus" class="h-3.5 w-3.5" />
              </button>
            </div>
          </div>

          <!-- Remotes List -->
          <div v-if="props.references?.remotes?.length" class="flex flex-col gap-0.5 mt-1">
            <UContextMenu
              v-for="remote in props.references.remotes"
              :key="remote.name"
              :items="getRemoteContextMenu(remote)"
            >
              <div
                class="group flex w-full items-center justify-between gap-1.5 rounded-lg px-2 py-1 text-start text-toned text-[11px] hover:bg-surface/80 cursor-pointer transition-colors"
                :title="remote.fetch_url || remote.push_url || remote.name"
              >
                <div class="flex items-center gap-1.5 min-w-0">
                  <UIcon name="i-lucide-globe" class="h-3.5 w-3.5 shrink-0 text-primary" />
                  <span class="font-bold text-highlighted truncate">{{ remote.name }}</span>
                  <span
                    v-if="remote.fetch_url"
                    class="font-mono text-[9px] text-muted truncate max-w-[120px]"
                    dir="ltr"
                  >
                    {{ remote.fetch_url }}
                  </span>
                </div>

                <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    type="button"
                    class="rounded p-0.5 text-muted hover:text-primary transition-colors cursor-pointer"
                    :title="t('remote.fetchFrom', { name: remote.name })"
                    @click.stop="fetchRemote(remote.name)"
                  >
                    <UIcon name="i-lucide-refresh-cw" class="h-3 w-3" />
                  </button>
                  <button
                    type="button"
                    class="rounded p-0.5 text-muted hover:text-highlighted transition-colors cursor-pointer"
                    :title="t('remote.editRemote')"
                    @click.stop="openEditRemoteModal(remote)"
                  >
                    <UIcon name="i-lucide-pencil" class="h-3 w-3" />
                  </button>
                  <button
                    type="button"
                    class="rounded p-0.5 text-muted hover:text-error transition-colors cursor-pointer"
                    :title="t('remote.removeRemote')"
                    @click.stop="handleRemoveRemote(remote)"
                  >
                    <UIcon name="i-lucide-trash-2" class="h-3 w-3" />
                  </button>
                </div>
              </div>
            </UContextMenu>

            <!-- Remote Tracking Branches Tree (Filtered or nested under remotes) -->
            <div v-if="remoteBranchTreeNodes.length" class="ps-2 mt-1 space-y-0.5 border-s border-default/40 ms-2">
              <RemoteBranchTreeNodeItem
                v-for="node in remoteBranchTreeNodes"
                :key="node.id"
                :node="node"
                :selected-branch-name="props.selectedBranchName"
                @select-branch="emit('selectBranch', $event)"
                @checkout="emit('checkout', $event)"
              />
            </div>
          </div>

          <div
            v-else
            class="px-2 py-1.5 text-[10px] text-muted italic flex items-center justify-between"
          >
            <span>No remotes</span>
            <UButton
              size="xs"
              variant="link"
              class="text-[10px] p-0 text-primary cursor-pointer"
              :label="t('remote.addRemote')"
              @click="isAddRemoteModalOpen = true"
            />
          </div>
        </div>

        <!-- Tags -->
        <div v-if="props.references?.tags?.length" class="mb-3">
          <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
            <span>{{ t('git.tags') }}</span>
            <span class="text-[10px]">{{ props.references.tags.length }}</span>
          </div>
          <div class="flex flex-col gap-0.5 mt-1">
            <div
              v-for="tag in props.references.tags"
              :key="tag.name"
              class="flex w-full items-center gap-1.5 rounded-lg px-2 py-1 text-start text-toned text-[11px] font-mono hover:bg-primary/10"
            >
              <UIcon name="i-lucide-tag" class="h-3.5 w-3.5 shrink-0 text-muted" />
              <span class="truncate">{{ tag.name }}</span>
            </div>
          </div>
        </div>

        <!-- Worktrees -->
        <div class="mb-3">
          <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
            <span>{{ t('worktree.title') }}</span>
            <div class="flex items-center gap-1">
              <span class="text-[10px]">{{ worktreeList.length }}</span>
              <button
                type="button"
                class="rounded p-0.5 text-muted hover:bg-primary/20 hover:text-primary transition-colors"
                :title="t('worktree.createTitle')"
                @click="emit('openCreateWorktreeModal')"
              >
                <UIcon name="i-lucide-plus" class="h-3.5 w-3.5" />
              </button>
            </div>
          </div>
          <!-- Worktrees Skeleton Loading -->
          <div v-if="isLoadingWorktrees && worktreeList.length === 0" class="space-y-1 mt-1 animate-pulse">
            <div v-for="wIndex in 2" :key="`worktree-skeleton-${wIndex}`" class="flex items-center justify-between rounded-lg px-2 py-1">
              <div class="flex items-center gap-1.5">
                <USkeleton class="h-3.5 w-3.5 rounded shrink-0" />
                <USkeleton class="h-3 w-28 rounded" />
              </div>
              <USkeleton class="h-4 w-10 rounded" />
            </div>
          </div>
          <div v-else class="flex flex-col gap-0.5 mt-1">
            <div
              v-for="worktree in worktreeList"
              :key="worktree.directory_path"
              class="group flex w-full items-center justify-between rounded-lg px-2 py-1 text-toned text-[11px] font-mono hover:bg-primary/10 transition-colors cursor-pointer"
              :class="worktree.directory_path === currentRepository?.path ? 'bg-primary/15 text-primary font-semibold' : ''"
              @click="emit('openWorktreeInTab', worktree.directory_path)"
            >
              <div class="flex items-center gap-1.5 min-w-0">
                <UIcon name="i-lucide-git-fork" class="h-3.5 w-3.5 shrink-0 text-primary" />
                <span class="truncate" :title="worktree.directory_path">
                  {{ worktree.branch_name || worktree.head_commit_hash.slice(0, 7) }}
                </span>
              </div>
              <div class="flex items-center gap-1 shrink-0">
                <UBadge v-if="worktree.is_main" size="xs" color="primary" variant="subtle">Main</UBadge>
                <UBadge v-else-if="worktree.is_locked" size="xs" color="warning" variant="subtle">Locked</UBadge>
                <button
                  v-if="!worktree.is_main"
                  type="button"
                  class="rounded p-0.5 text-muted opacity-0 group-hover:opacity-100 hover:bg-error/20 hover:text-error transition-all"
                  :title="t('common.delete')"
                  @click.stop="handleRemoveWorktree(worktree.directory_path)"
                >
                  <UIcon name="i-lucide-trash-2" class="h-3 w-3" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Rename Branch Modal -->
    <RenameBranchModal
      v-model:open="isRenameModalOpen"
      :current-branch-name="branchToRename"
      :existing-branch-names="existingBranchNames"
      @rename="handleRenameBranch"
    />

    <!-- Add Remote Modal -->
    <AddRemoteModal
      v-model:open="isAddRemoteModalOpen"
      :existing-remote-names="existingRemoteNames"
      @add="handleAddRemote"
    />

    <!-- Edit Remote Modal -->
    <EditRemoteModal
      v-model:open="isEditRemoteModalOpen"
      :remote="selectedRemoteToEdit"
      :existing-remote-names="existingRemoteNames"
      @save="handleSaveEditRemote"
    />

    <!-- Pull Requests Modal -->
    <WorkspacePullRequestsModal
      v-model:open="isPullRequestsModalOpen"
      :remotes="props.references?.remotes"
      @checkout-branch="emit('checkout', $event)"
    />
  </div>
</template>
