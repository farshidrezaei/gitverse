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
  openReflog: []
  openBisect: []
  openExportPatch: []
  openApplyPatch: []
  openSubmodules: []
}>()

const { t } = useI18n()
const { openInEditor, openInTerminal, openInFileManager } = useExternalToolsController()

const patchMenuItems = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: t('patch.exportTitle'),
      icon: 'i-lucide-file-diff',
      onSelect: () => emit('openExportPatch'),
    },
    {
      label: t('patch.applyTitle'),
      icon: 'i-lucide-file-input',
      onSelect: () => emit('openApplyPatch'),
    },
  ],
])

const externalEditorMenuItems = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: 'Visual Studio Code',
      icon: 'i-lucide-code-2',
      onSelect: () => openInEditor('vscode'),
    },
    {
      label: 'Cursor',
      icon: 'i-lucide-sparkles',
      onSelect: () => openInEditor('cursor'),
    },
    {
      label: 'Zed',
      icon: 'i-lucide-file-code',
      onSelect: () => openInEditor('zed'),
    },
    {
      label: 'Sublime Text',
      icon: 'i-lucide-file-text',
      onSelect: () => openInEditor('sublime'),
    },
    {
      label: 'IntelliJ IDEA',
      icon: 'i-lucide-layers',
      onSelect: () => openInEditor('intellij'),
    },
  ],
])

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
        label: t('externalTools.openTerminal'),
        icon: 'i-lucide-terminal',
        onSelect: () => openInTerminal(),
      },
      {
        label: t('externalTools.openFileManager'),
        icon: 'i-lucide-folder-open',
        onSelect: () => openInFileManager(),
      },
    ],
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
    const recentItems: DropdownMenuItem[] = [
      {
        label: t('repository.recentRepositories'),
        type: 'label' as const,
      },
      ...properties.recentRepositories.map(record => {
        const isOpen = alreadyOpenPaths.has(record.path)
        return {
          label: record.name,
          description: record.is_missing ? `${record.path} (${t('repository.missing')})` : record.path,
          icon: record.is_missing ? 'i-lucide-folder-x' : isOpen ? 'i-lucide-check-circle-2' : 'i-lucide-folder-git-2',
          color: record.is_missing ? ('error' as const) : isOpen ? ('primary' as const) : undefined,
          disabled: record.is_missing,
          onSelect: () => {
            if (!record.is_missing) {
              emit('openRecentRepository', record.path)
            }
          },
        }
      }),
    ]
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
      <UDropdownMenu :items="newRepositoryDropdownItems" :content="{ align: 'start' }">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-plus"
          trailing-icon="i-lucide-chevron-down"
          class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
          :title="t('tabs.openNewWorkspaceTab')"
        />
      </UDropdownMenu>
    </div>

    <!-- Quick External Tools Launchers -->
    <div v-if="properties.activeRepositoryPath" class="flex items-center gap-1 shrink-0 ps-2 border-s border-default/70">
      <UTooltip :text="t('reflog.title')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-history"
          class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
          :aria-label="t('reflog.title')"
          @click="emit('openReflog')"
        />
      </UTooltip>

      <UTooltip :text="t('bisect.modalTitle')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-crosshair"
          class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
          :aria-label="t('bisect.modalTitle')"
          @click="emit('openBisect')"
        />
      </UTooltip>

      <UDropdownMenu :items="patchMenuItems" :content="{ align: 'end' }">
        <UTooltip :text="t('patch.exportTitle')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-file-diff"
            class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
            :aria-label="t('patch.exportTitle')"
          />
        </UTooltip>
      </UDropdownMenu>

      <UTooltip :text="t('submodule.modalTitle')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-folder-git-2"
          class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
          :aria-label="t('submodule.modalTitle')"
          @click="emit('openSubmodules')"
        />
      </UTooltip>

      <UTooltip :text="t('externalTools.openTerminal')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-terminal"
          class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
          :aria-label="t('externalTools.openTerminal')"
          @click="openInTerminal"
        />
      </UTooltip>

      <UDropdownMenu :items="externalEditorMenuItems" :content="{ align: 'end' }">
        <UTooltip :text="t('externalTools.openEditor')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-code-2"
            trailing-icon="i-lucide-chevron-down"
            class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
            :aria-label="t('externalTools.openEditor')"
          />
        </UTooltip>
      </UDropdownMenu>

      <UTooltip :text="t('externalTools.openFileManager')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-folder-open"
          class="h-7 px-1.5 text-muted hover:text-highlighted shrink-0"
          :aria-label="t('externalTools.openFileManager')"
          @click="() => openInFileManager()"
        />
      </UTooltip>
    </div>
  </nav>
</template>
