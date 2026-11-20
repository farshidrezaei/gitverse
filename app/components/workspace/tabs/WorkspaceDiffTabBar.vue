<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { WorkspaceDiffTab, WorkspacePullRequestTab } from '~/types/workspaceTab'
import SharedFileStatusBadge from '~/components/shared/FileStatusBadge.vue'

interface WorkspaceDiffTabBarProperties {
  tabs: WorkspaceDiffTab[]
  activeTabId: string | null
  pullRequestTabs?: WorkspacePullRequestTab[]
  activePullRequestTabId?: string | null
}

const properties = withDefaults(defineProps<WorkspaceDiffTabBarProperties>(), {
  pullRequestTabs: () => [],
  activePullRequestTabId: null,
})

const emit = defineEmits<{
  selectDiffTab: [tabIdentifier: string]
  selectWorkbench: []
  closeDiffTab: [tabIdentifier: string]
  closeOtherDiffTabs: [tabIdentifier: string]
  closeAllDiffTabs: []
  pinDiffTab: [tabIdentifier: string]
  selectPullRequestTab: [tabIdentifier: string]
  closePullRequestTab: [tabIdentifier: string]
}>()

const { t } = useI18n()

function handleMiddleClick(event: MouseEvent, tab: WorkspaceDiffTab): void {
  if (event.button === 1 && tab.isClosable) {
    event.preventDefault()
    emit('closeDiffTab', tab.id)
  }
}

function handleDoubleClick(tab: WorkspaceDiffTab): void {
  if (!tab.isPinned) {
    emit('pinDiffTab', tab.id)
  }
}

function getDiffContextMenu(tab: WorkspaceDiffTab): ContextMenuItem[][] {
  const items: ContextMenuItem[] = []

  if (!tab.isPinned) {
    items.push({
      label: t('tabs.pin'),
      icon: 'i-lucide-pin',
      onSelect: () => emit('pinDiffTab', tab.id),
    })
  }

  items.push({
    label: t('tabs.close'),
    icon: 'i-lucide-x',
    onSelect: () => emit('closeDiffTab', tab.id),
  })

  items.push(
    {
      label: t('tabs.closeOthers'),
      icon: 'i-lucide-shrink',
      onSelect: () => emit('closeOtherDiffTabs', tab.id),
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
    class="flex h-8 w-full shrink-0 items-center justify-between border-b border-default/70 bg-[var(--gv-surface)]/40 px-2 select-none text-xs"
    aria-label="Diff and Pull Request Tabs"
  >
    <!-- Left: Workbench Return Button + Diff Tabs + Pull Request Tabs -->
    <div class="flex items-center gap-1 overflow-x-auto no-scrollbar min-w-0 flex-1 h-full py-0.5">
      <!-- Workbench Root Tab (Workspace overview) -->
      <button
        type="button"
        class="flex h-6 items-center gap-1.5 rounded px-2 text-xs font-medium transition-all shrink-0 border"
        :class="[
          properties.activeTabId === null && properties.activePullRequestTabId === null
            ? 'bg-[var(--gv-surface)] text-highlighted border-default shadow-xs font-semibold'
            : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
        ]"
        @click="emit('selectWorkbench')"
      >
        <UIcon name="i-lucide-layout-dashboard" class="h-3 w-3 text-primary" />
        <span>{{ t('tabs.workspace') }}</span>
      </button>

      <div class="h-3 w-px bg-default/40 mx-0.5 shrink-0" aria-hidden="true" />

      <!-- Pull Request Tabs -->
      <div
        v-for="prTab in properties.pullRequestTabs"
        :key="prTab.id"
        class="group relative flex h-6 items-center gap-1.5 rounded px-2 text-xs font-medium transition-all cursor-pointer border shrink-0"
        :class="[
          properties.activePullRequestTabId === prTab.id
            ? 'bg-[var(--gv-surface)] text-highlighted border-primary/50 shadow-xs'
            : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
        ]"
        :title="prTab.title"
        @click="emit('selectPullRequestTab', prTab.id)"
      >
        <UIcon name="i-lucide-git-pull-request" class="h-3 w-3 shrink-0 text-primary" />
        <span class="truncate max-w-[160px] font-medium">
          #{{ prTab.pullRequest.number }} {{ prTab.pullRequest.title }}
        </span>

        <UBadge
          size="xs"
          :color="prTab.pullRequest.state === 'open' ? 'success' : prTab.pullRequest.state === 'merged' ? 'primary' : 'neutral'"
          variant="subtle"
          class="text-[9px] px-1 py-0 leading-none"
        >
          {{ prTab.pullRequest.state }}
        </UBadge>

        <button
          type="button"
          class="ms-0.5 inline-flex h-3.5 w-3.5 items-center justify-center rounded-xs text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none"
          :title="t('tabs.close')"
          @click.stop="emit('closePullRequestTab', prTab.id)"
        >
          <UIcon name="i-lucide-x" class="h-2.5 w-2.5" />
        </button>
      </div>

      <div
        v-if="properties.pullRequestTabs.length > 0 && properties.tabs.length > 0"
        class="h-3 w-px bg-default/40 mx-0.5 shrink-0"
        aria-hidden="true"
      />

      <!-- Active Diff Tabs -->
      <UContextMenu
        v-for="tab in properties.tabs"
        :key="tab.id"
        :items="getDiffContextMenu(tab)"
        class="shrink-0"
      >
        <div
          class="group relative flex h-6 items-center gap-1.5 rounded px-2 text-xs font-medium transition-all cursor-pointer border shrink-0"
          :class="[
            properties.activeTabId === tab.id
              ? 'bg-[var(--gv-surface)] text-highlighted border-default shadow-xs'
              : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
            !tab.isPinned ? 'italic opacity-90' : '',
          ]"
          :title="tab.filePath"
          @click="emit('selectDiffTab', tab.id)"
          @auxclick="handleMiddleClick($event, tab)"
          @dblclick="handleDoubleClick(tab)"
        >
          <SharedFileStatusBadge
            v-if="tab.changeStatus"
            :status="tab.changeStatus"
            size="xs"
          />
          <UIcon
            v-else
            name="i-lucide-file-code"
            class="h-3 w-3 shrink-0 text-muted group-hover:text-default"
          />

          <span class="truncate max-w-[140px]">
            {{ tab.title }}
          </span>

          <!-- Staged / Commit indicator -->
          <span
            v-if="tab.isStaged"
            class="text-[8px] px-1 py-0.2 rounded bg-success/15 text-success font-semibold shrink-0"
          >
            {{ t('tabs.staged') }}
          </span>
          <span
            v-else-if="tab.commitHash"
            class="text-[8px] px-1 py-0.2 rounded bg-primary/15 text-primary font-mono shrink-0"
          >
            {{ tab.commitHash.substring(0, 6) }}
          </span>

          <!-- Close Diff Tab Button -->
          <button
            type="button"
            class="ms-0.5 inline-flex h-3.5 w-3.5 items-center justify-center rounded-xs text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none"
            :title="t('tabs.close')"
            @click.stop="emit('closeDiffTab', tab.id)"
          >
            <UIcon name="i-lucide-x" class="h-2.5 w-2.5" />
          </button>
        </div>
      </UContextMenu>
    </div>

    <!-- Right: Close All Diff Tabs Action -->
    <div
      v-if="properties.tabs.length > 0"
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

