<script setup lang="ts">
import type { FileTreeNode } from '~/utils/fileTree'
import type { PullRequestChangedFile } from '~/types/provider'

const props = defineProps<{
  nodes: FileTreeNode<{ path: string; file: PullRequestChangedFile }>[]
  selectedFilePath: string | null
}>()

const emit = defineEmits<{
  selectFile: [filePath: string]
}>()

const expandedFolders = ref<Set<string>>(new Set())

// Auto-expand all folders initially
watch(
  () => props.nodes,
  (nodes) => {
    function expandAll(items: FileTreeNode<{ path: string; file: PullRequestChangedFile }>[]) {
      for (const item of items) {
        if (item.isDirectory) {
          expandedFolders.value.add(item.id)
          expandAll(item.children)
        }
      }
    }
    expandAll(nodes)
  },
  { immediate: true },
)

function toggleFolder(folderId: string): void {
  if (expandedFolders.value.has(folderId)) {
    expandedFolders.value.delete(folderId)
  } else {
    expandedFolders.value.add(folderId)
  }
}
</script>

<template>
  <div class="space-y-0.5 select-none font-mono text-[11px]">
    <template v-for="node in nodes" :key="node.id">
      <!-- Directory Node -->
      <div v-if="node.isDirectory" class="space-y-0.5">
        <button
          type="button"
          class="flex w-full items-center gap-1.5 px-2 py-1 rounded hover:bg-muted/20 text-toned font-sans transition-colors text-start"
          @click="toggleFolder(node.id)"
        >
          <UIcon
            :name="expandedFolders.has(node.id) ? 'i-lucide-folder-open' : 'i-lucide-folder'"
            class="h-3.5 w-3.5 text-warning shrink-0"
          />
          <span class="truncate font-medium text-xs">{{ node.name }}</span>
        </button>

        <div v-if="expandedFolders.has(node.id)" class="ps-3 border-s border-default/30 ms-2.5">
          <PullRequestFileTreeNode
            :nodes="node.children"
            :selected-file-path="selectedFilePath"
            @select-file="emit('selectFile', $event)"
          />
        </div>
      </div>

      <!-- File Leaf Node -->
      <div
        v-else-if="node.fileEntry"
        class="flex items-center justify-between px-2 py-1 rounded cursor-pointer transition-colors group"
        :class="selectedFilePath === node.fileEntry.file.file_path
          ? 'bg-primary/15 text-primary font-semibold shadow-xs'
          : 'hover:bg-muted/20 text-toned'"
        @click="emit('selectFile', node.fileEntry.file.file_path)"
      >
        <div class="flex items-center gap-1.5 min-w-0">
          <UIcon
            :name="node.fileEntry.file.status === 'added' ? 'i-lucide-file-plus' :
                   node.fileEntry.file.status === 'deleted' ? 'i-lucide-file-minus' :
                   node.fileEntry.file.status === 'renamed' ? 'i-lucide-file-symlink' : 'i-lucide-file-edit'"
            class="h-3.5 w-3.5 shrink-0"
            :class="node.fileEntry.file.status === 'added' ? 'text-success' :
                    node.fileEntry.file.status === 'deleted' ? 'text-error' :
                    node.fileEntry.file.status === 'renamed' ? 'text-info' : 'text-warning'"
          />
          <span class="truncate" :title="node.fileEntry.file.file_path">
            {{ node.name }}
          </span>
        </div>

        <div class="flex items-center gap-1 text-[10px] shrink-0 ms-2">
          <span v-if="node.fileEntry.file.additions_count > 0" class="text-success font-bold">+{{ node.fileEntry.file.additions_count }}</span>
          <span v-if="node.fileEntry.file.deletions_count > 0" class="text-error font-bold">-{{ node.fileEntry.file.deletions_count }}</span>
        </div>
      </div>
    </template>
  </div>
</template>
