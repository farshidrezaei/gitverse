<script setup lang="ts">
import type { DropdownMenuItem, ContextMenuItem } from '@nuxt/ui'
import type { WorkspaceRepositoryTab } from '~/types/workspaceTab'
import { useWorkspaceTabController } from '~/composables/useWorkspaceTabController'
import { useWorkspaceManager } from '~/composables/useWorkspaceManager'
import { useWorkspaceModals } from '~/composables/useWorkspaceModals'

const { t } = useI18n()
const {
  repositoryTabs,
  activeRepositoryPath,
  activateRepositoryTab,
  closeRepositoryTab,
  closeOtherRepositoryTabs,
  openRepositoryTab,
} = useWorkspaceTabController()

const {
  recentRepositories,
  openRepository,
  closeRepository,
  promptAndOpenRepository,
} = useWorkspaceManager()

const {
  openFolderPicker,
  openReflog,
  openBisect,
  openExportPatch,
  openApplyPatch,
  openSubmodules,
  openInsights,
  openGitBundle,
} = useWorkspaceModals()

async function handleSelectRepository(repositoryPath: string): Promise<void> {
  activateRepositoryTab(repositoryPath)
  await openRepository(repositoryPath)
}

async function handleCloseRepository(repositoryPath: string): Promise<void> {
  closeRepositoryTab(repositoryPath)
  if (repositoryTabs.value.length === 0) {
    await closeRepository()
  } else if (activeRepositoryPath.value) {
    await openRepository(activeRepositoryPath.value)
  }
}

function handleMiddleClick(event: MouseEvent, repositoryPath: string): void {
  if (event.button === 1) {
    event.preventDefault()
    void handleCloseRepository(repositoryPath)
  }
}

function getRepositoryContextMenu(tab: WorkspaceRepositoryTab): ContextMenuItem[][] {
  return [
    [
      {
        label: t('repository.closeRepository'),
        icon: 'i-lucide-x',
        onSelect: () => void handleCloseRepository(tab.repositoryPath),
      },
      {
        label: t('tabs.closeOthers'),
        icon: 'i-lucide-shrink',
        disabled: repositoryTabs.value.length <= 1,
        onSelect: () => closeOtherRepositoryTabs(tab.repositoryPath),
      },
    ],
  ]
}

const newRepositoryDropdownItems = computed<DropdownMenuItem[][]>(() => {
  const items: DropdownMenuItem[][] = [
    [
      {
        label: t('repository.openExisting'),
        icon: 'i-lucide-folder-open',
        onSelect: async () => {
          const openedRepository = await promptAndOpenRepository()
          if (openedRepository) {
            openRepositoryTab(openedRepository.path, openedRepository.name)
          }
        },
      },
    ],
  ]

  if (recentRepositories.value.length > 0) {
    const recentItems: DropdownMenuItem[] = recentRepositories.value.slice(0, 5).map((recent) => ({
      label: recent.name,
      icon: 'i-lucide-history',
      onSelect: () => void handleSelectRepository(recent.path),
    }))
    items.push(recentItems)
  }

  return items
})

const gitToolsDropdownItems = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: t('reflog.title'),
      icon: 'i-lucide-history',
      onSelect: () => openReflog(),
    },
    {
      label: t('bisect.title'),
      icon: 'i-lucide-git-commit',
      onSelect: () => openBisect(),
    },
    {
      label: t('insights.title'),
      icon: 'i-lucide-bar-chart-2',
      onSelect: () => openInsights(),
    },
    {
      label: t('submodule.title'),
      icon: 'i-lucide-folder-git',
      onSelect: () => openSubmodules(),
    },
  ],
  [
    {
      label: t('patch.exportTitle'),
      icon: 'i-lucide-file-diff',
      onSelect: () => openExportPatch(),
    },
    {
      label: t('patch.applyTitle'),
      icon: 'i-lucide-file-input',
      onSelect: () => openApplyPatch(),
    },
    {
      label: t('archive.bundleModalTitle'),
      icon: 'i-lucide-package',
      onSelect: () => openGitBundle(),
    },
  ],
])
</script>

<template>
  <div class="flex items-center gap-1 min-w-0 max-w-xl h-full py-0.5">
    <!-- Scrollable Repository Tabs -->
    <div class="flex items-center gap-1 overflow-x-auto no-scrollbar min-w-0 flex-1 h-full">
      <UContextMenu
        v-for="tab in repositoryTabs"
        :key="tab.repositoryPath"
        :items="getRepositoryContextMenu(tab)"
        class="shrink-0"
      >
        <div
          class="group relative flex h-7 items-center gap-1.5 rounded-md px-2 text-xs font-medium transition-all cursor-pointer border shrink-0"
          :class="[
            activeRepositoryPath === tab.repositoryPath
              ? 'bg-[var(--gv-surface)] text-highlighted border-default shadow-2xs font-semibold'
              : 'border-transparent text-muted hover:bg-muted/30 hover:text-default',
          ]"
          :title="tab.repositoryPath"
          @click="handleSelectRepository(tab.repositoryPath)"
          @auxclick="handleMiddleClick($event, tab.repositoryPath)"
        >
          <UIcon
            name="i-lucide-folder-git-2"
            class="h-3.5 w-3.5 shrink-0"
            :class="activeRepositoryPath === tab.repositoryPath ? 'text-primary' : 'text-muted'"
          />

          <span class="truncate max-w-[120px]">
            {{ tab.repositoryName }}
          </span>

          <!-- Close Repository Tab Button -->
          <button
            type="button"
            class="ms-0.5 inline-flex h-3.5 w-3.5 items-center justify-center rounded-xs text-muted opacity-60 transition-opacity hover:bg-muted/40 hover:text-highlighted hover:opacity-100 focus:outline-none cursor-pointer"
            :title="t('repository.closeRepository')"
            @click.stop="handleCloseRepository(tab.repositoryPath)"
          >
            <UIcon name="i-lucide-x" class="h-2.5 w-2.5" />
          </button>
        </div>
      </UContextMenu>

      <!-- New Repository '+' Dropdown Button -->
      <UDropdownMenu :items="newRepositoryDropdownItems" :content="{ align: 'start' }">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-plus"
          class="h-6 w-6 p-0 text-muted hover:text-highlighted shrink-0"
          :title="t('tabs.openNewWorkspaceTab')"
        />
      </UDropdownMenu>
    </div>

    <!-- Git Tools Dropdown Menu (If an active repository exists) -->
    <div v-if="activeRepositoryPath" class="shrink-0 ps-1">
      <UDropdownMenu :items="gitToolsDropdownItems" :content="{ align: 'end' }">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-wrench"
          class="h-6 w-6 p-0 text-muted hover:text-highlighted"
          :title="t('git.tools')"
        />
      </UDropdownMenu>
    </div>
  </div>
</template>
