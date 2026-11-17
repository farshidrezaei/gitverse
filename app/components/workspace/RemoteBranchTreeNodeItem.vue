<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { RemoteBranchTreeNode } from '~/utils/branchTree'

const props = defineProps<{
  node: RemoteBranchTreeNode
}>()

const emit = defineEmits<{
  checkout: [branchName: string]
}>()

const { t } = useI18n()
const isOpen = ref(true)

function toggleOpen(): void {
  isOpen.value = !isOpen.value
}

const remoteContextMenuItems = computed<ContextMenuItem[][]>(() => {
  if (props.node.isFolder) return []

  return [
    [
      {
        label: t('git.checkout'),
        icon: 'i-lucide-check',
        onSelect: () => emit('checkout', props.node.fullBranchName),
      },
      {
        label: t('git.copyBranchName'),
        icon: 'i-lucide-copy',
        onSelect: async () => {
          await navigator.clipboard.writeText(props.node.fullBranchName)
        },
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
          class="h-3.5 w-3.5 shrink-0 text-primary/70"
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
      <WorkspaceRemoteBranchTreeNodeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        @checkout="emit('checkout', $event)"
      />
    </div>

    <!-- Remote Branch Leaf Item Wrapped in Context Menu -->
    <UContextMenu
      v-else-if="!node.isFolder"
      :items="remoteContextMenuItems"
    >
      <div
        class="flex w-full items-center gap-1.5 rounded-lg px-2 py-1 text-start text-toned text-[11px] font-mono hover:bg-primary/10 cursor-pointer transition-colors"
        :title="node.fullBranchName"
        @click="emit('checkout', node.fullBranchName)"
      >
        <UIcon name="i-lucide-cloud" class="h-3.5 w-3.5 shrink-0 text-muted" />
        <span class="truncate">{{ node.name }}</span>
      </div>
    </UContextMenu>
  </div>
</template>
