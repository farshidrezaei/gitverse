<script setup lang="ts">
import type {
  BranchSummary,
  RemoteBranchSummary,
  RepositoryReferences,
} from '~/types/repository'
import { buildBranchTree, buildRemoteBranchTree } from '~/utils/branchTree'
import BranchTreeNodeItem from '../BranchTreeNodeItem.vue'
import RemoteBranchTreeNodeItem from '../RemoteBranchTreeNodeItem.vue'

interface ExplorerBranchesSectionProps {
  references: RepositoryReferences | null
  favoriteBranches: string[]
  selectedBranchName: string | null
  isLoading: boolean
}

const props = defineProps<ExplorerBranchesSectionProps>()

const emit = defineEmits<{
  selectBranch: [branch: BranchSummary | RemoteBranchSummary]
  checkout: [branchName: string]
  createBranchFromBranch: [branchName: string]
  deleteBranch: [branchName: string]
  openCreateBranchModal: []
  toggleFavoriteBranch: [branchName: string]
  openRenameModal: [branchName: string]
}>()

const { t } = useI18n()

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
  <div class="flex flex-col gap-2 p-2 text-xs">
    <!-- Quick Add Branch Button -->
    <div class="flex items-center justify-between pb-1 border-b border-default/40">
      <span class="text-[11px] font-semibold text-muted uppercase tracking-wider">
        {{ t('git.branches') }}
      </span>
      <UTooltip :text="t('git.createBranch')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-plus"
          class="h-5 w-5 p-0"
          :aria-label="t('git.createBranch')"
          @click="emit('openCreateBranchModal')"
        />
      </UTooltip>
    </div>

    <!-- Skeletons loading state -->
    <div v-if="props.isLoading && !props.references" class="space-y-3 p-1">
      <div v-for="index in 3" :key="index" class="flex items-center gap-2">
        <USkeleton class="h-3.5 w-3.5 rounded-full shrink-0" />
        <USkeleton class="h-3 w-32" />
      </div>
    </div>

    <template v-else>
      <!-- Starred / Favorite Branches Section -->
      <div v-if="favoriteBranchSummaries.length > 0" class="mb-1">
        <div class="flex items-center justify-between px-2 py-0.5 text-[10px] font-semibold text-warning uppercase tracking-wider">
          <span class="flex items-center gap-1.5">
            <UIcon name="i-lucide-star" class="h-3 w-3 fill-warning text-warning" />
            {{ t('navigation.favorites') }}
          </span>
          <span class="text-[10px]">{{ favoriteBranchSummaries.length }}</span>
        </div>
        <div class="flex flex-col gap-0.5 mt-0.5">
          <div
            v-for="branch in favoriteBranchSummaries"
            :key="branch.name"
            class="group flex w-full items-center justify-between rounded-lg px-2 py-1 text-start transition-all cursor-pointer relative"
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

      <!-- Local Branches -->
      <div class="space-y-0.5">
        <BranchTreeNodeItem
          v-for="node in branchTreeNodes"
          :key="node.id"
          :node="node"
          :selected-branch-name="props.selectedBranchName"
          @select-branch="emit('selectBranch', $event)"
          @checkout="emit('checkout', $event)"
          @create-branch="emit('createBranchFromBranch', $event)"
          @delete="emit('deleteBranch', $event)"
          @rename="emit('openRenameModal', $event)"
        />
      </div>

      <!-- Remote Branches Header & Tree -->
      <div v-if="remoteBranchTreeNodes.length > 0" class="mt-2 pt-2 border-t border-default/40 space-y-1">
        <div class="px-2 text-[10px] font-semibold text-muted uppercase tracking-wider">
          {{ t('navigation.remotes') }}
        </div>
        <div class="space-y-0.5">
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
    </template>
  </div>
</template>
