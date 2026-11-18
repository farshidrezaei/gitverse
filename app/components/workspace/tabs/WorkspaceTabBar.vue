<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { WorkspaceTab, WorkspaceDiffTab } from '~/types/workspaceTab'
import SharedFileStatusBadge from '~/components/shared/FileStatusBadge.vue'

interface WorkspaceTabBarProperties {
  tabs: WorkspaceTab[]
  activeTabId: string
}

const properties = defineProps<WorkspaceTabBarProperties>()

const emit = defineEmits<{
  selectTab: [tabIdentifier: string]
  closeTab: [tabIdentifier: string]
  closeOtherTabs: [tabIdentifier: string]
  closeAllDiffTabs: []
  pinTab: [tabIdentifier: string]
}>()

const { t } = useI18n()

function handleMiddleClick(event: MouseEvent, tab: WorkspaceTab): void {
  if (event.button === 1 && tab.isClosable) {
    event.preventDefault()
    emit('closeTab', tab.id)
  }
}

function handleDoubleClick(tab: WorkspaceTab): void {
  if (tab.type === 'diff' && !tab.isPinned) {
    emit('pinTab', tab.id)
  }
}

function getContextMenuItems(tab: WorkspaceTab): ContextMenuItem[][] {
  const items: ContextMenuItem[] = []

  if (tab.type === 'diff' && !tab.isPinned) {
    items.push({
      label: t('tabs.pin'),
      icon: 'i-lucide-pin',
      onSelect: () => emit('pinTab', tab.id),
    })
  }

  if (tab.isClosable) {
    items.push({
      label: t('tabs.close'),
      icon: 'i-lucide-x',
      onSelect: () => emit('closeTab', tab.id),
    })
  }

  items.push(
    {
      label: t('tabs.closeOthers'),
      icon: 'i-lucide-shrink',
      onSelect: () => emit('closeOtherTabs', tab.id),
    },
    {
      label: t('tabs.closeAllDiffs'),
      icon: 'i-lucide-x-circle',
      color: 'error' as const,
      onSelect: () => emit('closeAllDiffTabs'),
    },
  )

  return [items]
}
</script>

<template>
  <nav
    class="flex h-9 w-full shrink-0 items-center justify-between border-b border-default bg-[var(--gv-sidebar)] px-1.5 select-none text-xs"
    aria-label="Workspace Tabs"
  >
    <!-- Scrollable Tab Strip -->
    <div class="flex items-center gap-1 overflow-x-auto no-scrollbar min-w-0 flex-1 h-full py-1">
      <UContextMenu
        v-for="tab in properties.tabs"
        :key="tab.id"
        :items="getContextMenuItems(tab)"
        class="shrink-0"
      >
        <div
          class="group relative flex h-7 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium transition-all cursor-pointer border shrink-0"
          :class="[
            properties.activeTabId === tab.id
              ? 'bg-[var(--gv-surface)] text-highlighted border-default shadow-xs'
              : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
            tab.type === 'diff' && !tab.isPinned ? 'italic opacity-90' : '',
          ]"
          :title="tab.type === 'diff' ? (tab as WorkspaceDiffTab).filePath : tab.title"
          @click="emit('selectTab', tab.id)"
          @auxclick="handleMiddleClick($event, tab)"
          @dblclick="handleDoubleClick(tab)"
        >
          <!-- Icon / Badge by Tab Type -->
          <template v-if="tab.type === 'workspace'">
            <UIcon
              name="i-lucide-git-branch"
              class="h-3.5 w-3.5 shrink-0 text-primary"
            />
            <span class="truncate max-w-[120px] font-semibold">
              {{ tab.title }}
            </span>
          </template>

          <template v-else>
            <SharedFileStatusBadge
              v-if="(tab as WorkspaceDiffTab).changeStatus"
              :status="(tab as WorkspaceDiffTab).changeStatus!"
              size="xs"
            />
            <UIcon
              v-else
              name="i-lucide-file-code"
              class="h-3.5 w-3.5 shrink-0 text-muted group-hover:text-default"
            />

            <span class="truncate max-w-[150px]">
              {{ tab.title }}
            </span>

            <!-- Staged or Commit Sub-indicator -->
            <span
              v-if="(tab as WorkspaceDiffTab).isStaged"
              class="text-[9px] px-1 py-0.2 rounded bg-success/15 text-success font-semibold shrink-0"
            >
              {{ t('tabs.staged') }}
            </span>
            <span
              v-else-if="(tab as WorkspaceDiffTab).commitHash"
              class="text-[9px] px-1 py-0.2 rounded bg-primary/15 text-primary font-mono shrink-0"
            >
              {{ (tab as WorkspaceDiffTab).commitHash!.substring(0, 6) }}
            </span>
          </template>

          <!-- Close Button (For closable tabs) -->
          <button
            v-if="tab.isClosable"
            type="button"
            class="ms-1 inline-flex h-4 w-4 items-center justify-center rounded-sm text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none"
            :title="t('tabs.close')"
            @click.stop="emit('closeTab', tab.id)"
          >
            <UIcon name="i-lucide-x" class="h-3 w-3" />
          </button>
        </div>
      </UContextMenu>
    </div>

    <!-- Trailing Actions (Close all diffs shortcut) -->
    <div
      v-if="properties.tabs.filter(tab => tab.isClosable).length > 0"
      class="flex items-center ps-2 pe-1 shrink-0"
    >
      <UTooltip :text="t('tabs.closeAllDiffs')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-x-circle"
          class="opacity-70 hover:opacity-100"
          :aria-label="t('tabs.closeAllDiffs')"
          @click="emit('closeAllDiffTabs')"
        />
      </UTooltip>
    </div>
  </nav>
</template>
