<script setup lang="ts">
import type { DiffLine, FileDiff } from '~/types/repository'
import { computeIntraLineDiff, type DiffTextSpan } from '~/utils/diffLayout'

interface DiffUnifiedViewProps {
  diff: FileDiff
  isWordWrap?: boolean
  isStaged?: boolean | null
}

const props = withDefaults(defineProps<DiffUnifiedViewProps>(), {
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

function getHunkLineSpans(lines: DiffLine[]): Map<number, DiffTextSpan[]> {
  const spansMap = new Map<number, DiffTextSpan[]>()
  for (let index = 0; index < lines.length; index += 1) {
    const current = lines[index]
    const next = lines[index + 1]
    if (
      current?.line_type === 'deletion' &&
      next?.line_type === 'addition' &&
      !spansMap.has(index)
    ) {
      const cleanOld = formatLineContent(current.content, current.line_type)
      const cleanNew = formatLineContent(next.content, next.line_type)
      const { oldSpans, newSpans } = computeIntraLineDiff(cleanOld, cleanNew)
      spansMap.set(index, oldSpans)
      spansMap.set(index + 1, newSpans)
    }
  }
  return spansMap
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
      <div class="flex items-center justify-between bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 py-1 font-semibold select-none border-y border-default/40">
        <span class="font-mono text-[11px]">{{ hunk.header }}</span>

        <!-- Hunk Action Controls -->
        <div v-if="props.isStaged !== null" class="flex items-center gap-1.5 opacity-80 hover:opacity-100 transition-opacity">
          <!-- Unstaged view: Stage Hunk & Discard Hunk -->
          <template v-if="!props.isStaged">
            <UButton
              size="xs"
              color="primary"
              variant="subtle"
              icon="i-lucide-plus"
              :label="t('diff.stageHunk')"
              @click.stop="emit('stageHunk', hunkIndex)"
            />
            <UButton
              size="xs"
              color="error"
              variant="ghost"
              icon="i-lucide-undo-2"
              :label="t('diff.discardHunk')"
              @click.stop="emit('discardHunk', hunkIndex)"
            />
          </template>

          <!-- Staged view: Unstage Hunk -->
          <template v-else>
            <UButton
              size="xs"
              color="neutral"
              variant="subtle"
              icon="i-lucide-minus"
              :label="t('diff.unstageHunk')"
              @click.stop="emit('unstageHunk', hunkIndex)"
            />
          </template>
        </div>
      </div>

      <!-- Lines in Hunk -->
      <div
        v-for="(line, lineIndex) in hunk.lines"
        :key="`inline-line-${hunkIndex}-${lineIndex}`"
        class="flex items-stretch group leading-[22px]"
        :class="[
          props.isWordWrap ? 'min-h-[22px]' : 'h-[22px] min-h-[22px] max-h-[22px]',
          {
            'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] border-s-2 border-[var(--gv-diff-add-border)]': line.line_type === 'addition',
            'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] border-s-2 border-[var(--gv-diff-remove-border)]': line.line_type === 'deletion',
            'text-highlighted hover:bg-muted/15 border-s-2 border-transparent': line.line_type === 'context',
          },
        ]"
      >
        <!-- Old Line Number -->
        <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/30 font-mono text-[10px]">
          {{ line.old_line_number || '' }}
        </div>

        <!-- New Line Number -->
        <div class="w-12 shrink-0 pe-2 text-end select-none text-muted/60 border-e border-default/30 font-mono text-[10px]">
          {{ line.new_line_number || '' }}
        </div>

        <!-- Gutter Prefix Sign (+ / - / space) & Hover Action -->
        <div class="w-6 shrink-0 flex items-center justify-center select-none font-bold font-mono">
          <template v-if="props.isStaged !== null && (line.line_type === 'addition' || line.line_type === 'deletion')">
            <span class="group-hover:hidden">
              <span v-if="line.line_type === 'addition'">+</span>
              <span v-else-if="line.line_type === 'deletion'">-</span>
            </span>

            <!-- Micro button on hover -->
            <button
              v-if="!props.isStaged"
              type="button"
              class="hidden group-hover:flex items-center justify-center h-4 w-4 rounded-xs bg-primary/20 text-primary hover:bg-primary hover:text-white transition-colors cursor-pointer"
              :title="t('diff.stageLine')"
              @click.stop="emit('stageLine', hunkIndex, lineIndex)"
            >
              <UIcon name="i-lucide-plus" class="h-3 w-3" />
            </button>
            <button
              v-else
              type="button"
              class="hidden group-hover:flex items-center justify-center h-4 w-4 rounded-xs bg-error/20 text-error hover:bg-error hover:text-white transition-colors cursor-pointer"
              :title="t('diff.unstageLine')"
              @click.stop="emit('unstageLine', hunkIndex, lineIndex)"
            >
              <UIcon name="i-lucide-minus" class="h-3 w-3" />
            </button>
          </template>
          <template v-else>
            <span v-if="line.line_type === 'addition'">+</span>
            <span v-else-if="line.line_type === 'deletion'">-</span>
            <span v-else>&nbsp;</span>
          </template>
        </div>

        <!-- Line Content -->
        <div
          class="flex-1 pe-4 font-mono select-text"
          :class="props.isWordWrap ? 'whitespace-pre-wrap break-all' : 'whitespace-pre overflow-x-visible'"
        >
          <template v-if="getHunkLineSpans(hunk.lines).get(lineIndex)">
            <span
              v-for="(span, spanIndex) in getHunkLineSpans(hunk.lines).get(lineIndex)"
              :key="`span-${lineIndex}-${spanIndex}`"
              :class="{
                'bg-[var(--gv-diff-remove-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.line_type === 'deletion',
                'bg-[var(--gv-diff-add-word-bg)] font-semibold rounded-xs px-0.5': span.isChanged && line.line_type === 'addition',
              }"
            >{{ span.text }}</span>
          </template>
          <template v-else>
            {{ formatLineContent(line.content, line.line_type) }}
          </template>
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
