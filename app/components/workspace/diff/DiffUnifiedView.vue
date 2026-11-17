<script setup lang="ts">
import type { FileDiff } from '~/types/repository'

interface DiffUnifiedViewProps {
  diff: FileDiff
}

const props = defineProps<DiffUnifiedViewProps>()

const INITIAL_VISIBLE_LINES_LIMIT = 500
const isFullyExpanded = ref<boolean>(false)

const totalLinesCount = computed(() => {
  return props.diff.hunks.reduce((acc, hunk) => acc + hunk.lines.length, 0)
})

const isTruncated = computed(() => {
  return !isFullyExpanded.value && totalLinesCount.value > INITIAL_VISIBLE_LINES_LIMIT
})

function formatLineContent(content: string, lineType: string): string {
  if (lineType === 'context' && content.startsWith(' ')) {
    return content.slice(1)
  }
  if (lineType === 'addition' && content.startsWith('+')) {
    return content.slice(1)
  }
  if (lineType === 'deletion' && content.startsWith('-')) {
    return content.slice(1)
  }
  return content
}

// Compute truncated hunks if total lines exceed limit
const displayedHunks = computed(() => {
  if (!isTruncated.value) {
    return props.diff.hunks
  }

  let remaining = INITIAL_VISIBLE_LINES_LIMIT
  const truncated = []

  for (const hunk of props.diff.hunks) {
    if (remaining <= 0) break
    if (hunk.lines.length <= remaining) {
      truncated.push(hunk)
      remaining -= hunk.lines.length
    } else {
      truncated.push({
        ...hunk,
        lines: hunk.lines.slice(0, remaining),
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
      :key="`inline-hunk-${hunkIndex}`"
      class="border-b border-default/50"
    >
      <!-- Hunk Header -->
      <div class="flex items-center bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 py-1 font-semibold select-none border-y border-default/40">
        <span class="font-mono text-[11px]">{{ hunk.header }}</span>
      </div>

      <!-- Lines in Hunk with Content-Visibility Optimization -->
      <div
        v-for="(line, lineIndex) in hunk.lines"
        :key="`inline-line-${hunkIndex}-${lineIndex}`"
        class="flex items-stretch group"
        style="content-visibility: auto; contain-intrinsic-size: 0 20px;"
        :class="{
          'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': line.line_type === 'addition',
          'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': line.line_type === 'deletion',
          'text-highlighted hover:bg-muted/15 border-s-2 border-transparent': line.line_type === 'context',
        }"
      >
        <!-- Old Line Number -->
        <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/30 font-mono text-[10px]">
          {{ line.old_line_number || '' }}
        </div>

        <!-- New Line Number -->
        <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/30 font-mono text-[10px]">
          {{ line.new_line_number || '' }}
        </div>

        <!-- Gutter Prefix Sign (+ / - / space) -->
        <div class="w-5 shrink-0 text-center select-none font-bold font-mono">
          <span v-if="line.line_type === 'addition'">+</span>
          <span v-else-if="line.line_type === 'deletion'">-</span>
          <span v-else>&nbsp;</span>
        </div>

        <!-- Line Content -->
        <div class="flex-1 pe-4 whitespace-pre font-mono select-text overflow-x-visible">
          {{ formatLineContent(line.content, line.line_type) }}
        </div>
      </div>
    </div>

    <!-- Expansion Banner for Large Diffs -->
    <div
      v-if="isTruncated"
      class="p-4 bg-muted/20 border-t border-default/40 flex items-center justify-between gap-3 text-xs"
    >
      <span class="text-muted">
        Showing first {{ INITIAL_VISIBLE_LINES_LIMIT }} of {{ totalLinesCount }} lines for performance
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
