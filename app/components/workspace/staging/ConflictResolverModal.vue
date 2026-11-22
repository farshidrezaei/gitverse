<script setup lang="ts">
import type { ConflictFileDetails } from '~/types/repository'
import {
  combineOursAndTheirs,
  computeConflictLineComparison,
  countConflictMarkers,
  hasConflictMarkers,
  parseConflictChunks,
  resolveConflictChunk,
  type ParsedConflictChunk,
} from '~/utils/conflict'

interface ConflictResolverModalProps {
  open: boolean
  conflictDetails: ConflictFileDetails | null
}

const props = defineProps<ConflictResolverModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  resolve: [strategy: 'ours' | 'theirs' | 'mark_resolved']
  saveContent: [content: string]
}>()

const { t } = useI18n()

const activeViewMode = ref<'workbench' | 'sideBySide' | 'editor'>('workbench')
const editableContent = ref<string>('')
const isBaseVisible = ref<boolean>(true)
const activeChunkIndex = ref<number>(0)

watch(
  () => props.conflictDetails,
  (newDetails) => {
    if (newDetails) {
      editableContent.value = newDetails.working_file_content || ''
      activeViewMode.value = 'workbench'
      activeChunkIndex.value = 0
    }
  },
  { immediate: true }
)

const parsedChunks = computed<ParsedConflictChunk[]>(() => {
  return parseConflictChunks(editableContent.value)
})

const remainingConflictCount = computed(() => {
  return parsedChunks.value.length
})

const stillHasConflictMarkers = computed(() => {
  return hasConflictMarkers(editableContent.value)
})

const currentChunk = computed<ParsedConflictChunk | null>(() => {
  if (parsedChunks.value.length === 0) return null
  const safeIndex = Math.min(Math.max(activeChunkIndex.value, 0), parsedChunks.value.length - 1)
  return parsedChunks.value[safeIndex] || null
})

function goToPreviousChunk(): void {
  if (activeChunkIndex.value > 0) {
    activeChunkIndex.value -= 1
  }
}

function goToNextChunk(): void {
  if (activeChunkIndex.value < parsedChunks.value.length - 1) {
    activeChunkIndex.value += 1
  }
}

function handleResolveCurrentChunk(strategy: 'ours' | 'theirs' | 'both_ours_first' | 'both_theirs_first'): void {
  if (!currentChunk.value) return
  editableContent.value = resolveConflictChunk(editableContent.value, currentChunk.value, strategy)
  if (activeChunkIndex.value >= parsedChunks.value.length && parsedChunks.value.length > 0) {
    activeChunkIndex.value = parsedChunks.value.length - 1
  }
}

const oursLineComparisons = computed(() => {
  return computeConflictLineComparison(
    props.conflictDetails?.ours_content || '',
    props.conflictDetails?.base_content || ''
  )
})

const theirsLineComparisons = computed(() => {
  return computeConflictLineComparison(
    props.conflictDetails?.theirs_content || '',
    props.conflictDetails?.base_content || ''
  )
})

const baseLines = computed(() => {
  if (!props.conflictDetails?.base_content) {
    return []
  }
  return props.conflictDetails.base_content.split(/\r?\n/)
})

const oursPaneReference = ref<HTMLDivElement | null>(null)
const basePaneReference = ref<HTMLDivElement | null>(null)
const theirsPaneReference = ref<HTMLDivElement | null>(null)

let isSyncingConflictScroll = false

function handleConflictPaneScroll(sourcePane: 'ours' | 'base' | 'theirs'): void {
  if (isSyncingConflictScroll) {
    return
  }
  isSyncingConflictScroll = true

  let targetScrollTop = 0
  if (sourcePane === 'ours' && oursPaneReference.value) {
    targetScrollTop = oursPaneReference.value.scrollTop
  } else if (sourcePane === 'base' && basePaneReference.value) {
    targetScrollTop = basePaneReference.value.scrollTop
  } else if (sourcePane === 'theirs' && theirsPaneReference.value) {
    targetScrollTop = theirsPaneReference.value.scrollTop
  }

  if (oursPaneReference.value && sourcePane !== 'ours') {
    oursPaneReference.value.scrollTop = targetScrollTop
  }
  if (basePaneReference.value && sourcePane !== 'base') {
    basePaneReference.value.scrollTop = targetScrollTop
  }
  if (theirsPaneReference.value && sourcePane !== 'theirs') {
    theirsPaneReference.value.scrollTop = targetScrollTop
  }

  requestAnimationFrame(() => {
    isSyncingConflictScroll = false
  })
}

function handleAcceptOurs(): void {
  emit('resolve', 'ours')
  emit('update:open', false)
}

function handleAcceptTheirs(): void {
  emit('resolve', 'theirs')
  emit('update:open', false)
}

function handleAcceptBoth(): void {
  if (!props.conflictDetails) {
    return
  }
  const ours = props.conflictDetails.ours_content || ''
  const theirs = props.conflictDetails.theirs_content || ''
  editableContent.value = combineOursAndTheirs(ours, theirs)
  activeViewMode.value = 'editor'
}

function handleSaveAndMarkResolved(): void {
  emit('saveContent', editableContent.value)
  emit('update:open', false)
}

function handleClose(): void {
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-7xl w-[96vw] p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden flex flex-col max-h-[92vh]',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col h-full overflow-hidden">
        <!-- Modal Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-4 py-3 bg-muted/20 shrink-0">
          <div class="flex items-center gap-2.5 min-w-0">
            <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-error/15 text-error">
              <UIcon name="i-lucide-git-merge" class="h-4 w-4" />
            </div>
            <div class="min-w-0">
              <div class="flex items-center gap-2">
                <h3 class="text-sm font-bold text-highlighted truncate">
                  {{ t('conflict.resolverTitle') }}
                </h3>
                <UBadge
                  v-if="remainingConflictCount > 0"
                  size="xs"
                  color="error"
                  variant="subtle"
                >
                  {{ t('conflict.markersRemaining', { count: remainingConflictCount }) }}
                </UBadge>
                <UBadge
                  v-else
                  size="xs"
                  color="success"
                  variant="subtle"
                >
                  {{ t('conflict.cleanNoMarkers') }}
                </UBadge>
              </div>
              <span class="font-mono text-xs text-muted truncate block" dir="ltr">
                {{ props.conflictDetails?.file_path }}
              </span>
            </div>
          </div>

          <!-- Mode Switcher & Navigation Controls -->
          <div class="flex items-center gap-3 shrink-0">
            <!-- Chunk Navigator (visible in Workbench mode) -->
            <div v-if="activeViewMode === 'workbench' && parsedChunks.length > 0" class="flex items-center gap-1.5 bg-surface/80 border border-default/70 px-2 py-1 rounded-lg">
              <span class="text-xs font-medium text-highlighted">
                {{ t('conflict.conflictCounter', { current: activeChunkIndex + 1, total: parsedChunks.length }) }}
              </span>
              <div class="flex items-center gap-0.5 ms-1">
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-chevron-up"
                  :disabled="activeChunkIndex <= 0"
                  :aria-label="t('conflict.jumpToPrevious')"
                  @click="goToPreviousChunk"
                />
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-chevron-down"
                  :disabled="activeChunkIndex >= parsedChunks.length - 1"
                  :aria-label="t('conflict.jumpToNext')"
                  @click="goToNextChunk"
                />
              </div>
            </div>

            <!-- Base Version Toggle -->
            <UButton
              v-if="props.conflictDetails?.base_content"
              size="xs"
              :color="isBaseVisible ? 'primary' : 'neutral'"
              :variant="isBaseVisible ? 'solid' : 'outline'"
              icon="i-lucide-git-commit"
              :label="t('conflict.toggleBase')"
              @click="isBaseVisible = !isBaseVisible"
            />

            <!-- Tab Switcher -->
            <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs">
              <button
                type="button"
                class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium transition-all"
                :class="activeViewMode === 'workbench' ? 'bg-primary text-inverted shadow-xs' : 'text-muted hover:text-highlighted'"
                @click="activeViewMode = 'workbench'"
              >
                <UIcon name="i-lucide-split" class="h-3.5 w-3.5" />
                <span>Workbench</span>
              </button>
              <button
                type="button"
                class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium transition-all"
                :class="activeViewMode === 'sideBySide' ? 'bg-primary text-inverted shadow-xs' : 'text-muted hover:text-highlighted'"
                @click="activeViewMode = 'sideBySide'"
              >
                <UIcon name="i-lucide-columns-2" class="h-3.5 w-3.5" />
                <span>{{ t('conflict.tabSideBySide') }}</span>
              </button>
              <button
                type="button"
                class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium transition-all"
                :class="activeViewMode === 'editor' ? 'bg-primary text-inverted shadow-xs' : 'text-muted hover:text-highlighted'"
                @click="activeViewMode = 'editor'"
              >
                <UIcon name="i-lucide-file-edit" class="h-3.5 w-3.5" />
                <span>{{ t('conflict.tabEditor') }}</span>
              </button>
            </div>

            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-x"
              @click="handleClose"
            />
          </div>
        </div>

        <!-- Modal Body -->
        <div class="flex-1 overflow-y-auto p-4 min-h-[460px] flex flex-col gap-4">
          <!-- WORKBENCH MODE (Interactive 3-Way Panes + Chunk Selector + Live Merged Output) -->
          <div v-if="activeViewMode === 'workbench'" class="flex flex-col gap-4 flex-1">
            <!-- Chunk Action Bar -->
            <div v-if="currentChunk" class="flex flex-wrap items-center justify-between p-3 rounded-xl border border-primary/30 bg-primary/5">
              <div class="flex items-center gap-2">
                <UIcon name="i-lucide-alert-circle" class="h-4 w-4 text-primary shrink-0" />
                <span class="text-xs font-semibold text-highlighted">
                  {{ t('conflict.resolveChunk') }} (L{{ currentChunk.startLine }} - L{{ currentChunk.endLine }}):
                </span>
              </div>
              <div class="flex flex-wrap items-center gap-1.5">
                <UButton
                  size="xs"
                  color="primary"
                  variant="solid"
                  icon="i-lucide-arrow-left"
                  :label="t('conflict.acceptOurs')"
                  @click="handleResolveCurrentChunk('ours')"
                />
                <UButton
                  size="xs"
                  color="warning"
                  variant="solid"
                  icon="i-lucide-arrow-right"
                  :label="t('conflict.acceptTheirs')"
                  @click="handleResolveCurrentChunk('theirs')"
                />
                <UButton
                  size="xs"
                  color="neutral"
                  variant="outline"
                  icon="i-lucide-combine"
                  :label="t('conflict.acceptBothOursFirst')"
                  @click="handleResolveCurrentChunk('both_ours_first')"
                />
                <UButton
                  size="xs"
                  color="neutral"
                  variant="outline"
                  icon="i-lucide-combine"
                  :label="t('conflict.acceptBothTheirsFirst')"
                  @click="handleResolveCurrentChunk('both_theirs_first')"
                />
              </div>
            </div>

            <!-- 3-Way Top Comparison (Ours | Base | Theirs) -->
            <div
              class="grid gap-3 flex-1 min-h-[220px]"
              :class="isBaseVisible ? 'grid-cols-3' : 'grid-cols-2'"
            >
              <!-- Ours Pane -->
              <div class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60">
                <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-primary/10">
                  <span class="text-xs font-semibold text-primary flex items-center gap-1.5">
                    <UIcon name="i-lucide-arrow-left" class="h-3.5 w-3.5 rtl:rotate-180" />
                    {{ t('conflict.paneOurs') }}
                  </span>
                  <span v-if="currentChunk" class="font-mono text-[10px] text-muted">
                    {{ currentChunk.oursBranchName }}
                  </span>
                </div>
                <div class="flex-1 p-3 font-mono text-xs overflow-auto whitespace-pre bg-surface/30 select-text" dir="ltr">
                  <template v-if="currentChunk">
                    {{ currentChunk.oursContent || '(empty)' }}
                  </template>
                  <template v-else>
                    <div class="text-muted text-center pt-8">
                      {{ t('conflict.noConflictsFound') }}
                    </div>
                  </template>
                </div>
              </div>

              <!-- Base Pane -->
              <div v-if="isBaseVisible" class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60">
                <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-muted/20">
                  <span class="text-xs font-semibold text-muted flex items-center gap-1.5">
                    <UIcon name="i-lucide-git-commit" class="h-3.5 w-3.5" />
                    {{ t('conflict.paneBase') }}
                  </span>
                </div>
                <div class="flex-1 p-3 font-mono text-xs overflow-auto whitespace-pre bg-surface/30 text-muted select-text" dir="ltr">
                  {{ props.conflictDetails?.base_content || '(no base content available)' }}
                </div>
              </div>

              <!-- Theirs Pane -->
              <div class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60">
                <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-warning/10">
                  <span class="text-xs font-semibold text-warning flex items-center gap-1.5">
                    <UIcon name="i-lucide-arrow-right" class="h-3.5 w-3.5 rtl:rotate-180" />
                    {{ t('conflict.paneTheirs') }}
                  </span>
                  <span v-if="currentChunk" class="font-mono text-[10px] text-muted">
                    {{ currentChunk.theirsBranchName }}
                  </span>
                </div>
                <div class="flex-1 p-3 font-mono text-xs overflow-auto whitespace-pre bg-surface/30 select-text" dir="ltr">
                  <template v-if="currentChunk">
                    {{ currentChunk.theirsContent || '(empty)' }}
                  </template>
                  <template v-else>
                    <div class="text-muted text-center pt-8">
                      {{ t('conflict.noConflictsFound') }}
                    </div>
                  </template>
                </div>
              </div>
            </div>

            <!-- Merged Output Result Pane -->
            <div class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/80 h-52 shrink-0">
              <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-muted/20">
                <span class="text-xs font-semibold text-highlighted flex items-center gap-1.5">
                  <UIcon name="i-lucide-file-code" class="h-3.5 w-3.5 text-primary" />
                  {{ t('conflict.paneResult') }}
                </span>
                <span class="text-[10px] text-muted font-mono">
                  {{ t('conflict.editorHint') }}
                </span>
              </div>
              <textarea
                v-model="editableContent"
                rows="8"
                wrap="off"
                class="w-full flex-1 p-3 font-mono text-xs bg-transparent text-highlighted focus:outline-none select-text leading-relaxed resize-none overflow-x-auto whitespace-pre"
                dir="ltr"
              />
            </div>
          </div>

          <!-- SIDE-BY-SIDE 3-COLOR VIEW -->
          <div
            v-else-if="activeViewMode === 'sideBySide'"
            class="grid gap-3 h-full min-h-[400px]"
            :class="isBaseVisible ? 'grid-cols-3' : 'grid-cols-2'"
          >
            <!-- Ours Line By Line with 3-Color -->
            <div class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60">
              <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-primary/10">
                <span class="text-xs font-semibold text-primary flex items-center gap-1.5">
                  <UIcon name="i-lucide-arrow-left" class="h-3.5 w-3.5 rtl:rotate-180" />
                  {{ t('conflict.oursTitle') }}
                </span>
                <UButton
                  size="xs"
                  color="primary"
                  variant="subtle"
                  :label="t('conflict.acceptOurs')"
                  @click="handleAcceptOurs"
                />
              </div>
              <div
                ref="oursPaneReference"
                class="flex-1 overflow-auto divide-y divide-default/15 font-mono text-xs select-text"
                dir="ltr"
                @scroll="handleConflictPaneScroll('ours')"
              >
                <div class="min-w-fit w-full divide-y divide-default/15">
                  <div
                    v-for="(line, lineIndex) in oursLineComparisons"
                    :key="`ours-line-${lineIndex}`"
                    class="flex items-center h-[22px] min-h-[22px] max-h-[22px] leading-[22px]"
                    :class="{
                      'bg-[var(--gv-diff-modify-bg)] text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]': line.status === 'modification',
                      'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': line.status === 'addition',
                      'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': line.status === 'deletion',
                      'text-highlighted border-s-2 border-transparent': line.status === 'context',
                    }"
                  >
                    <div class="w-10 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px]">
                      {{ line.lineNumber || '' }}
                    </div>
                    <div class="w-5 shrink-0 text-center select-none font-bold">
                      <span v-if="line.status === 'modification'">~</span>
                      <span v-else-if="line.status === 'addition'">+</span>
                      <span v-else-if="line.status === 'deletion'">-</span>
                      <span v-else>&nbsp;</span>
                    </div>
                    <div class="flex-1 px-1.5 whitespace-pre overflow-x-visible leading-[22px]">
                      <template v-if="line.spans && line.spans.length">
                        <span
                          v-for="(span, spanIndex) in line.spans"
                          :key="`span-ours-${lineIndex}-${spanIndex}`"
                          :class="{
                            'bg-[var(--gv-diff-modify-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.status === 'modification',
                            'bg-[var(--gv-diff-add-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.status === 'addition',
                            'bg-[var(--gv-diff-remove-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.status === 'deletion',
                          }"
                        >{{ span.text }}</span>
                      </template>
                      <template v-else>
                        {{ line.content }}
                      </template>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Base Pane -->
            <div
              v-if="isBaseVisible"
              class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60"
            >
              <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-muted/20">
                <span class="text-xs font-semibold text-muted flex items-center gap-1.5">
                  <UIcon name="i-lucide-git-commit" class="h-3.5 w-3.5" />
                  {{ t('conflict.baseTitle') }}
                </span>
              </div>
              <div
                ref="basePaneReference"
                class="flex-1 overflow-auto divide-y divide-default/15 font-mono text-xs select-text"
                dir="ltr"
                @scroll="handleConflictPaneScroll('base')"
              >
                <div class="min-w-fit w-full divide-y divide-default/15">
                  <div
                    v-for="(line, lineIndex) in baseLines"
                    :key="`base-line-${lineIndex}`"
                    class="flex items-center h-[22px] min-h-[22px] max-h-[22px] leading-[22px] text-muted border-s-2 border-transparent"
                  >
                    <div class="w-10 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px]">
                      {{ lineIndex + 1 }}
                    </div>
                    <div class="w-5 shrink-0 text-center select-none font-bold text-muted/40">
                      &nbsp;
                    </div>
                    <div class="flex-1 px-1.5 whitespace-pre overflow-x-visible leading-[22px]">
                      {{ line }}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Theirs Line By Line with 3-Color -->
            <div class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60">
              <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-warning/10">
                <span class="text-xs font-semibold text-warning flex items-center gap-1.5">
                  <UIcon name="i-lucide-arrow-right" class="h-3.5 w-3.5 rtl:rotate-180" />
                  {{ t('conflict.theirsTitle') }}
                </span>
                <UButton
                  size="xs"
                  color="warning"
                  variant="subtle"
                  :label="t('conflict.acceptTheirs')"
                  @click="handleAcceptTheirs"
                />
              </div>
              <div
                ref="theirsPaneReference"
                class="flex-1 overflow-auto divide-y divide-default/15 font-mono text-xs select-text"
                dir="ltr"
                @scroll="handleConflictPaneScroll('theirs')"
              >
                <div class="min-w-fit w-full divide-y divide-default/15">
                  <div
                    v-for="(line, lineIndex) in theirsLineComparisons"
                    :key="`theirs-line-${lineIndex}`"
                    class="flex items-center h-[22px] min-h-[22px] max-h-[22px] leading-[22px]"
                    :class="{
                      'bg-[var(--gv-diff-modify-bg)] text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]': line.status === 'modification',
                      'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': line.status === 'addition',
                      'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': line.status === 'deletion',
                      'text-highlighted border-s-2 border-transparent': line.status === 'context',
                    }"
                  >
                    <div class="w-10 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px]">
                      {{ line.lineNumber || '' }}
                    </div>
                    <div class="w-5 shrink-0 text-center select-none font-bold">
                      <span v-if="line.status === 'modification'">~</span>
                      <span v-else-if="line.status === 'addition'">+</span>
                      <span v-else-if="line.status === 'deletion'">-</span>
                      <span v-else>&nbsp;</span>
                    </div>
                    <div class="flex-1 px-1.5 whitespace-pre overflow-x-visible leading-[22px]">
                      <template v-if="line.spans && line.spans.length">
                        <span
                          v-for="(span, spanIndex) in line.spans"
                          :key="`span-theirs-${lineIndex}-${spanIndex}`"
                          :class="{
                            'bg-[var(--gv-diff-modify-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.status === 'modification',
                            'bg-[var(--gv-diff-add-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.status === 'addition',
                            'bg-[var(--gv-diff-remove-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.status === 'deletion',
                          }"
                        >{{ span.text }}</span>
                      </template>
                      <template v-else>
                        {{ line.content }}
                      </template>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- FULL RESULT EDITOR VIEW -->
          <div v-else class="flex flex-col h-full min-h-[400px] space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="text-xs font-semibold text-highlighted">
                  {{ t('conflict.editorHint') }}
                </span>
                <UBadge
                  v-if="stillHasConflictMarkers"
                  size="xs"
                  color="error"
                  variant="soft"
                >
                  {{ t('conflict.markersRemaining', { count: remainingConflictCount }) }}
                </UBadge>
                <UBadge
                  v-else
                  size="xs"
                  color="success"
                  variant="soft"
                >
                  {{ t('conflict.cleanNoMarkers') }}
                </UBadge>
              </div>

              <UButton
                size="xs"
                color="neutral"
                variant="outline"
                icon="i-lucide-combine"
                :label="t('conflict.acceptBoth')"
                @click="handleAcceptBoth"
              />
            </div>

            <textarea
              v-model="editableContent"
              rows="18"
              wrap="off"
              class="w-full flex-1 p-3 font-mono text-xs rounded-xl border border-default/70 bg-surface/90 text-highlighted focus:outline-none focus:ring-1 focus:ring-primary select-text leading-relaxed resize-none overflow-x-auto whitespace-pre"
              dir="ltr"
            />
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="flex items-center justify-between border-t border-default/70 px-4 py-3 bg-muted/20 shrink-0">
          <div class="flex items-center gap-2">
            <UButton
              size="xs"
              color="primary"
              variant="outline"
              :label="t('conflict.acceptOurs')"
              @click="handleAcceptOurs"
            />
            <UButton
              size="xs"
              color="warning"
              variant="outline"
              :label="t('conflict.acceptTheirs')"
              @click="handleAcceptTheirs"
            />
            <UButton
              size="xs"
              color="neutral"
              variant="outline"
              :label="t('conflict.acceptBoth')"
              @click="handleAcceptBoth"
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
              color="success"
              variant="solid"
              icon="i-lucide-check-check"
              :disabled="stillHasConflictMarkers"
              :label="t('conflict.markResolved')"
              @click="handleSaveAndMarkResolved"
            />
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>
