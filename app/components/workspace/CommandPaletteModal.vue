<script setup lang="ts">
import type { BranchSummary, RemoteBranchSummary, RecentRepositoryRecord } from '~/types/repository'

interface CommandItem {
  id: string
  title: string
  description?: string
  category: 'actions' | 'branches' | 'repositories' | 'preferences'
  icon: string
  badge?: string
  badgeColor?: 'primary' | 'success' | 'warning' | 'neutral'
  keywords?: string[]
  action: () => void | Promise<void>
}

const isOpen = defineModel<boolean>('open', { default: false })

const emit = defineEmits<{
  openSettings: []
  openFolderPicker: []
  openCreateBranch: []
  openCreateStash: []
}>()

const { t, locale, setLocale } = useI18n()
const {
  currentRepository,
  recentRepositories,
  repositoryReferences,
  fetchRemote,
  pullChanges,
  pushChanges,
  stageAll,
  unstageAll,
  fetchStatus,
  fetchCommitGraph,
  checkoutBranch,
  openRepository,
} = useWorkspaceManager()

const { colorMode } = useThemeController()
const {
  openFolderPicker,
  openReflog,
  openBisect,
  openExportPatch,
  openApplyPatch,
  openSubmodules,
  openInsights,
  openSearch,
  openGitBundle,
  openLfsManager,
  openCustomActions,
} = useWorkspaceModals()

const searchQuery = ref<string>('')
const selectedIndex = ref<number>(0)
const searchInputReference = ref<HTMLInputElement | null>(null)
const itemElements = ref<Record<number, HTMLElement | null>>({})

const allItems = computed<CommandItem[]>(() => {
  const items: CommandItem[] = []

  // 1. Git Actions (when repository is open)
  if (currentRepository.value) {
    items.push(
      {
        id: 'action-fetch',
        title: t('commandPalette.actions.fetch'),
        category: 'actions',
        icon: 'i-lucide-arrow-down-to-dot',
        keywords: ['fetch', 'remote', 'sync', 'دریافت'],
        action: async () => {
          await fetchRemote()
        },
      },
      {
        id: 'action-pull',
        title: t('commandPalette.actions.pull'),
        category: 'actions',
        icon: 'i-lucide-arrow-down',
        keywords: ['pull', 'merge', 'rebase', 'پول'],
        action: async () => {
          await pullChanges()
        },
      },
      {
        id: 'action-push',
        title: t('commandPalette.actions.push'),
        category: 'actions',
        icon: 'i-lucide-arrow-up',
        keywords: ['push', 'publish', 'پوش'],
        action: async () => {
          await pushChanges()
        },
      },
      {
        id: 'action-stage-all',
        title: t('commandPalette.actions.stageAll'),
        category: 'actions',
        icon: 'i-lucide-plus-circle',
        keywords: ['stage', 'add', 'استیج', 'همه'],
        action: async () => {
          await stageAll()
        },
      },
      {
        id: 'action-unstage-all',
        title: t('commandPalette.actions.unstageAll'),
        category: 'actions',
        icon: 'i-lucide-minus-circle',
        keywords: ['unstage', 'restore', 'حذف'],
        action: async () => {
          await unstageAll()
        },
      },
      {
        id: 'action-create-branch',
        title: t('commandPalette.actions.createBranch'),
        category: 'actions',
        icon: 'i-lucide-git-branch-plus',
        keywords: ['branch', 'new', 'شاخه'],
        action: () => {
          emit('openCreateBranch')
        },
      },
      {
        id: 'action-create-stash',
        title: t('commandPalette.actions.createStash'),
        category: 'actions',
        icon: 'i-lucide-archive',
        keywords: ['stash', 'save', 'استش'],
        action: () => {
          emit('openCreateStash')
        },
      },
      {
        id: 'action-refresh',
        title: t('commandPalette.actions.refresh'),
        category: 'actions',
        icon: 'i-lucide-refresh-cw',
        keywords: ['refresh', 'reload', 'تازه‌سازی'],
        action: async () => {
          await fetchStatus()
          await fetchCommitGraph()
        },
      },
      {
        id: 'action-open-reflog',
        title: t('commandPalette.actions.openReflog'),
        category: 'actions',
        icon: 'i-lucide-history',
        keywords: ['reflog', 'history', 'وقایع', 'رفلاگ'],
        action: () => {
          openReflog()
        },
      },
      {
        id: 'action-open-bisect',
        title: t('commandPalette.actions.openBisect'),
        category: 'actions',
        icon: 'i-lucide-git-commit',
        keywords: ['bisect', 'debug', 'باگ', 'بایسکت'],
        action: () => {
          openBisect()
        },
      },
      {
        id: 'action-open-insights',
        title: t('commandPalette.actions.openInsights'),
        category: 'actions',
        icon: 'i-lucide-bar-chart-2',
        keywords: ['insights', 'statistics', 'آمار', 'بینش'],
        action: () => {
          openInsights()
        },
      },
      {
        id: 'action-open-search',
        title: t('commandPalette.actions.openSearch'),
        category: 'actions',
        icon: 'i-lucide-search',
        keywords: ['search', 'find', 'جستجو'],
        action: () => {
          openSearch()
        },
      },
      {
        id: 'action-open-export-patch',
        title: t('commandPalette.actions.openExportPatch'),
        category: 'actions',
        icon: 'i-lucide-file-output',
        keywords: ['patch', 'export', 'پچ'],
        action: () => {
          openExportPatch()
        },
      },
      {
        id: 'action-open-apply-patch',
        title: t('commandPalette.actions.openApplyPatch'),
        category: 'actions',
        icon: 'i-lucide-file-input',
        keywords: ['patch', 'apply', 'اعمال پچ'],
        action: () => {
          openApplyPatch()
        },
      },
      {
        id: 'action-open-submodules',
        title: t('commandPalette.actions.openSubmodules'),
        category: 'actions',
        icon: 'i-lucide-folder-archive',
        keywords: ['submodule', 'زیرماژول'],
        action: () => {
          openSubmodules()
        },
      },
      {
        id: 'action-open-git-bundle',
        title: t('commandPalette.actions.openGitBundle'),
        category: 'actions',
        icon: 'i-lucide-package',
        keywords: ['bundle', 'باندل'],
        action: () => {
          openGitBundle()
        },
      },
      {
        id: 'action-open-lfs',
        title: t('commandPalette.actions.openLfs'),
        category: 'actions',
        icon: 'i-lucide-hard-drive',
        keywords: ['lfs', 'large files', 'فایل حجیم'],
        action: () => {
          openLfsManager()
        },
      },
      {
        id: 'action-open-custom-actions',
        title: t('commandPalette.actions.openCustomActions'),
        category: 'actions',
        icon: 'i-lucide-terminal',
        keywords: ['custom', 'actions', 'دستورات سفارشی'],
        action: () => {
          openCustomActions()
        },
      },
    )

    // 2. Branches (when repository is open)
    const localBranches = repositoryReferences.value?.branches || []
    for (const branch of localBranches) {
      items.push({
        id: `branch-local-${branch.name}`,
        title: branch.name,
        description: branch.upstream_name ? `Tracks ${branch.upstream_name}` : undefined,
        category: 'branches',
        icon: branch.is_head ? 'i-lucide-git-branch-plus' : 'i-lucide-git-branch',
        badge: branch.is_head ? 'HEAD' : undefined,
        badgeColor: branch.is_head ? 'primary' : 'neutral',
        keywords: ['branch', 'checkout', 'switch', branch.name],
        action: async () => {
          if (!branch.is_head) {
            await checkoutBranch(branch.name)
          }
        },
      })
    }

    const remoteBranches = repositoryReferences.value?.remote_branches || []
    for (const remoteBranch of remoteBranches) {
      items.push({
        id: `branch-remote-${remoteBranch.name}`,
        title: remoteBranch.name,
        category: 'branches',
        icon: 'i-lucide-git-branch',
        badge: 'Remote',
        badgeColor: 'neutral',
        keywords: ['branch', 'remote', remoteBranch.name],
        action: async () => {
          await checkoutBranch(remoteBranch.name)
        },
      })
    }
  }

  // 3. Global & Recent Repositories
  items.push({
    id: 'global-open-repository',
    title: t('commandPalette.actions.openRepository'),
    category: 'repositories',
    icon: 'i-lucide-folder-open',
    keywords: ['open', 'folder', 'directory', 'پوشه'],
    action: () => {
      emit('openFolderPicker')
    },
  })

  const recents = recentRepositories.value || []
  for (const repositoryRecord of recents) {
    if (currentRepository.value && repositoryRecord.path === currentRepository.value.path) {
      continue
    }
    items.push({
      id: `recent-${repositoryRecord.path}`,
      title: repositoryRecord.name,
      description: repositoryRecord.path,
      category: 'repositories',
      icon: 'i-lucide-folder-git-2',
      keywords: ['repository', repositoryRecord.name, repositoryRecord.path],
      action: async () => {
        await openRepository(repositoryRecord.path)
      },
    })
  }

  // 4. Preferences & Settings
  items.push(
    {
      id: 'pref-settings',
      title: t('commandPalette.actions.settings'),
      category: 'preferences',
      icon: 'i-lucide-settings',
      keywords: ['settings', 'preferences', 'تنظیمات'],
      action: () => {
        emit('openSettings')
      },
    },
    {
      id: 'pref-theme-dark',
      title: t('commandPalette.actions.switchThemeDark'),
      category: 'preferences',
      icon: 'i-lucide-moon',
      keywords: ['theme', 'dark', 'تاریک'],
      action: () => {
        colorMode.preference = 'dark'
      },
    },
    {
      id: 'pref-theme-light',
      title: t('commandPalette.actions.switchThemeLight'),
      category: 'preferences',
      icon: 'i-lucide-sun',
      keywords: ['theme', 'light', 'روشن'],
      action: () => {
        colorMode.preference = 'light'
      },
    },
    {
      id: 'pref-lang-en',
      title: t('commandPalette.actions.switchLanguageEn'),
      category: 'preferences',
      icon: 'i-lucide-languages',
      keywords: ['english', 'language', 'زبان'],
      action: async () => {
        await setLocale('en')
      },
    },
    {
      id: 'pref-lang-fa',
      title: t('commandPalette.actions.switchLanguageFa'),
      category: 'preferences',
      icon: 'i-lucide-languages',
      keywords: ['persian', 'farsi', 'فارسی', 'زبان'],
      action: async () => {
        await setLocale('fa')
      },
    },
  )

  return items
})

const filteredItems = computed<CommandItem[]>(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) {
    return allItems.value
  }
  return allItems.value.filter((item) => {
    if (item.title.toLowerCase().includes(query)) return true
    if (item.description?.toLowerCase().includes(query)) return true
    if (item.keywords?.some((keyword) => keyword.toLowerCase().includes(query))) return true
    return false
  })
})

const categoryLabels: Record<CommandItem['category'], string> = {
  actions: 'commandPalette.categories.actions',
  branches: 'commandPalette.categories.branches',
  repositories: 'commandPalette.categories.repositories',
  preferences: 'commandPalette.categories.preferences',
}

function handleNavigateDown(): void {
  if (filteredItems.value.length === 0) return
  selectedIndex.value = (selectedIndex.value + 1) % filteredItems.value.length
  scrollToActiveItem()
}

function handleNavigateUp(): void {
  if (filteredItems.value.length === 0) return
  selectedIndex.value = (selectedIndex.value - 1 + filteredItems.value.length) % filteredItems.value.length
  scrollToActiveItem()
}

function scrollToActiveItem(): void {
  nextTick(() => {
    const element = itemElements.value[selectedIndex.value]
    if (element) {
      element.scrollIntoView({ block: 'nearest' })
    }
  })
}

function setItemReference(index: number) {
  return (element: unknown): void => {
    itemElements.value[index] = (element as HTMLElement | null) ?? null
  }
}

async function handleExecuteActiveItem(): Promise<void> {
  const item = filteredItems.value[selectedIndex.value]
  if (!item) return
  isOpen.value = false
  await item.action()
}

async function handleExecuteItem(item: CommandItem): Promise<void> {
  isOpen.value = false
  await item.action()
}

watch(searchQuery, () => {
  selectedIndex.value = 0
})

watch(isOpen, (newIsOpen) => {
  if (newIsOpen) {
    searchQuery.value = ''
    selectedIndex.value = 0
    nextTick(() => {
      searchInputReference.value?.focus()
    })
  }
})
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :ui="{
      content: 'sm:max-w-2xl max-h-[75vh] p-0 border-0 bg-transparent shadow-none ring-0 focus:outline-none overflow-hidden rounded-2xl',
    }"
  >
    <template #content>
      <div class="flex flex-col w-full max-h-[75vh] rounded-2xl border border-[var(--gv-border)] bg-[var(--gv-surface)] shadow-2xl overflow-hidden text-xs">
        <!-- Input Search Bar -->
        <div class="relative flex items-center border-b border-[var(--gv-border)] px-4 py-3 bg-[var(--gv-sidebar)] select-none">
          <UIcon name="i-lucide-search" class="h-4 w-4 shrink-0 text-muted me-2" />
          <input
            ref="searchInputReference"
            v-model="searchQuery"
            type="text"
            class="flex-1 bg-transparent text-sm text-highlighted placeholder:text-muted focus:outline-none"
            :placeholder="t('commandPalette.placeholder')"
            @keydown.down.prevent="handleNavigateDown"
            @keydown.up.prevent="handleNavigateUp"
            @keydown.enter.prevent="handleExecuteActiveItem"
            @keydown.esc.prevent="isOpen = false"
          >
          <div class="flex items-center gap-1.5 ms-2">
            <UKbd size="sm" class="font-mono text-[10px]">Esc</UKbd>
          </div>
        </div>

        <!-- Scrollable Command List -->
        <div class="flex-1 overflow-y-auto p-2 space-y-1 min-h-[160px] max-h-[55vh]">
          <!-- Empty State -->
          <div
            v-if="filteredItems.length === 0"
            class="flex flex-col items-center justify-center p-8 text-center text-muted"
          >
            <UIcon name="i-lucide-search-x" class="h-8 w-8 text-muted/60 mb-2" />
            <p class="font-medium text-xs text-highlighted">{{ t('commandPalette.noResults') }}</p>
          </div>

          <!-- Items List -->
          <div
            v-for="(item, index) in filteredItems"
            :key="item.id"
            :ref="setItemReference(index)"
            class="flex items-center justify-between gap-3 px-3 py-2 rounded-lg cursor-pointer transition-colors"
            :class="selectedIndex === index
              ? 'bg-primary/15 text-primary border border-primary/30'
              : 'hover:bg-muted/15 text-highlighted border border-transparent'"
            @mouseenter="selectedIndex = index"
            @click="handleExecuteItem(item)"
          >
            <!-- Leading Icon & Title -->
            <div class="flex items-center gap-2.5 min-w-0">
              <UIcon
                :name="item.icon"
                class="h-4 w-4 shrink-0"
                :class="selectedIndex === index ? 'text-primary' : 'text-muted'"
              />
              <div class="flex flex-col min-w-0">
                <span class="font-medium text-[12px] truncate">{{ item.title }}</span>
                <span
                  v-if="item.description"
                  class="text-[10px] text-muted truncate font-mono"
                  dir="ltr"
                >
                  {{ item.description }}
                </span>
              </div>
            </div>

            <!-- Trailing Badge / Category -->
            <div class="flex items-center gap-2 shrink-0">
              <UBadge
                v-if="item.badge"
                size="xs"
                :color="item.badgeColor || 'neutral'"
                variant="subtle"
              >
                {{ item.badge }}
              </UBadge>
              <span class="text-[10px] text-muted font-normal">
                {{ t(categoryLabels[item.category]) }}
              </span>
            </div>
          </div>
        </div>

        <!-- Footer Shortcuts Help -->
        <div class="flex items-center justify-between border-t border-[var(--gv-border)] px-4 py-2 bg-[var(--gv-sidebar)] text-[11px] text-muted select-none">
          <div class="flex items-center gap-3">
            <span class="flex items-center gap-1">
              <UKbd size="sm">↑</UKbd>
              <UKbd size="sm">↓</UKbd>
              <span>{{ t('common.navigate') || 'Navigate' }}</span>
            </span>
            <span class="flex items-center gap-1">
              <UKbd size="sm">↵</UKbd>
              <span>{{ t('common.select') || 'Select' }}</span>
            </span>
          </div>
          <span class="font-mono text-[10px]">{{ filteredItems.length }} {{ t('common.items') || 'items' }}</span>
        </div>
      </div>
    </template>
  </UModal>
</template>
