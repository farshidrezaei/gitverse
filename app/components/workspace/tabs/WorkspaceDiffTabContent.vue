<script setup lang="ts">
import type { FileDiff } from '~/types/repository'
import type { WorkspaceDiffTab } from '~/types/workspaceTab'
import type { SelectedDiffTarget } from '~/composables/useStagingController'
import DiffHeaderToolbar from '~/components/workspace/diff/DiffHeaderToolbar.vue'
import DiffUnifiedView from '~/components/workspace/diff/DiffUnifiedView.vue'
import DiffSplitView from '~/components/workspace/diff/DiffSplitView.vue'
import DiffViewerSkeleton from '~/components/workspace/diff/DiffViewerSkeleton.vue'
import { buildHunkPatch, buildSelectedLinesPatch } from '~/utils/patchGenerator'

interface WorkspaceDiffTabContentProperties {
  tab: WorkspaceDiffTab
}

const properties = defineProps<WorkspaceDiffTabContentProperties>()

const emit = defineEmits<{
  stage: [filePath: string]
  unstage: [filePath: string]
  discard: [filePath: string, isUntracked: boolean]
  close: [tabIdentifier: string]
}>()

const { t } = useI18n()
const { currentRepository } = useRepositoryWorkspace()
const { stagePatch, unstagePatch, discardPatch } = useStagingController()

const diffData = ref<FileDiff | null>(null)
const isLoading = ref<boolean>(false)

const isTargetStaged = computed<boolean | null>(() => {
  if (properties.tab.commitHash || properties.tab.comparisonBaseHash) return null
  return properties.tab.isStaged ?? null
})

async function handleStageHunk(hunkIndex: number): Promise<void> {
  if (!diffData.value) return
  const patch = buildHunkPatch(diffData.value, hunkIndex)
  if (!patch) return
  const success = await stagePatch(patch)
  if (success) {
    await loadDiff()
  }
}

async function handleUnstageHunk(hunkIndex: number): Promise<void> {
  if (!diffData.value) return
  const patch = buildHunkPatch(diffData.value, hunkIndex)
  if (!patch) return
  const success = await unstagePatch(patch)
  if (success) {
    await loadDiff()
  }
}

async function handleDiscardHunk(hunkIndex: number): Promise<void> {
  if (!diffData.value) return
  const patch = buildHunkPatch(diffData.value, hunkIndex)
  if (!patch) return
  const success = await discardPatch(patch)
  if (success) {
    await loadDiff()
  }
}

async function handleStageLine(hunkIndex: number, lineIndex: number): Promise<void> {
  if (!diffData.value) return
  const patch = buildSelectedLinesPatch(diffData.value, hunkIndex, [lineIndex], 'stage')
  if (!patch) return
  const success = await stagePatch(patch)
  if (success) {
    await loadDiff()
  }
}

async function handleUnstageLine(hunkIndex: number, lineIndex: number): Promise<void> {
  if (!diffData.value) return
  const patch = buildSelectedLinesPatch(diffData.value, hunkIndex, [lineIndex], 'unstage')
  if (!patch) return
  const success = await unstagePatch(patch)
  if (success) {
    await loadDiff()
  }
}

async function handleDiscardLine(hunkIndex: number, lineIndex: number): Promise<void> {
  if (!diffData.value) return
  const patch = buildSelectedLinesPatch(diffData.value, hunkIndex, [lineIndex], 'discard')
  if (!patch) return
  const success = await discardPatch(patch)
  if (success) {
    await loadDiff()
  }
}

const DIFF_VIEW_MODE_STORAGE_KEY = 'gitverse_diff_view_mode'
const DIFF_WORD_WRAP_STORAGE_KEY = 'gitverse_diff_word_wrap'
type DiffViewMode = 'inline' | 'side-by-side'
const viewMode = ref<DiffViewMode>('side-by-side')
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
  void loadDiff()
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

watch(
  () => [properties.tab.id, properties.tab.isStaged],
  () => {
    void loadDiff()
  },
)

async function isTauriEnvironment(): Promise<boolean> {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

async function loadDiff(): Promise<void> {
  if (!currentRepository.value) {
    diffData.value = null
    return
  }

  isLoading.value = true
  try {
    if (await isTauriEnvironment()) {
      const { invoke } = await import('@tauri-apps/api/core')

      if (properties.tab.comparisonBaseHash && properties.tab.comparisonTargetHash) {
        const diff = await invoke<FileDiff>('get_comparison_file_diff', {
          baseHash: properties.tab.comparisonBaseHash,
          targetHash: properties.tab.comparisonTargetHash,
          filePath: properties.tab.filePath,
          directoryPath: currentRepository.value.path,
        })
        diffData.value = diff
      } else if (properties.tab.commitHash) {
        const diff = await invoke<FileDiff>('get_commit_file_diff', {
          commitHash: properties.tab.commitHash,
          filePath: properties.tab.filePath,
          directoryPath: currentRepository.value.path,
        })
        diffData.value = diff
      } else {
        const diff = await invoke<FileDiff>('get_file_diff', {
          filePath: properties.tab.filePath,
          isStaged: properties.tab.isStaged ?? false,
          directoryPath: currentRepository.value.path,
        })
        diffData.value = diff
      }
    } else {
      // Browser preview mode mock diff
      diffData.value = {
        old_path: properties.tab.isStaged ? null : properties.tab.filePath,
        new_path: properties.tab.filePath,
        is_binary: false,
        is_new_file: false,
        is_deleted_file: false,
        total_additions: 4,
        total_deletions: 2,
        hunks: [
          {
            header: '@@ -1,8 +1,10 @@',
            old_start: 1,
            old_lines: 8,
            new_start: 1,
            new_lines: 10,
            lines: [
              {
                line_type: 'context',
                old_line_number: 1,
                new_line_number: 1,
                content: `// Workspace Tab Diff for: ${properties.tab.filePath}`,
              },
              {
                line_type: 'deletion',
                old_line_number: 2,
                new_line_number: null,
                content: '-const previousConfiguration = false;',
              },
              {
                line_type: 'addition',
                old_line_number: null,
                new_line_number: 2,
                content: '+const updatedConfiguration = true;',
              },
              {
                line_type: 'addition',
                old_line_number: null,
                new_line_number: 3,
                content: '+const fullScreenDiffTabEnabled = true;',
              },
              {
                line_type: 'context',
                old_line_number: 3,
                new_line_number: 4,
                content: '',
              },
            ],
          },
        ],
      }
    }
  } catch {
    diffData.value = null
  } finally {
    isLoading.value = false
  }
}

const computedSelectedTarget = computed<SelectedDiffTarget | null>(() => {
  if (properties.tab.commitHash) {
    return null
  }
  return {
    filePath: properties.tab.filePath,
    isStaged: properties.tab.isStaged ?? false,
  }
})

function handleStage(filePath: string): void {
  emit('stage', filePath)
  setTimeout(() => {
    void loadDiff()
  }, 200)
}

function handleUnstage(filePath: string): void {
  emit('unstage', filePath)
  setTimeout(() => {
    void loadDiff()
  }, 200)
}
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden bg-[var(--gv-surface)] text-xs select-none" dir="ltr">
    <!-- Diff Header Toolbar with Close Tab action -->
    <div class="flex items-center justify-between border-b border-default/70 bg-muted/15 pe-2">
      <div class="flex-1 min-w-0">
        <DiffHeaderToolbar
          :diff="diffData"
          :selected-target="computedSelectedTarget"
          :view-mode="viewMode"
          :is-word-wrap="isWordWrap"
          @update:view-mode="viewMode = $event"
          @update:is-word-wrap="isWordWrap = $event"
          @stage="handleStage"
          @unstage="handleUnstage"
        />
      </div>

      <!-- Action buttons: Refresh & Close Tab -->
      <div class="flex items-center gap-1 shrink-0 ps-2">
        <UTooltip :text="t('common.refresh')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-refresh-cw"
            :loading="isLoading"
            :aria-label="t('common.refresh')"
            @click="loadDiff"
          />
        </UTooltip>

        <UTooltip :text="t('tabs.close')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            class="hover:bg-error/15 hover:text-error"
            :aria-label="t('tabs.close')"
            @click="emit('close', properties.tab.id)"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Main Diff Viewport -->
    <div
      class="flex-1 min-h-0 bg-[var(--gv-surface)] font-mono text-[11px]"
      :class="viewMode === 'side-by-side' ? 'overflow-hidden flex flex-col' : 'overflow-auto leading-5'"
    >
      <DiffViewerSkeleton v-if="isLoading" />

      <!-- Binary File Notice -->
      <div
        v-else-if="diffData?.is_binary"
        class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted"
      >
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-binary" class="h-8 w-8 text-warning" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.binaryFile') }}</p>
          <p class="text-[11px] text-muted mt-0.5 font-mono">
            {{ properties.tab.filePath }}
          </p>
        </div>
      </div>

      <!-- Empty / No Changes Notice -->
      <div
        v-else-if="!diffData || diffData.hunks.length === 0"
        class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted"
      >
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-file-check" class="h-8 w-8 text-success" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.noChanges') }}</p>
          <p class="text-[11px] text-muted mt-0.5 font-mono">
            {{ properties.tab.filePath }}
          </p>
        </div>
      </div>

      <!-- Unified (Inline) Diff View -->
      <DiffUnifiedView
        v-else-if="viewMode === 'inline'"
        :diff="diffData"
        :is-word-wrap="isWordWrap"
        :is-staged="isTargetStaged"
        @stage-hunk="handleStageHunk"
        @unstage-hunk="handleUnstageHunk"
        @discard-hunk="handleDiscardHunk"
        @stage-line="handleStageLine"
        @unstage-line="handleUnstageLine"
        @discard-line="handleDiscardLine"
      />

      <!-- Split (Side-by-Side) Diff View -->
      <DiffSplitView
        v-else
        :diff="diffData"
        :is-word-wrap="isWordWrap"
        :is-staged="isTargetStaged"
        @stage-hunk="handleStageHunk"
        @unstage-hunk="handleUnstageHunk"
        @discard-hunk="handleDiscardHunk"
        @stage-line="handleStageLine"
        @unstage-line="handleUnstageLine"
        @discard-line="handleDiscardLine"
      />
    </div>
  </div>
</template>
