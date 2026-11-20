<script setup lang="ts">
import type { FileDiff, StashDetails } from '~/types/repository'
import FileStatusBadge from '~/components/shared/FileStatusBadge.vue'
import DiffUnifiedView from '~/components/workspace/diff/DiffUnifiedView.vue'
import DiffViewerSkeleton from '~/components/workspace/diff/DiffViewerSkeleton.vue'

interface StashPreviewModalProps {
  open: boolean
  stashIndex: number | null
  stashIdentifier: string | null
}

const props = defineProps<StashPreviewModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  apply: [stashIndex: number]
  pop: [stashIndex: number]
  drop: [stashIndex: number]
}>()

const { t } = useI18n()
const { fetchStashDetails, fetchStashFileDiff } = useStashController()

const isLoadingDetails = ref<boolean>(false)
const isLoadingFileDiff = ref<boolean>(false)
const stashDetails = ref<StashDetails | null>(null)
const selectedFilePath = ref<string | null>(null)
const activeFileDiff = ref<FileDiff | null>(null)
const fileSearchQuery = ref<string>('')

const filteredFiles = computed(() => {
  if (!stashDetails.value) {
    return []
  }
  const query = fileSearchQuery.value.trim().toLowerCase()
  if (!query) {
    return stashDetails.value.changed_files
  }
  return stashDetails.value.changed_files.filter((file) =>
    file.path.toLowerCase().includes(query)
  )
})

watch(
  () => [props.open, props.stashIndex],
  async ([isOpen, newIndex]) => {
    if (isOpen && newIndex !== null && typeof newIndex === 'number') {
      await loadStashDetails(newIndex)
    } else if (!isOpen) {
      stashDetails.value = null
      selectedFilePath.value = null
      activeFileDiff.value = null
      fileSearchQuery.value = ''
    }
  },
  { immediate: true }
)

async function loadStashDetails(targetIndex: number): Promise<void> {
  isLoadingDetails.value = true
  stashDetails.value = null
  selectedFilePath.value = null
  activeFileDiff.value = null

  try {
    const details = await fetchStashDetails(targetIndex)
    stashDetails.value = details

    if (details && details.changed_files.length > 0) {
      const initialFile = details.changed_files[0]
      if (initialFile) {
        await selectFile(initialFile.path)
      }
    }
  } finally {
    isLoadingDetails.value = false
  }
}

async function selectFile(filePath: string): Promise<void> {
  if (props.stashIndex === null) {
    return
  }
  selectedFilePath.value = filePath
  isLoadingFileDiff.value = true

  try {
    const diff = await fetchStashFileDiff(props.stashIndex, filePath)
    activeFileDiff.value = diff
  } finally {
    isLoadingFileDiff.value = false
  }
}

function handleClose(): void {
  emit('update:open', false)
}

function handleApply(): void {
  if (props.stashIndex !== null) {
    emit('apply', props.stashIndex)
    emit('update:open', false)
  }
}

function handlePop(): void {
  if (props.stashIndex !== null) {
    emit('pop', props.stashIndex)
    emit('update:open', false)
  }
}

function handleDrop(): void {
  if (props.stashIndex !== null) {
    emit('drop', props.stashIndex)
    emit('update:open', false)
  }
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-5xl w-full h-[85vh] p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden flex flex-col',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col h-full overflow-hidden select-none">
        <!-- Modal Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20 shrink-0">
          <div class="flex items-center gap-2.5 min-w-0">
            <UIcon name="i-lucide-archive" class="h-5 w-5 text-primary shrink-0" />
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-sm font-semibold text-highlighted truncate">
                {{ t('stash.previewTitle') }}
              </span>
              <UBadge
                v-if="stashIdentifier"
                size="xs"
                color="neutral"
                variant="subtle"
                class="font-mono text-[11px] shrink-0"
              >
                {{ stashIdentifier }}
              </UBadge>
            </div>
          </div>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            @click="handleClose"
          />
        </div>

        <!-- Stash Metadata Banner -->
        <div class="px-5 py-3 border-b border-default/50 bg-surface/50 shrink-0">
          <div v-if="isLoadingDetails" class="space-y-2">
            <USkeleton class="h-4 w-3/4 rounded" />
            <div class="flex items-center gap-4">
              <USkeleton class="h-3 w-32 rounded" />
              <USkeleton class="h-3 w-24 rounded" />
            </div>
          </div>
          <div v-else-if="stashDetails" class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between gap-4">
              <span class="text-xs font-semibold text-highlighted truncate font-mono">
                {{ stashDetails.message || t('stash.title') }}
              </span>
              <div class="flex items-center gap-2 shrink-0 text-xs">
                <span class="text-success font-medium">+{{ stashDetails.total_additions }}</span>
                <span class="text-error font-medium">-{{ stashDetails.total_deletions }}</span>
              </div>
            </div>
            <div class="flex items-center gap-3 text-[11px] text-muted">
              <span class="font-mono text-muted/80">commit {{ stashDetails.commit_hash.slice(0, 7) }}</span>
              <span>&bull;</span>
              <span>{{ t('stash.filesChanged', { count: stashDetails.changed_files.length }) }}</span>
            </div>
          </div>
        </div>

        <!-- Main Workspace: Left file list + Right diff viewer -->
        <div class="flex-1 flex overflow-hidden min-h-0">
          <!-- Files list sidebar -->
          <div class="w-72 border-e border-default/70 flex flex-col bg-surface/30 shrink-0">
            <div class="p-2 border-b border-default/50">
              <UInput
                v-model="fileSearchQuery"
                size="xs"
                icon="i-lucide-search"
                :placeholder="t('stash.searchFiles')"
                class="w-full"
              />
            </div>

            <!-- Loading file list skeleton -->
            <div v-if="isLoadingDetails" class="p-2 space-y-1.5">
              <div
                v-for="skeletonIndex in 6"
                :key="`file-skeleton-${skeletonIndex}`"
                class="flex items-center gap-2 p-1.5"
              >
                <USkeleton class="h-4 w-4 rounded" />
                <USkeleton class="h-3.5 flex-1 rounded" />
              </div>
            </div>

            <!-- Files list -->
            <div
              v-else-if="filteredFiles.length > 0"
              class="flex-1 overflow-y-auto p-1.5 space-y-0.5"
            >
              <button
                v-for="fileItem in filteredFiles"
                :key="fileItem.path"
                type="button"
                class="w-full flex items-center justify-between gap-2 px-2 py-1.5 rounded-lg text-start transition-colors"
                :class="selectedFilePath === fileItem.path
                  ? 'bg-primary/10 text-primary font-medium'
                  : 'hover:bg-muted/10 text-default'"
                @click="selectFile(fileItem.path)"
              >
                <div class="flex items-center gap-1.5 min-w-0">
                  <FileStatusBadge :status="fileItem.status" size="xs" />
                  <span class="text-xs truncate font-mono" :title="fileItem.path">
                    {{ fileItem.path }}
                  </span>
                </div>
                <div class="flex items-center gap-1 shrink-0 text-[10px] font-mono">
                  <span v-if="fileItem.additions > 0" class="text-success">+{{ fileItem.additions }}</span>
                  <span v-if="fileItem.deletions > 0" class="text-error">-{{ fileItem.deletions }}</span>
                </div>
              </button>
            </div>

            <!-- Empty files state -->
            <div
              v-else
              class="flex-1 flex flex-col items-center justify-center p-4 text-center text-muted"
            >
              <UIcon name="i-lucide-folder-open" class="h-8 w-8 mb-1.5 opacity-40" />
              <span class="text-xs">{{ t('stash.previewNoFiles') }}</span>
            </div>
          </div>

          <!-- Diff viewer area -->
          <div class="flex-1 overflow-y-auto bg-surface/10 p-3 min-w-0">
            <!-- Loading diff skeleton -->
            <div v-if="isLoadingFileDiff">
              <DiffViewerSkeleton />
            </div>

            <!-- Unified diff view -->
            <div v-else-if="activeFileDiff && activeFileDiff.hunks.length > 0">
              <DiffUnifiedView :diff="activeFileDiff" />
            </div>

            <!-- Empty diff state -->
            <div
              v-else
              class="h-full flex flex-col items-center justify-center text-muted p-8 text-center"
            >
              <UIcon name="i-lucide-file-text" class="h-10 w-10 mb-2 opacity-40" />
              <p class="text-xs font-medium">{{ t('stash.previewSelectFile') }}</p>
            </div>
          </div>
        </div>

        <!-- Modal Footer Actions -->
        <div class="flex items-center justify-between border-t border-default/70 px-5 py-3 bg-muted/20 shrink-0">
          <div class="flex items-center gap-2">
            <UButton
              size="xs"
              color="error"
              variant="ghost"
              icon="i-lucide-trash"
              :label="t('stash.drop')"
              @click="handleDrop"
            />
          </div>

          <div class="flex items-center gap-2">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              :label="t('common.cancel')"
              @click="handleClose"
            />
            <UButton
              size="xs"
              color="neutral"
              variant="outline"
              icon="i-lucide-play"
              :label="t('stash.apply')"
              @click="handleApply"
            />
            <UButton
              size="xs"
              color="primary"
              variant="solid"
              icon="i-lucide-arrow-up-right"
              :label="t('stash.pop')"
              @click="handlePop"
            />
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>
