<script setup lang="ts">
import type { DirectoryListing } from '~/types/repository'

interface FolderPickerModalProperties {
  open: boolean
}

const properties = defineProps<FolderPickerModalProperties>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  selectFolder: [folderPath: string]
}>()

const { t } = useI18n()
const { listDirectoryFolders } = useWorkspaceManager()

const currentListing = ref<DirectoryListing | null>(null)
const manualPathInput = ref<string>('')
const folderSearchQuery = ref<string>('')
const isLoadingFolders = ref<boolean>(false)

watch(
  () => properties.open,
  async (isOpen) => {
    if (isOpen) {
      folderSearchQuery.value = ''
      await navigateToDirectory()
    }
  },
)

async function navigateToDirectory(targetPath?: string): Promise<void> {
  isLoadingFolders.value = true
  try {
    const listing = await listDirectoryFolders(targetPath)
    if (listing) {
      currentListing.value = listing
      manualPathInput.value = listing.current_path
    }
  } finally {
    isLoadingFolders.value = false
  }
}

async function handleNavigateUp(): Promise<void> {
  if (currentListing.value?.parent_path) {
    await navigateToDirectory(currentListing.value.parent_path)
  }
}

async function handleFolderDoubleClick(folderPath: string): Promise<void> {
  await navigateToDirectory(folderPath)
}

function handleConfirmSelection(): void {
  if (currentListing.value) {
    emit('selectFolder', currentListing.value.current_path)
    emit('update:open', false)
  }
}

function handleQuickPath(path: string): void {
  void navigateToDirectory(path)
}

const filteredFolders = computed(() => {
  if (!currentListing.value) return []
  if (!folderSearchQuery.value.trim()) return currentListing.value.folders
  const query = folderSearchQuery.value.toLowerCase()
  return currentListing.value.folders.filter(folder => folder.name.toLowerCase().includes(query))
})
</script>

<template>
  <UModal
    :open="properties.open"
    :title="t('repository.selectFolderTitle')"
    :description="t('repository.selectFolderSubtitle')"
    :ui="{ content: 'sm:max-w-2xl' }"
    @update:open="emit('update:open', $event)"
  >
    <template #body>
      <div class="space-y-3 select-none text-xs">
        <!-- Quick Nav Shortcuts -->
        <div class="flex items-center gap-1.5 overflow-x-auto pb-1 text-xs">
          <UButton
            size="xs"
            color="neutral"
            variant="soft"
            icon="i-lucide-home"
            @click="handleQuickPath('')"
          >
            Home
          </UButton>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-monitor"
            @click="handleQuickPath('~/Desktop')"
          >
            Desktop
          </UButton>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-folder"
            @click="handleQuickPath('~/Documents')"
          >
            Documents
          </UButton>
        </div>

        <!-- Path Bar with Up button -->
        <div class="flex items-center gap-1.5">
          <UButton
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-arrow-up"
            :disabled="!currentListing?.parent_path"
            :title="t('repository.parentFolder')"
            @click="handleNavigateUp"
          />

          <UInput
            v-model="manualPathInput"
            class="flex-1 font-mono text-xs"
            size="xs"
            icon="i-lucide-folder"
            @keydown.enter="navigateToDirectory(manualPathInput)"
          />

          <UButton
            size="xs"
            color="primary"
            variant="soft"
            icon="i-lucide-arrow-right"
            @click="navigateToDirectory(manualPathInput)"
          />
        </div>

        <!-- Filter Subfolders Input -->
        <UInput
          v-model="folderSearchQuery"
          class="w-full text-xs"
          size="xs"
          icon="i-lucide-search"
          :placeholder="t('repository.filterFolders')"
        />

        <!-- Folders Only List -->
        <div class="h-64 rounded-lg border border-default bg-[var(--gv-surface)]/40 overflow-y-auto p-1 space-y-1">
          <div
            v-if="isLoadingFolders"
            class="flex h-full items-center justify-center text-muted gap-2"
          >
            <UIcon name="i-lucide-loader-2" class="h-4 w-4 animate-spin" />
            <span>{{ t('common.loading') }}</span>
          </div>

          <template v-else-if="filteredFolders.length > 0">
            <div
              v-for="folder in filteredFolders"
              :key="folder.path"
              class="group flex items-center justify-between rounded-md px-2.5 py-1.5 transition-colors cursor-pointer hover:bg-primary/10 hover:text-primary"
              @dblclick="handleFolderDoubleClick(folder.path)"
              @click="manualPathInput = folder.path"
            >
              <div class="flex items-center gap-2 min-w-0">
                <UIcon
                  :name="folder.is_git_repository ? 'i-lucide-folder-git-2' : 'i-lucide-folder'"
                  class="h-4 w-4 shrink-0"
                  :class="folder.is_git_repository ? 'text-primary' : 'text-muted group-hover:text-primary'"
                />
                <span class="font-medium truncate">{{ folder.name }}</span>
              </div>

              <!-- Git Badge if it's a repository -->
              <div class="flex items-center gap-1.5 shrink-0">
                <UBadge
                  v-if="folder.is_git_repository"
                  size="xs"
                  color="primary"
                  variant="subtle"
                >
                  Git
                </UBadge>
                <UIcon
                  name="i-lucide-chevron-right"
                  class="h-3 w-3 text-muted/60 opacity-0 group-hover:opacity-100 rtl:rotate-180 transition-opacity"
                />
              </div>
            </div>
          </template>

          <div
            v-else
            class="flex h-full flex-col items-center justify-center text-muted gap-1 text-xs"
          >
            <UIcon name="i-lucide-folder-x" class="h-6 w-6 text-muted/60" />
            <span>{{ t('repository.noSubfolders') }}</span>
          </div>
        </div>

        <!-- Current folder Git status indicator -->
        <div
          v-if="currentListing"
          class="flex items-center justify-between rounded-md px-2.5 py-1.5 text-xs border"
          :class="[
            currentListing.is_current_git_repository
              ? 'bg-success/10 border-success/30 text-success'
              : 'bg-muted/15 border-default text-muted',
          ]"
        >
          <div class="flex items-center gap-1.5">
            <UIcon
              :name="currentListing.is_current_git_repository ? 'i-lucide-check-circle-2' : 'i-lucide-info'"
              class="h-3.5 w-3.5"
            />
            <span class="font-medium">
              {{
                currentListing.is_current_git_repository
                  ? t('repository.folderIsGitRepository')
                  : t('repository.folderNotGitRepository')
              }}
            </span>
          </div>
          <span class="font-mono text-[10px] opacity-75 truncate max-w-[200px]" dir="ltr">
            {{ currentListing.current_path }}
          </span>
        </div>
      </div>
    </template>

    <template #footer>
      <div class="flex w-full items-center justify-end gap-2">
        <UButton
          size="sm"
          color="neutral"
          variant="outline"
          @click="emit('update:open', false)"
        >
          {{ t('common.cancel') }}
        </UButton>
        <UButton
          size="sm"
          color="primary"
          icon="i-lucide-check"
          :disabled="!currentListing?.is_current_git_repository"
          @click="handleConfirmSelection"
        >
          {{ t('repository.openThisFolder') }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
