<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { FileTreeNode } from '~/utils/fileTree'
import FileStatusBadge from '~/components/shared/FileStatusBadge.vue'

const props = defineProps<{
  node: FileTreeNode
  isStaged: boolean
  selectedPath?: string | null
  selectedIsStaged?: boolean | null
}>()

const emit = defineEmits<{
  stage: [path: string]
  unstage: [path: string]
  discard: [path: string, isUntracked: boolean]
  select: [path: string, isStaged: boolean]
  openDiffTab: [path: string, isStaged: boolean]
}>()

const { t } = useI18n()
const isOpen = ref(true)

function toggleOpen(): void {
  isOpen.value = !isOpen.value
}

const fileContextMenuItems = computed<ContextMenuItem[][]>(() => {
  if (props.node.isDirectory) return []

  const isStaged = props.isStaged

  return [
    [
      isStaged
        ? {
            label: t('git.unstage'),
            icon: 'i-lucide-minus',
            onSelect: () => emit('unstage', props.node.fullPath),
          }
        : {
            label: t('git.stage'),
            icon: 'i-lucide-plus',
            onSelect: () => emit('stage', props.node.fullPath),
          },
      ...(isStaged
        ? []
        : [
            {
              label: t('diff.discardFile'),
              icon: 'i-lucide-undo-2',
              color: 'error' as const,
              onSelect: () => emit('discard', props.node.fullPath, props.node.fileEntry?.status === 'untracked'),
            },
          ]),
    ],
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
    <div v-if="node.isDirectory && isOpen" class="ps-3 border-s border-default/40 ms-2 my-0.5 space-y-0.5">
      <WorkspaceStagingTreeNodeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :is-staged="isStaged"
        :selected-path="selectedPath"
        :selected-is-staged="selectedIsStaged"
        @stage="emit('stage', $event)"
        @unstage="emit('unstage', $event)"
        @discard="(path, isUntracked) => emit('discard', path, isUntracked)"
        @select="(path, staged) => emit('select', path, staged)"
        @open-diff-tab="(path, staged) => emit('openDiffTab', path, staged)"
      />
    </div>

    <!-- File Item Row Wrapped in Context Menu -->
    <UContextMenu
      v-else-if="!node.isDirectory"
      :items="fileContextMenuItems"
    >
      <div
        class="group flex items-center justify-between py-1 px-1.5 rounded-md cursor-pointer transition-colors"
        style="content-visibility: auto; contain-intrinsic-size: 0 24px;"
        :class="selectedPath === node.fullPath && selectedIsStaged === isStaged
          ? 'bg-primary/15 text-primary font-medium'
          : 'hover:bg-muted/40'"
        @click="emit('select', node.fullPath, isStaged)"
        @dblclick="emit('openDiffTab', node.fullPath, isStaged)"
      >
        <div class="flex items-center gap-1.5 min-w-0">
          <FileStatusBadge
            v-if="node.fileEntry?.status"
            :status="node.fileEntry.status"
            size="xs"
          />
          <UIcon name="i-lucide-file-code" class="h-3.5 w-3.5 shrink-0 text-muted" />
          <span class="truncate font-mono text-[11px]" dir="ltr">
            {{ node.name }}
          </span>
        </div>

        <!-- Action buttons -->
        <div class="flex items-center gap-0.5 opacity-60 group-hover:opacity-100 transition-opacity">
          <template v-if="isStaged">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-minus"
              class="h-5 w-5 shrink-0"
              title="Unstage"
              @click.stop="emit('unstage', node.fullPath)"
            />
          </template>
          <template v-else>
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-undo-2"
              class="h-5 w-5 shrink-0"
              title="Discard changes"
              @click.stop="emit('discard', node.fullPath, node.fileEntry?.status === 'untracked')"
            />
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-plus"
              class="h-5 w-5 shrink-0"
              title="Stage"
              @click.stop="emit('stage', node.fullPath)"
            />
          </template>
        </div>
      </div>
    </UContextMenu>
  </div>
</template>
