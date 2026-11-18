<script setup lang="ts">
import type { ConflictFileDetails } from '~/types/repository'
import {
  combineOursAndTheirs,
  computeConflictLineComparison,
  countConflictMarkers,
  hasConflictMarkers,
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

const activeTab = ref<'diff' | 'editor'>('diff')
const editableContent = ref<string>('')
const isBaseVisible = ref<boolean>(false)

watch(
  () => props.conflictDetails,
  (newDetails) => {
    if (newDetails) {
      editableContent.value = newDetails.working_file_content || ''
      activeTab.value = 'diff'
    }
  },
  { immediate: true }
)

const remainingConflictCount = computed(() => {
  return countConflictMarkers(editableContent.value)
})

const stillHasConflictMarkers = computed(() => {
  return hasConflictMarkers(editableContent.value)
})

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
  activeTab.value = 'editor'
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
      content: 'sm:max-w-6xl w-[94vw] p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden flex flex-col max-h-[90vh]',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col h-full overflow-hidden">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-default/70 px-4 py-3 bg-muted/20">
        <div class="flex items-center gap-2.5 min-w-0">
          <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-error/15 text-error">
            <UIcon name="i-lucide-git-merge" class="h-4 w-4" />
          </div>
          <div class="min-w-0">
            <h3 class="text-sm font-bold text-highlighted truncate">
              {{ t('conflict.resolverTitle') }}
            </h3>
            <span class="font-mono text-xs text-muted truncate block" dir="ltr">
              {{ props.conflictDetails?.file_path }}
            </span>
          </div>
        </div>

        <!-- View mode tabs & Controls -->
        <div class="flex items-center gap-3 shrink-0">
          <!-- 3-Color Diff Legend -->
          <div v-if="activeTab === 'diff'" class="hidden md:flex items-center gap-2 text-[10px] text-muted">
            <span class="flex items-center gap-1 font-mono">
              <span class="inline-block h-2 w-2 rounded-xs bg-[var(--gv-diff-add-text)]" />
              {{ t('conflict.legendAdded') }}
            </span>
            <span class="flex items-center gap-1 font-mono">
              <span class="inline-block h-2 w-2 rounded-xs bg-[var(--gv-diff-modify-text)]" />
              {{ t('conflict.legendModified') }}
            </span>
            <span class="flex items-center gap-1 font-mono">
              <span class="inline-block h-2 w-2 rounded-xs bg-[var(--gv-diff-remove-text)]" />
              {{ t('conflict.legendDeleted') }}
            </span>
          </div>

          <!-- Base Version Toggle -->
          <UButton
            v-if="activeTab === 'diff' && props.conflictDetails?.base_content"
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
              :class="activeTab === 'diff' ? 'bg-primary text-inverted shadow-xs' : 'text-muted hover:text-highlighted'"
              @click="activeTab = 'diff'"
            >
              <UIcon name="i-lucide-columns-2" class="h-3.5 w-3.5" />
              <span>{{ t('conflict.tabSideBySide') }}</span>
            </button>
            <button
              type="button"
              class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium transition-all"
              :class="activeTab === 'editor' ? 'bg-primary text-inverted shadow-xs' : 'text-muted hover:text-highlighted'"
              @click="activeTab = 'editor'"
            >
              <UIcon name="i-lucide-file-edit" class="h-3.5 w-3.5" />
              <span>{{ t('conflict.tabEditor') }}</span>
              <UBadge
                v-if="remainingConflictCount > 0"
                size="xs"
                color="error"
                variant="subtle"
                class="ms-1"
              >
                {{ remainingConflictCount }}
              </UBadge>
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
      <div class="flex-1 overflow-y-auto p-4 min-h-[420px]">
        <!-- Side-by-Side 3-Color View -->
        <div
          v-if="activeTab === 'diff'"
          class="grid gap-3 h-full min-h-[400px]"
          :class="isBaseVisible ? 'grid-cols-3' : 'grid-cols-2'"
        >
          <!-- Pane 1: Ours / Current -->
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

            <!-- Ours Line By Line with 3-Color & Intra-line Diff -->
            <div class="flex-1 overflow-auto divide-y divide-default/15 font-mono text-xs select-text" dir="ltr">
              <div class="min-w-fit w-full divide-y divide-default/15">
                <div
                  v-for="(line, lineIndex) in oursLineComparisons"
                  :key="`ours-line-${lineIndex}`"
                  class="flex items-stretch min-h-[22px] leading-relaxed"
                  :class="{
                    'bg-[var(--gv-diff-modify-bg)] text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]': line.status === 'modification',
                    'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': line.status === 'addition',
                    'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': line.status === 'deletion',
                    'text-highlighted border-s-2 border-transparent': line.status === 'context',
                  }"
                >
                  <!-- Line Number -->
                  <div class="w-10 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px]">
                    {{ line.lineNumber || '' }}
                  </div>
                  <!-- Gutter Sign -->
                  <div class="w-5 shrink-0 text-center select-none font-bold">
                    <span v-if="line.status === 'modification'">~</span>
                    <span v-else-if="line.status === 'addition'">+</span>
                    <span v-else-if="line.status === 'deletion'">-</span>
                    <span v-else>&nbsp;</span>
                  </div>
                  <!-- Spans with Intra-line Highlighting -->
                  <div class="flex-1 px-1.5 whitespace-pre overflow-x-visible">
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

          <!-- Pane 2: Base / Ancestor (Optional) -->
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

            <!-- Base Lines List -->
            <div class="flex-1 overflow-auto divide-y divide-default/15 font-mono text-xs select-text" dir="ltr">
              <div
                v-for="(line, lineIndex) in baseLines"
                :key="`base-line-${lineIndex}`"
                class="flex items-stretch min-h-[22px] leading-relaxed text-muted border-s-2 border-transparent"
              >
                <div class="w-10 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px]">
                  {{ lineIndex + 1 }}
                </div>
                <div class="w-5 shrink-0 text-center select-none font-bold text-muted/40">
                  &nbsp;
                </div>
                <div class="flex-1 px-1.5 whitespace-pre truncate">
                  {{ line }}
                </div>
              </div>
            </div>
          </div>

          <!-- Pane 3: Theirs / Incoming -->
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

            <!-- Theirs Line By Line with 3-Color & Intra-line Diff -->
            <div class="flex-1 overflow-auto divide-y divide-default/15 font-mono text-xs select-text" dir="ltr">
              <div
                v-for="(line, lineIndex) in theirsLineComparisons"
                :key="`theirs-line-${lineIndex}`"
                class="flex items-stretch min-h-[22px] leading-relaxed"
                :class="{
                  'bg-[var(--gv-diff-modify-bg)] text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]': line.status === 'modification',
                  'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': line.status === 'addition',
                  'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': line.status === 'deletion',
                  'text-highlighted border-s-2 border-transparent': line.status === 'context',
                }"
              >
                <!-- Line Number -->
                <div class="w-10 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px]">
                  {{ line.lineNumber || '' }}
                </div>
                <!-- Gutter Sign -->
                <div class="w-5 shrink-0 text-center select-none font-bold">
                  <span v-if="line.status === 'modification'">~</span>
                  <span v-else-if="line.status === 'addition'">+</span>
                  <span v-else-if="line.status === 'deletion'">-</span>
                  <span v-else>&nbsp;</span>
                </div>
                <!-- Spans with Intra-line Highlighting -->
                <div class="flex-1 px-1.5 whitespace-pre truncate">
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

        <!-- Result Editor View -->
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
            class="w-full flex-1 p-3 font-mono text-xs rounded-xl border border-default/70 bg-surface/90 text-highlighted focus:outline-none focus:ring-1 focus:ring-primary select-text leading-relaxed resize-none"
            dir="ltr"
          />
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-between border-t border-default/70 px-4 py-3 bg-muted/20">
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
