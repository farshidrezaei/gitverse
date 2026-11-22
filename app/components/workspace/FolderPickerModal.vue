<script setup lang="ts">
import type { DirectoryListing, FileEntry } from '~/types/repository'
import type { PathPickerMode } from '~/composables/usePathPickerModal'

interface FolderPickerModalProperties {
  open: boolean
  mode?: PathPickerMode
  title?: string
  subtitle?: string
  confirmLabel?: string
  mustBeGitRepository?: boolean
  fileExtensions?: string[]
  initialPath?: string
  defaultFileName?: string
}

const properties = withDefaults(defineProps<FolderPickerModalProperties>(), {
  mode: 'folder',
  mustBeGitRepository: undefined,
  title: undefined,
  subtitle: undefined,
  confirmLabel: undefined,
  fileExtensions: undefined,
  initialPath: undefined,
  defaultFileName: undefined,
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  selectFolder: [selectedPath: string]
  cancel: []
}>()

const { t } = useI18n()
const { listDirectoryFolders } = useWorkspaceManager()

const currentListing = ref<DirectoryListing | null>(null)
const manualPathInput = ref<string>('')
const folderSearchQuery = ref<string>('')
const isLoadingFolders = ref<boolean>(false)
const selectedFilePath = ref<string | null>(null)
const saveFileNameInput = ref<string>('')

const isGitRepositoryRequired = computed<boolean>(() => {
  if (properties.mode !== 'folder') return false
  if (properties.mustBeGitRepository !== undefined) return properties.mustBeGitRepository
  return true
})

watch(
  () => properties.open,
  async (isOpen) => {
    if (isOpen) {
      folderSearchQuery.value = ''
      selectedFilePath.value = null
      saveFileNameInput.value = properties.defaultFileName || ''
      await navigateToDirectory(properties.initialPath)
    }
  },
)

async function navigateToDirectory(targetPath?: string): Promise<void> {
  isLoadingFolders.value = true
  selectedFilePath.value = null
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

function handleFolderClick(folderPath: string): void {
  if (properties.mode === 'folder') {
    manualPathInput.value = folderPath
  }
}

function handleFileClick(file: FileEntry): void {
  selectedFilePath.value = file.path
  if (properties.mode === 'save_file') {
    saveFileNameInput.value = file.name
  }
}

function handleFileDoubleClick(file: FileEntry): void {
  selectedFilePath.value = file.path
  handleConfirmSelection()
}

function handleConfirmSelection(): void {
  if (properties.mode === 'file') {
    if (selectedFilePath.value) {
      emit('selectFolder', selectedFilePath.value)
      emit('update:open', false)
    }
  } else if (properties.mode === 'save_file') {
    if (currentListing.value && saveFileNameInput.value.trim()) {
      const fullPath = `${currentListing.value.current_path}/${saveFileNameInput.value.trim()}`
      emit('selectFolder', fullPath)
      emit('update:open', false)
    }
  } else {
    if (currentListing.value) {
      const selectedPath = manualPathInput.value.trim() || currentListing.value.current_path
      emit('selectFolder', selectedPath)
      emit('update:open', false)
    }
  }
}

function handleClose(): void {
  emit('cancel')
  emit('update:open', false)
}

function handleQuickPath(path: string): void {
  void navigateToDirectory(path)
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function getFileIcon(extension?: string | null): string {
  if (!extension) return 'i-lucide-file'
  const lower = extension.toLowerCase()
  if (['patch', 'diff'].includes(lower)) return 'i-lucide-file-code'
  if (['bundle', 'zip', 'tar', 'gz'].includes(lower)) return 'i-lucide-archive'
  if (['json', 'yml', 'yaml', 'toml', 'xml'].includes(lower)) return 'i-lucide-file-cog'
  return 'i-lucide-file-text'
}

const filteredFolders = computed(() => {
  if (!currentListing.value) return []
  if (!folderSearchQuery.value.trim()) return currentListing.value.folders
  const query = folderSearchQuery.value.toLowerCase()
  return currentListing.value.folders.filter(folder => folder.name.toLowerCase().includes(query))
})

const filteredFiles = computed(() => {
  if (!currentListing.value || properties.mode === 'folder') return []
  let files = currentListing.value.files || []

  if (properties.fileExtensions && properties.fileExtensions.length > 0) {
    const allowedExtensions = properties.fileExtensions.map(e => e.toLowerCase().replace(/^\./, ''))
    files = files.filter(f => f.extension && allowedExtensions.includes(f.extension.toLowerCase()))
  }

  if (folderSearchQuery.value.trim()) {
    const query = folderSearchQuery.value.toLowerCase()
    files = files.filter(f => f.name.toLowerCase().includes(query))
  }

  return files
})

const canConfirm = computed<boolean>(() => {
  if (isLoadingFolders.value) return false
  if (properties.mode === 'file') {
    return Boolean(selectedFilePath.value)
  }
  if (properties.mode === 'save_file') {
    return Boolean(currentListing.value && saveFileNameInput.value.trim())
  }
  if (isGitRepositoryRequired.value) {
    return Boolean(currentListing.value?.is_current_git_repository)
  }
  return Boolean(currentListing.value)
})

const headerIcon = computed<string>(() => {
  if (properties.mode === 'file') return 'i-lucide-file-code'
  if (properties.mode === 'save_file') return 'i-lucide-file-down'
  return 'i-lucide-folder-git-2'
})

const headerTitle = computed<string>(() => {
  if (properties.title) return properties.title
  if (properties.mode === 'file') return t('repository.selectFileTitle')
  if (properties.mode === 'save_file') return t('repository.selectSavePathTitle')
  return t('repository.selectFolderTitle')
})

const headerSubtitle = computed<string>(() => {
  if (properties.subtitle) return properties.subtitle
  return t('repository.selectFolderSubtitle')
})

const confirmButtonLabel = computed<string>(() => {
  if (properties.confirmLabel) return properties.confirmLabel
  if (properties.mode === 'file') return t('repository.selectThisFile')
  if (properties.mode === 'save_file') return t('repository.saveToThisLocation')
  if (isGitRepositoryRequired.value) return t('repository.openThisFolder')
  return t('repository.selectThisFolder')
})
</script>

<template>
  <UModal
    :open="properties.open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-2xl p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="handleClose"
  >
    <template #content>
      <div class="flex flex-col select-none">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20">
          <div class="flex items-center gap-2">
            <UIcon :name="headerIcon" class="h-5 w-5 text-primary" />
            <div>
              <h3 class="text-sm font-semibold text-highlighted">{{ headerTitle }}</h3>
              <p class="text-[11px] text-muted">{{ headerSubtitle }}</p>
            </div>
          </div>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            :aria-label="t('common.close')"
            @click="handleClose"
          />
        </div>

        <!-- Body -->
        <div class="p-5 space-y-3 text-xs">
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
            :placeholder="properties.mode === 'folder' ? t('repository.filterFolders') : t('repository.filterFilesAndFolders')"
          />

          <!-- Files & Folders List -->
          <div class="h-64 rounded-lg border border-default bg-[var(--gv-surface)]/60 overflow-y-auto p-1 space-y-1">
            <!-- Zero-CLS Skeleton Loading for Folders List -->
            <div
              v-if="isLoadingFolders"
              class="space-y-1 p-1 animate-pulse"
            >
              <div
                v-for="index in 6"
                :key="`folder-skeleton-${index}`"
                class="flex items-center justify-between rounded-md px-2.5 py-1.5"
              >
                <div class="flex items-center gap-2">
                  <USkeleton class="h-4 w-4 rounded shrink-0" />
                  <USkeleton
                    class="h-3.5 rounded"
                    :class="[
                      index % 3 === 0 ? 'w-36' :
                      index % 2 === 0 ? 'w-24' : 'w-48'
                    ]"
                  />
                </div>
                <USkeleton v-if="index % 2 === 0" class="h-4 w-12 rounded-full shrink-0" />
              </div>
            </div>

            <template v-else-if="filteredFolders.length > 0 || filteredFiles.length > 0">
              <!-- Folders -->
              <div
                v-for="folder in filteredFolders"
                :key="folder.path"
                class="group flex items-center justify-between rounded-md px-2.5 py-1.5 transition-colors cursor-pointer hover:bg-primary/10 hover:text-primary"
                :class="properties.mode === 'folder' && manualPathInput === folder.path ? 'bg-primary/15 text-primary font-medium' : ''"
                @dblclick="handleFolderDoubleClick(folder.path)"
                @click="handleFolderClick(folder.path)"
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

              <!-- Files (in file/save_file mode) -->
              <div
                v-for="file in filteredFiles"
                :key="file.path"
                class="group flex items-center justify-between rounded-md px-2.5 py-1.5 transition-colors cursor-pointer hover:bg-primary/10 hover:text-primary"
                :class="selectedFilePath === file.path ? 'bg-primary/20 text-primary font-medium' : ''"
                @dblclick="handleFileDoubleClick(file)"
                @click="handleFileClick(file)"
              >
                <div class="flex items-center gap-2 min-w-0">
                  <UIcon
                    :name="getFileIcon(file.extension)"
                    class="h-4 w-4 shrink-0 text-muted group-hover:text-primary"
                  />
                  <span class="truncate">{{ file.name }}</span>
                </div>

                <span class="text-[10px] text-muted font-mono shrink-0">
                  {{ formatFileSize(file.size_bytes) }}
                </span>
              </div>
            </template>

            <div
              v-else
              class="flex h-full flex-col items-center justify-center text-muted gap-1 text-xs"
            >
              <UIcon name="i-lucide-folder-x" class="h-6 w-6 text-muted/60" />
              <span>{{ properties.mode === 'folder' ? t('repository.noSubfolders') : t('repository.noMatchingEntries') }}</span>
            </div>
          </div>

          <!-- Save File Name Input (in save_file mode) -->
          <div v-if="properties.mode === 'save_file'" class="space-y-1 pt-1">
            <label class="text-[11px] font-medium text-highlighted">
              {{ t('repository.fileNameLabel') }}
            </label>
            <UInput
              v-model="saveFileNameInput"
              class="w-full font-mono text-xs"
              size="xs"
              icon="i-lucide-file-plus"
              :placeholder="t('repository.fileNamePlaceholder')"
              @keydown.enter="handleConfirmSelection"
            />
          </div>

          <!-- Current folder Git status indicator (in folder mode) -->
          <div
            v-if="currentListing && properties.mode === 'folder'"
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

        <!-- Footer -->
        <div class="flex w-full items-center justify-end gap-2 border-t border-default/70 px-5 py-3 bg-muted/20">
          <UButton
            size="sm"
            color="neutral"
            variant="outline"
            @click="handleClose"
          >
            {{ t('common.cancel') }}
          </UButton>
          <UButton
            size="sm"
            color="primary"
            icon="i-lucide-check"
            :disabled="!canConfirm"
            @click="handleConfirmSelection"
          >
            {{ confirmButtonLabel }}
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>
