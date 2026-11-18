<script setup lang="ts">
import type { FileDiff } from '~/types/repository'
import type { SelectedDiffTarget } from '~/composables/useStagingController'
import DiffHeaderToolbar from './diff/DiffHeaderToolbar.vue'
import DiffUnifiedView from './diff/DiffUnifiedView.vue'
import DiffSplitView from './diff/DiffSplitView.vue'
import DiffViewerSkeleton from './diff/DiffViewerSkeleton.vue'

const props = defineProps<{
  diff: FileDiff | null
  selectedTarget: SelectedDiffTarget | null
  isLoading: boolean
}>()

defineEmits<{
  stage: [filePath: string]
  unstage: [filePath: string]
  discard: [filePath: string, isUntracked: boolean]
}>()

const { t } = useI18n()

const DIFF_VIEW_MODE_STORAGE_KEY = 'gitverse_diff_view_mode'
const DIFF_WORD_WRAP_STORAGE_KEY = 'gitverse_diff_word_wrap'
type DiffViewMode = 'inline' | 'side-by-side'
const viewMode = ref<DiffViewMode>('inline')
const isWordWrap = ref<boolean>(false)

onMounted(() => {
  if (import.meta.client) {
    try {
      const savedMode = localStorage.getItem(DIFF_VIEW_MODE_STORAGE_KEY)
      if (savedMode === 'inline' || savedMode === 'side-by-side') {
        viewMode.value = savedMode
      }
      const savedWrap = localStorage.getItem(DIFF_WORD_WRAP_STORAGE_KEY)
      if (savedWrap !== null) {
        isWordWrap.value = savedWrap === 'true'
      }
    } catch {
      // ignore
    }
  }
})

watch(viewMode, (newMode) => {
  if (import.meta.client) {
    try {
      localStorage.setItem(DIFF_VIEW_MODE_STORAGE_KEY, newMode)
    } catch {
      // ignore
    }
  }
})

watch(isWordWrap, (newWrap) => {
  if (import.meta.client) {
    try {
      localStorage.setItem(DIFF_WORD_WRAP_STORAGE_KEY, String(newWrap))
    } catch {
      // ignore
    }
  }
})
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden text-xs select-none" dir="ltr">
    <!-- Top Header Bar -->
    <DiffHeaderToolbar
      :diff="props.diff"
      :selected-target="props.selectedTarget"
      :view-mode="viewMode"
      :is-word-wrap="isWordWrap"
      @update:view-mode="viewMode = $event"
      @update:is-word-wrap="isWordWrap = $event"
      @stage="$emit('stage', $event)"
      @unstage="$emit('unstage', $event)"
    />

    <!-- Main Diff Body Container -->
    <div
      class="flex-1 min-h-0 bg-[var(--gv-surface)] font-mono text-[11px]"
      :class="viewMode === 'side-by-side' ? 'overflow-hidden flex flex-col' : 'overflow-auto leading-5'"
    >
      <!-- Loading Skeleton -->
      <DiffViewerSkeleton v-if="props.isLoading" />

      <!-- No Target Selected State -->
      <div
        v-else-if="!props.selectedTarget && !props.diff"
        class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted"
      >
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-file-diff" class="h-8 w-8 text-muted/70" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.noFileSelected') }}</p>
          <p class="text-[11px] text-muted mt-0.5">{{ t('diff.selectFilePrompt') }}</p>
        </div>
      </div>

      <!-- Binary File State -->
      <div
        v-else-if="props.diff?.is_binary"
        class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted"
      >
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-binary" class="h-8 w-8 text-warning" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.binaryFile') }}</p>
          <p class="text-[11px] text-muted mt-0.5 font-mono">
            {{ props.selectedTarget ? props.selectedTarget.filePath : props.diff.new_path }}
          </p>
        </div>
      </div>

      <!-- Empty / No Changes State -->
      <div
        v-else-if="!props.diff || props.diff.hunks.length === 0"
        class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted"
      >
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-file-check" class="h-8 w-8 text-success" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.noChanges') }}</p>
          <p class="text-[11px] text-muted mt-0.5 font-mono">
            {{ props.selectedTarget ? props.selectedTarget.filePath : props.diff?.new_path }}
          </p>
        </div>
      </div>

      <!-- Inline Unified View -->
      <DiffUnifiedView
        v-else-if="viewMode === 'inline'"
        :diff="props.diff"
        :is-word-wrap="isWordWrap"
      />

      <!-- Side-by-Side Split View -->
      <DiffSplitView
        v-else-if="viewMode === 'side-by-side'"
        :diff="props.diff"
        :is-word-wrap="isWordWrap"
      />
    </div>
  </div>
</template>
