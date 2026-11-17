<script setup lang="ts">
import type { FileDiff } from '~/types/repository'
import { buildSideBySideHunk } from '~/utils/diffLayout'

interface DiffSplitViewProps {
  diff: FileDiff
}

const props = defineProps<DiffSplitViewProps>()

const INITIAL_VISIBLE_ROWS_LIMIT = 500
const isFullyExpanded = ref<boolean>(false)

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
  <div class="min-w-fit">
    <div
      v-for="(hunk, hunkIndex) in displayedHunks"
      :key="`sbs-hunk-${hunkIndex}`"
      class="border-b border-default/50"
    >
      <!-- Hunk Header -->
      <div class="flex items-center bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 py-1 font-semibold select-none border-y border-default/40">
        <span class="font-mono text-[11px]">{{ hunk.header }}</span>
      </div>

      <!-- Side-by-Side Paired Rows with Content-Visibility Optimization -->
      <div
        v-for="row in hunk.rows"
        :key="row.id"
        class="grid grid-cols-2 divide-x divide-default/50 border-b border-default/20 last:border-b-0"
        style="content-visibility: auto; contain-intrinsic-size: 0 20px;"
      >
        <!-- Left Side: Base / Old -->
        <div
          class="flex items-stretch overflow-hidden"
          :class="{
            'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': row.left.lineType === 'deletion',
            'bg-muted/10 opacity-40': row.left.lineType === 'empty',
            'text-highlighted border-s-2 border-transparent': row.left.lineType === 'context',
          }"
        >
          <!-- Old Line Number -->
          <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/30 font-mono text-[10px]">
            {{ row.left.lineNumber || '' }}
          </div>

          <!-- Old Gutter Prefix -->
          <div class="w-4 shrink-0 text-center select-none font-bold font-mono">
            <span v-if="row.left.lineType === 'deletion'">-</span>
            <span v-else-if="row.left.lineType === 'context'">&nbsp;</span>
          </div>

          <!-- Old Content -->
          <div class="flex-1 pe-2 whitespace-pre font-mono select-text truncate">
            {{ row.left.content }}
          </div>
        </div>

        <!-- Right Side: Modified / New -->
        <div
          class="flex items-stretch overflow-hidden"
          :class="{
            'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': row.right.lineType === 'addition',
            'bg-muted/10 opacity-40': row.right.lineType === 'empty',
            'text-highlighted border-s-2 border-transparent': row.right.lineType === 'context',
          }"
        >
          <!-- New Line Number -->
          <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/30 font-mono text-[10px]">
            {{ row.right.lineNumber || '' }}
          </div>

          <!-- New Gutter Prefix -->
          <div class="w-4 shrink-0 text-center select-none font-bold font-mono">
            <span v-if="row.right.lineType === 'addition'">+</span>
            <span v-else-if="row.right.lineType === 'context'">&nbsp;</span>
          </div>

          <!-- New Content -->
          <div class="flex-1 pe-2 whitespace-pre font-mono select-text truncate">
            {{ row.right.content }}
          </div>
        </div>
      </div>
    </div>

    <!-- Expansion Banner for Large Diffs -->
    <div
      v-if="isTruncated"
      class="p-4 bg-muted/20 border-t border-default/40 flex items-center justify-between gap-3 text-xs"
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
