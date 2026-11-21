<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { WorkspaceDiffTab, WorkspacePullRequestTab, WorkspaceWorkflowRunTab } from '~/types/workspaceTab'
import SharedFileStatusBadge from '~/components/shared/FileStatusBadge.vue'

interface WorkspaceDiffTabBarProperties {
  tabs: WorkspaceDiffTab[]
  activeTabId: string | null
  pullRequestTabs?: WorkspacePullRequestTab[]
  activePullRequestTabId?: string | null
  workflowRunTabs?: WorkspaceWorkflowRunTab[]
  activeWorkflowRunTabId?: string | null
}

const properties = withDefaults(defineProps<WorkspaceDiffTabBarProperties>(), {
  pullRequestTabs: () => [],
  activePullRequestTabId: null,
  workflowRunTabs: () => [],
  activeWorkflowRunTabId: null,
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
  selectWorkflowRunTab: [tabIdentifier: string]
  closeWorkflowRunTab: [tabIdentifier: string]
}>()

const { t } = useI18n()

const isOverviewActive = computed<boolean>(() => {
  return (
    properties.activeTabId === null &&
    properties.activePullRequestTabId === null &&
    properties.activeWorkflowRunTabId === null
  )
})

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
    class="flex h-9 w-full shrink-0 items-center justify-between border-b border-default/70 bg-[var(--gv-surface)]/50 px-2 select-none text-xs"
    aria-label="Workspace Content Tabs"
  >
    <!-- Left: Pinned Permanent Overview Tab + Dynamic Diff & PR Tabs -->
    <div class="flex items-center gap-1 overflow-x-auto no-scrollbar min-w-0 flex-1 h-full py-0.5">
      <!-- Tab 1: Pinned Permanent Root Tab (Overview / Graph Dashboard - NEVER CLOSEABLE) -->
      <button
        type="button"
        class="group relative flex h-7 items-center gap-1.5 rounded-[var(--gv-radius-md)] px-2.5 text-xs transition-all shrink-0 border cursor-pointer"
        :class="[
          isOverviewActive
            ? 'bg-[var(--gv-surface)] text-highlighted border-default shadow-2xs font-semibold'
            : 'border-transparent text-muted hover:bg-muted/30 hover:text-default font-medium',
        ]"
        :title="t('tabs.workspace')"
        @click="emit('selectWorkbench')"
      >
        <UIcon name="i-lucide-git-graph" class="h-3.5 w-3.5 text-primary shrink-0" />
        <span>{{ t('tabs.workspace') }}</span>
        <UIcon
          name="i-lucide-pin"
          class="h-2.5 w-2.5 text-muted/60 ms-0.5 shrink-0"
          :title="t('tabs.pin')"
        />
      </button>

      <div class="h-4 w-px bg-default/50 mx-0.5 shrink-0" aria-hidden="true" />

      <!-- Pull Request Tabs -->
      <div
        v-for="prTab in properties.pullRequestTabs"
        :key="prTab.id"
        class="group relative flex h-7 items-center gap-1.5 rounded-[var(--gv-radius-md)] px-2.5 text-xs font-medium transition-all cursor-pointer border shrink-0"
        :class="[
          properties.activePullRequestTabId === prTab.id
            ? 'bg-[var(--gv-surface)] text-highlighted border-primary/50 shadow-2xs'
            : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
        ]"
        :title="prTab.title"
        @click="emit('selectPullRequestTab', prTab.id)"
      >
        <UIcon name="i-lucide-git-pull-request" class="h-3.5 w-3.5 shrink-0 text-primary" />
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
          class="ms-0.5 inline-flex h-4 w-4 items-center justify-center rounded-xs text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none cursor-pointer"
          :title="t('tabs.close')"
          @click.stop="emit('closePullRequestTab', prTab.id)"
        >
          <UIcon name="i-lucide-x" class="h-3 w-3" />
        </button>
      </div>

      <div
        v-if="properties.pullRequestTabs.length > 0 && (properties.workflowRunTabs.length > 0 || properties.tabs.length > 0)"
        class="h-4 w-px bg-default/50 mx-0.5 shrink-0"
        aria-hidden="true"
      />

      <!-- Workflow Run / Actions Tabs -->
      <div
        v-for="runTab in properties.workflowRunTabs"
        :key="runTab.id"
        class="group relative flex h-7 items-center gap-1.5 rounded-[var(--gv-radius-md)] px-2.5 text-xs font-medium transition-all cursor-pointer border shrink-0"
        :class="[
          properties.activeWorkflowRunTabId === runTab.id
            ? 'bg-[var(--gv-surface)] text-highlighted border-primary/50 shadow-2xs'
            : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
        ]"
        :title="runTab.title"
        @click="emit('selectWorkflowRunTab', runTab.id)"
      >
        <UIcon name="i-lucide-play-circle" class="h-3.5 w-3.5 shrink-0 text-primary" />
        <span class="truncate max-w-[160px] font-medium">
          {{ runTab.title }}
        </span>

        <UBadge
          size="xs"
          :color="runTab.workflowRun.conclusion === 'success' ? 'success' : runTab.workflowRun.conclusion === 'failure' ? 'error' : 'neutral'"
          variant="subtle"
          class="text-[9px] px-1 py-0 leading-none"
        >
          {{ runTab.workflowRun.conclusion || runTab.workflowRun.status }}
        </UBadge>

        <button
          type="button"
          class="ms-0.5 inline-flex h-4 w-4 items-center justify-center rounded-xs text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none cursor-pointer"
          :title="t('tabs.close')"
          @click.stop="emit('closeWorkflowRunTab', runTab.id)"
        >
          <UIcon name="i-lucide-x" class="h-3 w-3" />
        </button>
      </div>

      <div
        v-if="properties.workflowRunTabs.length > 0 && properties.tabs.length > 0"
        class="h-4 w-px bg-default/50 mx-0.5 shrink-0"
        aria-hidden="true"
      />

      <!-- Dynamic Diff Tabs (Single-click = Preview italic, Double-click/Pin = Pinned) -->
      <UContextMenu
        v-for="tab in properties.tabs"
        :key="tab.id"
        :items="getDiffContextMenu(tab)"
        class="shrink-0"
      >
        <div
          class="group relative flex h-7 items-center gap-1.5 rounded-[var(--gv-radius-md)] px-2.5 text-xs font-medium transition-all cursor-pointer border shrink-0"
          :class="[
            properties.activeTabId === tab.id
              ? 'bg-[var(--gv-surface)] text-highlighted border-default shadow-2xs font-semibold'
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
            class="h-3.5 w-3.5 shrink-0 text-muted group-hover:text-default"
          />

          <span class="truncate max-w-[150px]">
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

          <!-- Pin indicator for pinned tabs / Pin button for preview tabs -->
          <button
            v-if="!tab.isPinned"
            type="button"
            class="ms-0.5 inline-flex h-3.5 w-3.5 items-center justify-center rounded-xs text-muted opacity-0 group-hover:opacity-70 hover:opacity-100 hover:text-primary transition-all cursor-pointer"
            :title="t('tabs.pin')"
            @click.stop="emit('pinDiffTab', tab.id)"
          >
            <UIcon name="i-lucide-pin" class="h-2.5 w-2.5" />
          </button>

          <!-- Close Diff Tab Button -->
          <button
            type="button"
            class="ms-0.5 inline-flex h-4 w-4 items-center justify-center rounded-xs text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none cursor-pointer"
            :title="t('tabs.close')"
            @click.stop="emit('closeDiffTab', tab.id)"
          >
            <UIcon name="i-lucide-x" class="h-3 w-3" />
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
          class="h-6 w-6 opacity-70 hover:opacity-100"
          :aria-label="t('tabs.closeAllDiffs')"
          @click="emit('closeAllDiffTabs')"
        />
      </UTooltip>
    </div>
  </nav>
</template>
