<script setup lang="ts">
import type { FileDiff } from '~/types/repository'
import { buildSideBySideHunk } from '~/utils/diffLayout'

interface DiffSplitViewProps {
  diff: FileDiff
  isWordWrap?: boolean
  isStaged?: boolean | null
}

const props = withDefaults(defineProps<DiffSplitViewProps>(), {
  isWordWrap: false,
  isStaged: null,
})

const emit = defineEmits<{
  stageHunk: [hunkIndex: number]
  unstageHunk: [hunkIndex: number]
  discardHunk: [hunkIndex: number]
  stageLine: [hunkIndex: number, lineIndex: number]
  unstageLine: [hunkIndex: number, lineIndex: number]
  discardLine: [hunkIndex: number, lineIndex: number]
}>()

const { t } = useI18n()

const INITIAL_VISIBLE_ROWS_LIMIT = 500
const isFullyExpanded = ref<boolean>(false)

const leftPaneReference = ref<HTMLDivElement | null>(null)
const rightPaneReference = ref<HTMLDivElement | null>(null)

let isSyncingLeftScroll = false
let isSyncingRightScroll = false

function handleLeftScroll(): void {
  if (isSyncingLeftScroll) {
    isSyncingLeftScroll = false
    return
  }
  if (!leftPaneReference.value || !rightPaneReference.value) return

  isSyncingRightScroll = true
  rightPaneReference.value.scrollTop = leftPaneReference.value.scrollTop
}

function handleRightScroll(): void {
  if (isSyncingRightScroll) {
    isSyncingRightScroll = false
    return
  }
  if (!leftPaneReference.value || !rightPaneReference.value) return

  isSyncingLeftScroll = true
  leftPaneReference.value.scrollTop = rightPaneReference.value.scrollTop
}

const sideBySideHunks = computed(() => {
  if (!props.diff || !props.diff.hunks) return []
  return props.diff.hunks.map((hunk) => buildSideBySideHunk(hunk))
})

const totalRowsCount = computed(() => {
  return sideBySideHunks.value.reduce((acc, hunk) => acc + hunk.rows.length, 0)
})

const isTruncated = computed(() => {
  return !isFullyExpanded.value && totalRowsCount.value > INITIAL_VISIBLE_ROWS_LIMIT
})

const displayedHunks = computed(() => {
  if (!isTruncated.value) {
    return sideBySideHunks.value
  }

  let remaining = INITIAL_VISIBLE_ROWS_LIMIT
  const truncated = []

  for (const hunk of sideBySideHunks.value) {
    if (remaining <= 0) break
    if (hunk.rows.length <= remaining) {
      truncated.push(hunk)
      remaining -= hunk.rows.length
    } else {
      truncated.push({
        ...hunk,
        rows: hunk.rows.slice(0, remaining),
      })
      remaining = 0
    }
  }

  return truncated
})
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden select-text font-mono text-[11px]">
    <!-- Main Dual-Pane Scroll Container with Synchronized Vertical Scroll -->
    <div class="flex flex-1 min-h-0 w-full overflow-hidden">
      <!-- Left Pane: Base / Old -->
      <div
        ref="leftPaneReference"
        class="flex-1 h-full overflow-y-auto overflow-x-auto border-e border-default/40 bg-[var(--gv-surface)]"
        @scroll="handleLeftScroll"
      >
        <div class="min-w-fit w-full">
          <div
            v-for="(hunk, hunkIndex) in displayedHunks"
            :key="`sbs-left-hunk-${hunkIndex}`"
          >
            <!-- Left Hunk Header (Exact 24px height matches right pane) -->
            <div class="h-6 flex items-center bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 text-[11px] font-semibold select-none border-b border-default/40">
              <span class="truncate font-mono">{{ hunk.header }}</span>
            </div>

            <!-- Left Line Rows (Fixed 22px height prevents vertical text collisions) -->
            <div
              v-for="row in hunk.rows"
              :key="`left-${row.id}`"
              class="h-[22px] min-h-[22px] max-h-[22px] flex items-center border-b border-default/15 group"
              :class="{
                'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': row.left.lineType === 'deletion',
                'bg-[var(--gv-diff-modify-bg)] text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]': row.left.lineType === 'modification',
                'bg-muted/10 opacity-30': row.left.lineType === 'empty',
                'text-highlighted border-s-2 border-transparent': row.left.lineType === 'context',
              }"
            >
              <!-- Old Line Number -->
              <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px] leading-[22px]">
                {{ row.left.lineNumber || '' }}
              </div>

              <!-- Old Gutter Prefix & Line Action -->
              <div class="w-5 shrink-0 flex items-center justify-center select-none font-bold leading-[22px]">
                <template v-if="props.isStaged !== null && row.left.originalLineIndex !== undefined && (row.left.lineType === 'deletion' || row.left.lineType === 'modification')">
                  <span class="group-hover:hidden">
                    <span v-if="row.left.lineType === 'deletion'">-</span>
                    <span v-else-if="row.left.lineType === 'modification'">~</span>
                  </span>
                  <button
                    v-if="!props.isStaged"
                    type="button"
                    class="hidden group-hover:flex items-center justify-center h-4 w-4 rounded-xs bg-primary/20 text-primary hover:bg-primary hover:text-white transition-colors cursor-pointer"
                    :title="t('diff.stageLine')"
                    @click.stop="emit('stageLine', hunkIndex, row.left.originalLineIndex)"
                  >
                    <UIcon name="i-lucide-plus" class="h-3 w-3" />
                  </button>
                  <button
                    v-else
                    type="button"
                    class="hidden group-hover:flex items-center justify-center h-4 w-4 rounded-xs bg-error/20 text-error hover:bg-error hover:text-white transition-colors cursor-pointer"
                    :title="t('diff.unstageLine')"
                    @click.stop="emit('unstageLine', hunkIndex, row.left.originalLineIndex)"
                  >
                    <UIcon name="i-lucide-minus" class="h-3 w-3" />
                  </button>
                </template>
                <template v-else>
                  <span v-if="row.left.lineType === 'deletion'">-</span>
                  <span v-else-if="row.left.lineType === 'modification'">~</span>
                  <span v-else>&nbsp;</span>
                </template>
              </div>

              <!-- Old Line Content (Smooth independent horizontal scroll) -->
              <div
                class="flex-1 pe-4 font-mono leading-[22px]"
                :class="props.isWordWrap ? 'whitespace-pre-wrap break-all' : 'whitespace-pre overflow-x-visible'"
              >
                <template v-if="row.left.spans && row.left.spans.length">
                  <span
                    v-for="(span, spanIndex) in row.left.spans"
                    :key="`left-span-${spanIndex}`"
                    :class="{
                      'bg-[var(--gv-diff-modify-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && row.left.lineType === 'modification',
                      'bg-[var(--gv-diff-remove-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && row.left.lineType === 'deletion',
                    }"
                  >{{ span.text }}</span>
                </template>
                <template v-else>
                  {{ row.left.content }}
                </template>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Pane: Modified / New -->
      <div
        ref="rightPaneReference"
        class="flex-1 h-full overflow-y-auto overflow-x-auto bg-[var(--gv-surface)]"
        @scroll="handleRightScroll"
      >
        <div class="min-w-fit w-full">
          <div
            v-for="(hunk, hunkIndex) in displayedHunks"
            :key="`sbs-right-hunk-${hunkIndex}`"
          >
            <!-- Hunk Header (Exact 24px height on both sides) -->
            <div class="h-6 flex items-center justify-between bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 text-[11px] font-semibold select-none border-b border-default/40">
              <span class="truncate font-mono">{{ hunk.header }}</span>

              <!-- Hunk Action Controls -->
              <div v-if="props.isStaged !== null" class="flex items-center gap-1.5 opacity-80 hover:opacity-100 transition-opacity">
                <template v-if="!props.isStaged">
                  <button
                    type="button"
                    class="flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded-xs bg-primary/20 text-primary hover:bg-primary hover:text-white transition-colors cursor-pointer"
                    :title="t('diff.stageHunk')"
                    @click.stop="emit('stageHunk', hunkIndex)"
                  >
                    <UIcon name="i-lucide-plus" class="h-3 w-3" />
                    <span>{{ t('diff.stageHunk') }}</span>
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded-xs bg-error/20 text-error hover:bg-error hover:text-white transition-colors cursor-pointer"
                    :title="t('diff.discardHunk')"
                    @click.stop="emit('discardHunk', hunkIndex)"
                  >
                    <UIcon name="i-lucide-undo-2" class="h-3 w-3" />
                    <span>{{ t('diff.discardHunk') }}</span>
                  </button>
                </template>
                <template v-else>
                  <button
                    type="button"
                    class="flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded-xs bg-neutral-500/20 text-neutral-300 hover:bg-neutral-600 hover:text-white transition-colors cursor-pointer"
                    :title="t('diff.unstageHunk')"
                    @click.stop="emit('unstageHunk', hunkIndex)"
                  >
                    <UIcon name="i-lucide-minus" class="h-3 w-3" />
                    <span>{{ t('diff.unstageHunk') }}</span>
                  </button>
                </template>
              </div>
            </div>

            <!-- Right Line Rows (Fixed 22px height prevents vertical text collisions) -->
            <div
              v-for="row in hunk.rows"
              :key="`right-${row.id}`"
              class="h-[22px] min-h-[22px] max-h-[22px] flex items-center border-b border-default/15 group"
              :class="{
                'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': row.right.lineType === 'addition',
                'bg-[var(--gv-diff-modify-bg)] text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]': row.right.lineType === 'modification',
                'bg-muted/10 opacity-30': row.right.lineType === 'empty',
                'text-highlighted border-s-2 border-transparent': row.right.lineType === 'context',
              }"
            >
              <!-- New Line Number -->
              <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/20 text-[10px] leading-[22px]">
                {{ row.right.lineNumber || '' }}
              </div>

              <!-- New Gutter Prefix & Line Action -->
              <div class="w-5 shrink-0 flex items-center justify-center select-none font-bold leading-[22px]">
                <template v-if="props.isStaged !== null && row.right.originalLineIndex !== undefined && (row.right.lineType === 'addition' || row.right.lineType === 'modification')">
                  <span class="group-hover:hidden">
                    <span v-if="row.right.lineType === 'addition'">+</span>
                    <span v-else-if="row.right.lineType === 'modification'">~</span>
                  </span>
                  <button
                    v-if="!props.isStaged"
                    type="button"
                    class="hidden group-hover:flex items-center justify-center h-4 w-4 rounded-xs bg-primary/20 text-primary hover:bg-primary hover:text-white transition-colors cursor-pointer"
                    :title="t('diff.stageLine')"
                    @click.stop="emit('stageLine', hunkIndex, row.right.originalLineIndex)"
                  >
                    <UIcon name="i-lucide-plus" class="h-3 w-3" />
                  </button>
                  <button
                    v-else
                    type="button"
                    class="hidden group-hover:flex items-center justify-center h-4 w-4 rounded-xs bg-error/20 text-error hover:bg-error hover:text-white transition-colors cursor-pointer"
                    :title="t('diff.unstageLine')"
                    @click.stop="emit('unstageLine', hunkIndex, row.right.originalLineIndex)"
                  >
                    <UIcon name="i-lucide-minus" class="h-3 w-3" />
                  </button>
                </template>
                <template v-else>
                  <span v-if="row.right.lineType === 'addition'">+</span>
                  <span v-else-if="row.right.lineType === 'modification'">~</span>
                  <span v-else>&nbsp;</span>
                </template>
              </div>

              <!-- New Line Content (Smooth independent horizontal scroll) -->
              <div
                class="flex-1 pe-4 font-mono leading-[22px]"
                :class="props.isWordWrap ? 'whitespace-pre-wrap break-all' : 'whitespace-pre overflow-x-visible'"
              >
                <template v-if="row.right.spans && row.right.spans.length">
                  <span
                    v-for="(span, spanIndex) in row.right.spans"
                    :key="`right-span-${spanIndex}`"
                    :class="{
                      'bg-[var(--gv-diff-modify-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && row.right.lineType === 'modification',
                      'bg-[var(--gv-diff-add-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && row.right.lineType === 'addition',
                    }"
                  >{{ span.text }}</span>
                </template>
                <template v-else>
                  {{ row.right.content }}
                </template>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Expansion Banner for Large Diffs -->
    <div
      v-if="isTruncated"
      class="shrink-0 p-3 bg-muted/20 border-t border-default/40 flex items-center justify-between gap-3 text-xs"
    >
      <span class="text-muted">
        Showing first {{ INITIAL_VISIBLE_ROWS_LIMIT }} of {{ totalRowsCount }} rows for performance
      </span>
      <UButton
        size="xs"
        color="primary"
        variant="outline"
        icon="i-lucide-chevrons-down"
        label="Load full diff"
        @click="isFullyExpanded = true"
      />
    </div>
  </div>
</template>
