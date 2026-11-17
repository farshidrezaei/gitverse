<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { CommitChangedFile } from '~/types/repository'
import { buildFileTree } from '~/utils/fileTree'
import FileStatusBadge from '~/components/shared/FileStatusBadge.vue'
import CommitFileTreeNodeItem from './CommitFileTreeNodeItem.vue'

interface CommitChangedFileListProps {
  files: CommitChangedFile[]
  selectedFilePath: string | null
}

const props = defineProps<CommitChangedFileListProps>()

const emit = defineEmits<{
  selectFile: [filePath: string]
  openDiffTab: [filePath: string]
}>()

const { t } = useI18n()
const filterQuery = ref<string>('')
const viewMode = ref<'flat' | 'tree'>('tree')

const filteredFiles = computed(() => {
  const query = filterQuery.value.trim().toLowerCase()
  if (!query) {
    return props.files
  }
  return props.files.filter(
    (file) =>
      file.path.toLowerCase().includes(query) ||
      file.original_path?.toLowerCase().includes(query)
  )
})

const treeNodes = computed(() => {
  return buildFileTree(filteredFiles.value)
})

function getFlatFileContextMenuItems(filePath: string): ContextMenuItem[][] {
  return [
    [
      {
        label: t('git.copyPath'),
        icon: 'i-lucide-copy',
        onSelect: async () => {
          await navigator.clipboard.writeText(filePath)
        },
      },
    ],
  ]
}
</script>

<template>
  <div class="flex-1 flex flex-col min-h-0">
    <!-- Filter bar with View Mode toggle (Flat vs Tree) -->
    <div class="flex items-center gap-2 p-2 border-b border-default/70 bg-surface/20">
      <UInput
        v-model="filterQuery"
        size="xs"
        icon="i-lucide-search"
        :placeholder="t('commit.filterFilesPlaceholder')"
        class="flex-1"
      />

      <!-- Segmented View Mode Switcher -->
      <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs shrink-0">
        <UTooltip :text="t('repository.viewFlat')">
          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="viewMode === 'flat'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :aria-label="t('repository.viewFlat')"
            @click="viewMode = 'flat'"
          >
            <UIcon name="i-lucide-list" class="h-3 w-3" />
            <span>{{ t('repository.viewFlatShort') }}</span>
          </button>
        </UTooltip>
        <UTooltip :text="t('repository.viewTree')">
          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="viewMode === 'tree'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :aria-label="t('repository.viewTree')"
            @click="viewMode = 'tree'"
          >
            <UIcon name="i-lucide-folder-tree" class="h-3 w-3" />
            <span>{{ t('repository.viewTreeShort') }}</span>
          </button>
        </UTooltip>
      </div>
    </div>

    <!-- Scrollable file list or tree container -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-if="filteredFiles.length === 0"
        class="p-4 text-center text-xs text-muted"
      >
        {{ t('common.empty') }}
      </div>

      <!-- Tree View Mode -->
      <div
        v-else-if="viewMode === 'tree'"
        class="p-2 space-y-0.5"
      >
        <CommitFileTreeNodeItem
          v-for="node in treeNodes"
          :key="node.id"
          :node="node"
          :selected-file-path="props.selectedFilePath"
          @select-file="emit('selectFile', $event)"
          @open-diff-tab="emit('openDiffTab', $event)"
        />
      </div>

      <!-- Flat View Mode -->
      <div
        v-else
        class="divide-y divide-default/20"
      >
        <UContextMenu
          v-for="file in filteredFiles"
          :key="file.path"
          :items="getFlatFileContextMenuItems(file.path)"
        >
          <button
            type="button"
            class="w-full text-start flex items-center justify-between px-3 py-1.5 hover:bg-muted/15 transition-colors group select-none text-xs"
            :class="{
              'bg-primary/10 text-primary font-medium': props.selectedFilePath === file.path,
            }"
            @click="emit('selectFile', file.path)"
            @dblclick="emit('openDiffTab', file.path)"
          >
            <div class="flex items-center gap-2 min-w-0 flex-1 me-2">
              <FileStatusBadge :status="file.status" size="xs" />
              <span class="font-mono text-[11px] truncate text-highlighted group-hover:text-default" :title="file.path">
                {{ file.path }}
              </span>
            </div>

            <!-- Additions / Deletions count -->
            <div class="flex items-center gap-1 shrink-0 font-mono text-[10px]">
              <span v-if="file.additions > 0" class="text-success">
                +{{ file.additions }}
              </span>
              <span v-if="file.deletions > 0" class="text-error">
                -{{ file.deletions }}
              </span>
            </div>
          </button>
        </UContextMenu>
      </div>
    </div>
  </div>
</template>
