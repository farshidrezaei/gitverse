<script setup lang="ts">
import type { FileDiff } from '~/types/repository'
import type { SelectedDiffTarget } from '~/composables/useWorkspaceManager'
import { buildSideBySideHunk } from '~/utils/diffLayout'

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

type DiffViewMode = 'inline' | 'side-by-side'
const viewMode = ref<DiffViewMode>('inline')

const sideBySideHunks = computed(() => {
  if (!props.diff || !props.diff.hunks) return []
  return props.diff.hunks.map(hunk => buildSideBySideHunk(hunk))
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
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden text-xs select-none" dir="ltr">
    <!-- Top Header Bar -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3">
      <!-- File Information & Status Badges -->
      <div v-if="selectedTarget || diff" class="flex items-center gap-2 min-w-0">
        <UIcon name="i-lucide-file-diff" class="h-4 w-4 shrink-0 text-primary" />
        <span class="font-mono font-semibold text-highlighted text-[11px] truncate" :title="selectedTarget ? selectedTarget.filePath : diff?.new_path">
          {{ selectedTarget ? selectedTarget.filePath : diff?.new_path }}
        </span>

        <UBadge
          v-if="selectedTarget?.isStaged"
          size="xs"
          color="success"
          variant="subtle"
          class="shrink-0"
        >
          {{ t('diff.stagedChanges') }}
        </UBadge>
        <UBadge
          v-else-if="selectedTarget"
          size="xs"
          color="warning"
          variant="subtle"
          class="shrink-0"
        >
          {{ t('diff.workingTreeChanges') }}
        </UBadge>
        <UBadge
          v-else
          size="xs"
          color="primary"
          variant="subtle"
          class="shrink-0"
        >
          {{ t('commit.details') }}
        </UBadge>

        <template v-if="diff && !diff.is_binary">
          <UBadge
            v-if="diff.total_additions > 0"
            size="xs"
            color="success"
            variant="soft"
            class="shrink-0 font-mono"
          >
            +{{ diff.total_additions }}
          </UBadge>
          <UBadge
            v-if="diff.total_deletions > 0"
            size="xs"
            color="error"
            variant="soft"
            class="shrink-0 font-mono"
          >
            -{{ diff.total_deletions }}
          </UBadge>
        </template>
      </div>

      <!-- Fallback empty header label -->
      <div v-else class="flex items-center gap-1.5 text-muted">
        <UIcon name="i-lucide-file-code" class="h-4 w-4" />
        <span class="font-medium text-[11px]">{{ t('diff.noFileSelected') }}</span>
      </div>

      <!-- Controls: View Mode Switcher & Quick Actions -->
      <div v-if="selectedTarget || diff" class="flex items-center gap-2">
        <!-- Quick Stage / Unstage Action (Working Tree Only) -->
        <template v-if="selectedTarget">
          <UButton
            v-if="selectedTarget.isStaged"
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-minus"
            :label="t('diff.unstageFile')"
            @click="$emit('unstage', selectedTarget.filePath)"
          />
          <UButton
            v-else
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-plus"
            :label="t('diff.stageFile')"
            @click="$emit('stage', selectedTarget.filePath)"
          />

          <div class="h-3.5 w-px bg-default/80" aria-hidden="true" />
        </template>

        <!-- Segmented Pill Switcher (Inline vs Side-by-Side) -->
        <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs">
          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="viewMode === 'inline'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :title="t('diff.viewInline')"
            @click="viewMode = 'inline'"
          >
            <UIcon name="i-lucide-rows-2" class="h-3 w-3" />
            <span>{{ t('diff.viewInline') }}</span>
          </button>

          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="viewMode === 'side-by-side'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :title="t('diff.viewSideBySide')"
            @click="viewMode = 'side-by-side'"
          >
            <UIcon name="i-lucide-columns-2" class="h-3 w-3" />
            <span>{{ t('diff.viewSideBySide') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Main Diff Body Container -->
    <div class="flex-1 overflow-auto bg-[var(--gv-surface)] font-mono text-[11px] leading-5">
      <!-- High-Fidelity Zero-CLS Skeleton Loading State -->
      <div v-if="isLoading" class="min-w-fit p-0">
        <!-- Skeleton Hunk Header -->
        <div class="flex items-center bg-[var(--gv-diff-hunk-bg)]/40 px-3 py-1 border-y border-default/40">
          <USkeleton class="h-3 w-48 rounded" />
        </div>
        <!-- Skeleton Diff Lines -->
        <div class="space-y-0.5 py-1">
          <div
            v-for="index in 8"
            :key="`diff-skeleton-${index}`"
            class="flex items-center h-5 px-1 animate-pulse"
            :class="{
              'bg-[var(--gv-diff-remove-bg)]/40': index === 2 || index === 3,
              'bg-[var(--gv-diff-add-bg)]/40': index === 4 || index === 5 || index === 6,
            }"
          >
            <!-- Old Line Number -->
            <div class="w-12 pe-2 text-end">
              <USkeleton class="h-2.5 w-5 ms-auto rounded" />
            </div>
            <!-- New Line Number -->
            <div class="w-12 pe-2 text-end">
              <USkeleton class="h-2.5 w-5 ms-auto rounded" />
            </div>
            <!-- Gutter Prefix -->
            <div class="w-5 text-center text-muted font-bold">
              <span v-if="index === 2 || index === 3">-</span>
              <span v-else-if="index >= 4 && index <= 6">+</span>
            </div>
            <!-- Line Content Skeleton -->
            <div class="flex-1 pe-4">
              <USkeleton
                class="h-2.5 rounded"
                :class="[
                  index === 1 ? 'w-2/5' :
                  index === 2 ? 'w-3/5' :
                  index === 3 ? 'w-1/2' :
                  index === 4 ? 'w-4/5' :
                  index === 5 ? 'w-2/3' :
                  index === 6 ? 'w-3/4' : 'w-1/3'
                ]"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- No Target Selected State -->
      <div v-else-if="!selectedTarget && !diff" class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted">
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-file-diff" class="h-8 w-8 text-muted/70" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.noFileSelected') }}</p>
          <p class="text-[11px] text-muted mt-0.5">{{ t('diff.selectFilePrompt') }}</p>
        </div>
      </div>

      <!-- Binary File State -->
      <div v-else-if="diff?.is_binary" class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted">
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-binary" class="h-8 w-8 text-warning" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.binaryFile') }}</p>
          <p class="text-[11px] text-muted mt-0.5 font-mono">{{ selectedTarget ? selectedTarget.filePath : diff.new_path }}</p>
        </div>
      </div>

      <!-- Empty / No Changes State -->
      <div v-else-if="!diff || diff.hunks.length === 0" class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted">
        <div class="rounded-full bg-muted/20 p-3">
          <UIcon name="i-lucide-file-check" class="h-8 w-8 text-success" />
        </div>
        <div class="text-center font-sans">
          <p class="font-medium text-highlighted text-xs">{{ t('diff.noChanges') }}</p>
          <p class="text-[11px] text-muted mt-0.5 font-mono">{{ selectedTarget ? selectedTarget.filePath : diff?.new_path }}</p>
        </div>
      </div>

      <!-- INLINE VIEW MODE -->
      <div v-else-if="viewMode === 'inline'" class="min-w-fit">
        <div v-for="(hunk, hunkIndex) in diff.hunks" :key="`inline-hunk-${hunkIndex}`" class="border-b border-default/50">
          <!-- Hunk Header -->
          <div class="flex items-center bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 py-1 font-semibold select-none border-y border-default/40">
            <span class="font-mono text-[11px]">{{ hunk.header }}</span>
          </div>

          <!-- Lines in Hunk -->
          <div
            v-for="(line, lineIndex) in hunk.lines"
            :key="`inline-line-${hunkIndex}-${lineIndex}`"
            class="flex items-stretch group"
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
      </div>

      <!-- SIDE-BY-SIDE VIEW MODE -->
      <div v-else-if="viewMode === 'side-by-side'" class="min-w-fit">
        <div v-for="(hunk, hunkIndex) in sideBySideHunks" :key="`sbs-hunk-${hunkIndex}`" class="border-b border-default/50">
          <!-- Hunk Header -->
          <div class="flex items-center bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 py-1 font-semibold select-none border-y border-default/40">
            <span class="font-mono text-[11px]">{{ hunk.header }}</span>
          </div>

          <!-- Side-by-Side Paired Rows -->
          <div
            v-for="row in hunk.rows"
            :key="row.id"
            class="grid grid-cols-2 divide-x divide-default/50 border-b border-default/20 last:border-b-0"
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
      </div>
    </div>
  </div>
</template>
