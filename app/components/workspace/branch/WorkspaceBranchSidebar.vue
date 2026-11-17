<script setup lang="ts">
import type {
  BranchSummary,
  RemoteBranchSummary,
  RepositoryReferences,
} from '~/types/repository'
import { buildBranchTree, buildRemoteBranchTree } from '~/utils/branchTree'
import BranchTreeNodeItem from '../BranchTreeNodeItem.vue'
import RemoteBranchTreeNodeItem from '../RemoteBranchTreeNodeItem.vue'
import RenameBranchModal from '../RenameBranchModal.vue'

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
  toggleFavoriteBranch: [branchName: string]
}>()

const { t } = useI18n()
const { renameBranch } = useWorkspaceManager()

const isRenameModalOpen = ref(false)
const branchToRename = ref('')

function openRenameBranchModal(branchName: string): void {
  branchToRename.value = branchName
  isRenameModalOpen.value = true
}

async function handleRenameBranch(oldName: string, newName: string): Promise<void> {
  await renameBranch(oldName, newName)
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
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden">
    <!-- Sidebar Header -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
      <span class="text-xs font-semibold text-highlighted uppercase tracking-wider">
        {{ t('navigation.references') }}
      </span>
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

        <!-- Remote Branches Tree -->
        <div v-if="props.references?.remote_branches?.length" class="mb-3">
          <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
            <span>{{ t('navigation.remotes') }}</span>
            <span class="text-[10px]">{{ props.references.remote_branches.length }}</span>
          </div>
          <div class="flex flex-col gap-0.5 mt-1">
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

        <!-- Tags -->
        <div v-if="props.references?.tags?.length">
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
      </template>
    </div>

    <!-- Rename Branch Modal -->
    <RenameBranchModal
      v-model:open="isRenameModalOpen"
      :current-branch-name="branchToRename"
      @rename="handleRenameBranch"
    />
  </div>
</template>
