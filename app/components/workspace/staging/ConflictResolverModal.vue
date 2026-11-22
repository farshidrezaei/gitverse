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
import ConflictWorkbenchView from './ConflictWorkbenchView.vue'
import ConflictSideBySideView from './ConflictSideBySideView.vue'
import ConflictResultEditorView from './ConflictResultEditorView.vue'

interface ConflictResolverModalProps {
  open: boolean
  conflictDetails: ConflictFileDetails | null
  showContinueAction?: boolean
}

const props = withDefaults(defineProps<ConflictResolverModalProps>(), {
  showContinueAction: false,
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  resolve: [strategy: 'ours' | 'theirs' | 'mark_resolved']
  saveContent: [content: string]
  saveAndContinue: [content: string]
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

function handleSaveAndContinue(): void {
  emit('saveAndContinue', editableContent.value)
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
          <!-- WORKBENCH MODE -->
          <ConflictWorkbenchView
            v-if="activeViewMode === 'workbench'"
            v-model:editable-content="editableContent"
            :current-chunk="currentChunk"
            :conflict-details="props.conflictDetails"
            :is-base-visible="isBaseVisible"
            @resolve-chunk="handleResolveCurrentChunk"
          />

          <!-- SIDE-BY-SIDE 3-COLOR VIEW -->
          <ConflictSideBySideView
            v-else-if="activeViewMode === 'sideBySide'"
            :ours-line-comparisons="oursLineComparisons"
            :theirs-line-comparisons="theirsLineComparisons"
            :base-lines="baseLines"
            :is-base-visible="isBaseVisible"
            @accept-ours="handleAcceptOurs"
            @accept-theirs="handleAcceptTheirs"
          />

          <!-- FULL RESULT EDITOR VIEW -->
          <ConflictResultEditorView
            v-else
            v-model:editable-content="editableContent"
            :still-has-conflict-markers="stillHasConflictMarkers"
            :remaining-conflict-count="remainingConflictCount"
            @accept-both="handleAcceptBoth"
          />
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
              variant="outline"
              icon="i-lucide-check"
              :disabled="stillHasConflictMarkers"
              :label="t('conflict.markResolved')"
              @click="handleSaveAndMarkResolved"
            />
            <UButton
              v-if="props.showContinueAction"
              size="xs"
              color="success"
              variant="solid"
              icon="i-lucide-play"
              :disabled="stillHasConflictMarkers"
              :label="t('conflict.saveAndContinue')"
              @click="handleSaveAndContinue"
            />
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>
