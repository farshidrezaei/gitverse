<script setup lang="ts">
import type { BranchTreeNode } from '~/utils/branchTree'

const props = defineProps<{
  node: BranchTreeNode
}>()

const emit = defineEmits<{
  checkout: [branchName: string]
  delete: [branchName: string]
}>()

// Default folder open if it contains the HEAD branch
const isOpen = ref<boolean>(props.node.hasHeadBranch || true)

function toggleOpen(): void {
  isOpen.value = !isOpen.value
}
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
        @checkout="emit('checkout', $event)"
        @delete="emit('delete', $event)"
      />
    </div>

    <!-- Branch Leaf Item -->
    <div
      v-else-if="!node.isFolder && node.branch"
      class="group flex w-full items-center justify-between rounded-lg px-2 py-1.5 text-start transition-colors cursor-pointer"
      :class="node.branch.is_head
        ? 'bg-primary/15 font-semibold text-primary'
        : 'text-toned hover:bg-primary/10'"
      :title="node.fullBranchName"
      @click="emit('checkout', node.fullBranchName)"
    >
      <div class="flex items-center gap-1.5 min-w-0">
        <UIcon
          :name="node.branch.is_head ? 'i-lucide-check-circle-2' : 'i-lucide-git-branch'"
          class="h-3.5 w-3.5 shrink-0"
        />
        <span class="truncate font-mono text-[11px]">{{ node.name }}</span>
      </div>

      <div class="flex items-center gap-1 text-[10px]">
        <span v-if="node.branch.ahead_count > 0" class="text-success font-mono">↑{{ node.branch.ahead_count }}</span>
        <span v-if="node.branch.behind_count > 0" class="text-warning font-mono">↓{{ node.branch.behind_count }}</span>
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
  </div>
</template>
