<script setup lang="ts">
import type { DropdownMenuItem, ContextMenuItem } from '@nuxt/ui'
import type { WorkspaceRepositoryTab } from '~/types/workspaceTab'
import type { RecentRepositoryRecord } from '~/types/repository'

interface WorkspaceRepositoryTabBarProperties {
  tabs: WorkspaceRepositoryTab[]
  activeRepositoryPath: string
  recentRepositories: RecentRepositoryRecord[]
}

const properties = defineProps<WorkspaceRepositoryTabBarProperties>()

const emit = defineEmits<{
  selectRepository: [repositoryPath: string]
  closeRepository: [repositoryPath: string]
  closeOtherRepositories: [repositoryPath: string]
  openRepositoryPrompt: []
  openRecentRepository: [repositoryPath: string]
}>()

const { t } = useI18n()

function handleMiddleClick(event: MouseEvent, repositoryPath: string): void {
  if (event.button === 1) {
    event.preventDefault()
    emit('closeRepository', repositoryPath)
  }
}

function getRepositoryContextMenu(tab: WorkspaceRepositoryTab): ContextMenuItem[][] {
  return [
    [
      {
        label: t('repository.closeRepository'),
        icon: 'i-lucide-x',
        onSelect: () => emit('closeRepository', tab.repositoryPath),
      },
      {
        label: t('tabs.closeOthers'),
        icon: 'i-lucide-shrink',
        onSelect: () => emit('closeOtherRepositories', tab.repositoryPath),
      },
    ],
  ]
}

const newRepositoryDropdownItems = computed<DropdownMenuItem[][]>(() => {
  const items: DropdownMenuItem[][] = [
    [
      {
        label: t('repository.openRepository'),
        icon: 'i-lucide-folder-open',
        onSelect: () => emit('openRepositoryPrompt'),
      },
    ],
  ]

  const alreadyOpenPaths = new Set(properties.tabs.map(tab => tab.repositoryPath))

  if (properties.recentRepositories.length > 0) {
    const recentItems: DropdownMenuItem[] = properties.recentRepositories.map(record => ({
      label: record.name,
      description: record.path,
      icon: 'i-lucide-git-fork',
      color: alreadyOpenPaths.has(record.path) ? ('primary' as const) : undefined,
      onSelect: () => emit('openRecentRepository', record.path),
    }))
    items.push(recentItems)
  }

  return items
})
</script>

<template>
  <nav
    class="flex h-9 w-full shrink-0 items-center justify-between border-b border-default bg-[var(--gv-sidebar)] px-2 select-none text-xs"
    aria-label="Repository Tabs"
  >
    <!-- Scrollable Repository Tabs Strip -->
    <div class="flex items-center gap-1.5 overflow-x-auto no-scrollbar min-w-0 flex-1 h-full py-1">
      <UContextMenu
        v-for="tab in properties.tabs"
        :key="tab.repositoryPath"
        :items="getRepositoryContextMenu(tab)"
        class="shrink-0"
      >
        <div
          class="group relative flex h-7 items-center gap-1.5 rounded-md px-2.5 text-xs font-semibold transition-all cursor-pointer border shrink-0"
          :class="[
            properties.activeRepositoryPath === tab.repositoryPath
              ? 'bg-[var(--gv-surface)] text-highlighted border-default shadow-xs'
              : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
          ]"
          :title="tab.repositoryPath"
          @click="emit('selectRepository', tab.repositoryPath)"
          @auxclick="handleMiddleClick($event, tab.repositoryPath)"
        >
          <UIcon
            name="i-lucide-folder-git-2"
            class="h-3.5 w-3.5 shrink-0"
            :class="properties.activeRepositoryPath === tab.repositoryPath ? 'text-primary' : 'text-muted'"
          />

          <span class="truncate max-w-[140px]">
            {{ tab.repositoryName }}
          </span>

          <!-- Close Repository Tab Button -->
          <button
            type="button"
            class="ms-1 inline-flex h-4 w-4 items-center justify-center rounded-sm text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none"
            :title="t('repository.closeRepository')"
            @click.stop="emit('closeRepository', tab.repositoryPath)"
          >
            <UIcon name="i-lucide-x" class="h-3 w-3" />
          </button>
        </div>
      </UContextMenu>

      <!-- Dropdown button to open another repository -->
      <UDropdownMenu :items="newRepositoryDropdownItems">
        <button
          type="button"
          class="flex h-7 items-center gap-1 rounded-md border border-dashed border-default/80 px-2 text-muted transition-colors hover:border-primary/50 hover:bg-muted/30 hover:text-highlighted focus:outline-none shrink-0"
          :title="t('tabs.openNewWorkspaceTab')"
        >
          <UIcon name="i-lucide-plus" class="h-3.5 w-3.5" />
          <UIcon name="i-lucide-chevron-down" class="h-3 w-3 text-muted/70" />
        </button>
      </UDropdownMenu>
    </div>
  </nav>
</template>
