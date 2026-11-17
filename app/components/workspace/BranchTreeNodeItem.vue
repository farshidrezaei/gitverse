<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { BranchTreeNode } from '~/utils/branchTree'
import type { BranchSummary } from '~/types/repository'

const props = defineProps<{
  node: BranchTreeNode
  selectedBranchName?: string | null
}>()

const emit = defineEmits<{
  checkout: [branchName: string]
  selectBranch: [branch: BranchSummary]
  createBranch: [sourceBranchName: string]
  delete: [branchName: string]
}>()

const { t } = useI18n()
const { isFavoriteBranch, toggleFavoriteBranch } = useWorkspaceManager()

// Default folder open if it contains the HEAD branch
const isOpen = ref<boolean>(props.node.hasHeadBranch || true)

function toggleOpen(): void {
  isOpen.value = !isOpen.value
}

const isStarred = computed(() => isFavoriteBranch(props.node.fullBranchName))
const isSelected = computed(() => props.selectedBranchName === props.node.fullBranchName)

const branchContextMenuItems = computed<ContextMenuItem[][]>(() => {
  if (props.node.isFolder || !props.node.branch) return []

  return [
    [
      {
        label: t('git.checkout'),
        icon: 'i-lucide-check',
        disabled: props.node.branch.is_head,
        onSelect: () => emit('checkout', props.node.fullBranchName),
      },
      {
        label: t('git.createBranch'),
        icon: 'i-lucide-git-branch-plus',
        onSelect: () => emit('createBranch', props.node.fullBranchName),
      },
      {
        label: isStarred.value ? t('git.unstarBranch') : t('git.starBranch'),
        icon: isStarred.value ? 'i-lucide-star-off' : 'i-lucide-star',
        onSelect: () => toggleFavoriteBranch(props.node.fullBranchName),
      },
    ],
    [
      {
        label: t('git.copyBranchName'),
        icon: 'i-lucide-copy',
        onSelect: async () => {
          await navigator.clipboard.writeText(props.node.fullBranchName)
        },
      },
    ],
    [
      {
        label: t('git.deleteBranch'),
        icon: 'i-lucide-trash-2',
        color: 'error',
        disabled: props.node.branch.is_head,
        onSelect: () => emit('delete', props.node.fullBranchName),
      },
    ],
  ]
})
</script>

<template>
  <div class="text-xs select-none">
    <!-- Folder Row -->
    <div
      v-if="node.isFolder"
      class="group flex items-center justify-between py-1 px-1.5 rounded-md hover:bg-muted/30 cursor-pointer transition-colors"
      @click="toggleOpen"
    >
      <div class="flex items-center gap-1.5 min-w-0">
        <UIcon
          :name="isOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
          class="h-3 w-3 shrink-0 text-muted transition-transform rtl:rotate-180"
        />
        <UIcon
          :name="isOpen ? 'i-lucide-folder-open' : 'i-lucide-folder'"
          class="h-3.5 w-3.5 shrink-0"
          :class="node.hasHeadBranch ? 'text-primary' : 'text-primary/70'"
        />
        <span class="font-medium text-highlighted truncate text-[11px] font-mono">{{ node.name }}</span>
      </div>
      <span class="text-[10px] text-muted font-mono px-1">
        {{ node.children.length }}
      </span>
    </div>

    <!-- Recursive Children for Folder -->
    <div
      v-if="node.isFolder && isOpen"
      class="ps-2.5 border-s border-default/40 ms-2 my-0.5 space-y-0.5"
    >
      <WorkspaceBranchTreeNodeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :selected-branch-name="selectedBranchName"
        @select-branch="emit('selectBranch', $event)"
        @checkout="emit('checkout', $event)"
        @create-branch="emit('createBranch', $event)"
        @delete="emit('delete', $event)"
      />
    </div>

    <!-- Branch Leaf Item Wrapped in Nuxt UI Context Menu -->
    <UContextMenu
      v-else-if="!node.isFolder && node.branch"
      :items="branchContextMenuItems"
    >
      <div
        class="group flex w-full items-center justify-between rounded-lg px-2 py-1.5 text-start transition-all cursor-pointer relative"
        style="content-visibility: auto; contain-intrinsic-size: 0 28px;"
        :class="[
          node.branch.is_head
            ? 'bg-primary/15 font-semibold text-primary border-s-2 border-primary ps-2'
            : isSelected
              ? 'bg-muted/40 text-highlighted ring-1 ring-primary/40 font-medium'
              : 'text-toned hover:bg-primary/10 hover:text-highlighted'
        ]"
        :title="node.branch.is_head ? `${node.fullBranchName} (${t('git.currentBranch')})` : `${node.fullBranchName} (${t('git.doubleClickToCheckout')})`"
        @click="emit('selectBranch', node.branch)"
        @dblclick="emit('checkout', node.fullBranchName)"
      >
        <div class="flex items-center gap-1.5 min-w-0">
          <UIcon
            :name="node.branch.is_head ? 'i-lucide-check-circle-2' : 'i-lucide-git-branch'"
            class="h-3.5 w-3.5 shrink-0"
            :class="node.branch.is_head ? 'text-primary' : isSelected ? 'text-highlighted' : 'text-muted'"
          />
          <span class="truncate font-mono text-[11px]">{{ node.name }}</span>

          <!-- Head Branch Pill Badge -->
          <UBadge
            v-if="node.branch.is_head"
            size="xs"
            color="primary"
            variant="subtle"
            class="font-mono text-[9px] px-1 py-0 leading-none shrink-0"
          >
            HEAD
          </UBadge>

          <!-- Starred Indicator Badge -->
          <UIcon
            v-if="isStarred"
            name="i-lucide-star"
            class="h-3 w-3 text-warning fill-warning shrink-0"
            :title="t('git.unstarBranch')"
            @click.stop="toggleFavoriteBranch(node.fullBranchName)"
          />
        </div>

        <div class="flex items-center gap-1 text-[10px]">
          <span v-if="node.branch.ahead_count > 0" class="text-success font-mono">↑{{ node.branch.ahead_count }}</span>
          <span v-if="node.branch.behind_count > 0" class="text-warning font-mono">↓{{ node.branch.behind_count }}</span>

          <!-- Quick Star/Unstar Action on Hover -->
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            :icon="isStarred ? 'i-lucide-star-off' : 'i-lucide-star'"
            class="h-4 w-4 opacity-0 group-hover:opacity-80 hover:text-warning"
            :title="isStarred ? t('git.unstarBranch') : t('git.starBranch')"
            @click.stop="toggleFavoriteBranch(node.fullBranchName)"
          />

          <UButton
            v-if="!node.branch.is_head"
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-trash-2"
            class="h-4 w-4 opacity-0 group-hover:opacity-80 hover:text-error"
            title="Delete branch"
            @click.stop="emit('delete', node.fullBranchName)"
          />
        </div>
      </div>
    </UContextMenu>
  </div>
</template>
