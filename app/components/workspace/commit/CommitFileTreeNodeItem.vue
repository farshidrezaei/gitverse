<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { CommitChangedFile } from '~/types/repository'
import type { FileTreeNode } from '~/utils/fileTree'
import FileStatusBadge from '~/components/shared/FileStatusBadge.vue'

defineOptions({
  name: 'CommitFileTreeNodeItem',
})

interface CommitFileTreeNodeItemProps {
  node: FileTreeNode<CommitChangedFile>
  selectedFilePath?: string | null
}

const props = defineProps<CommitFileTreeNodeItemProps>()

const emit = defineEmits<{
  selectFile: [filePath: string]
}>()

const { t } = useI18n()
const isOpen = ref(true)

function toggleOpen(): void {
  isOpen.value = !isOpen.value
}

const fileContextMenuItems = computed<ContextMenuItem[][]>(() => {
  if (props.node.isDirectory) {
    return []
  }

  return [
    [
      {
        label: t('git.copyPath'),
        icon: 'i-lucide-copy',
        onSelect: async () => {
          await navigator.clipboard.writeText(props.node.fullPath)
        },
      },
    ],
  ]
})
</script>

<template>
  <div class="text-xs select-none">
    <!-- Directory Row -->
    <div
      v-if="node.isDirectory"
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
          class="h-3.5 w-3.5 shrink-0 text-primary/80"
        />
        <span class="font-medium text-highlighted truncate">{{ node.name }}</span>
      </div>
    </div>

    <!-- Recursive Children for Directories -->
    <div
      v-if="node.isDirectory && isOpen"
      class="ps-3 border-s border-default/40 ms-2 my-0.5 space-y-0.5"
    >
      <CommitFileTreeNodeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :selected-file-path="props.selectedFilePath"
        @select-file="emit('selectFile', $event)"
      />
    </div>

    <!-- File Item Row Wrapped in Context Menu -->
    <UContextMenu
      v-else-if="!node.isDirectory"
      :items="fileContextMenuItems"
    >
      <div
        class="group flex items-center justify-between py-1 px-1.5 rounded-md cursor-pointer transition-colors"
        :class="props.selectedFilePath === node.fullPath
          ? 'bg-primary/15 text-primary font-medium'
          : 'hover:bg-muted/40'"
        @click="emit('selectFile', node.fullPath)"
      >
        <div class="flex items-center gap-1.5 min-w-0 flex-1 me-2">
          <FileStatusBadge
            v-if="node.fileEntry"
            :status="node.fileEntry.status"
            size="xs"
          />
          <UIcon name="i-lucide-file-code" class="h-3.5 w-3.5 shrink-0 text-muted" />
          <span class="truncate font-mono text-[11px]" dir="ltr">
            {{ node.name }}
          </span>
        </div>

        <!-- Additions / Deletions count -->
        <div
          v-if="node.fileEntry"
          class="flex items-center gap-1 shrink-0 font-mono text-[10px]"
        >
          <span v-if="node.fileEntry.additions > 0" class="text-success">
            +{{ node.fileEntry.additions }}
          </span>
          <span v-if="node.fileEntry.deletions > 0" class="text-error">
            -{{ node.fileEntry.deletions }}
          </span>
        </div>
      </div>
    </UContextMenu>
  </div>
</template>
