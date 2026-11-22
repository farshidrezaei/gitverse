<script setup lang="ts">
import type { ConflictLineComparison } from '~/utils/conflict'

interface ConflictSideBySideViewProps {
  oursLineComparisons: ConflictLineComparison[]
  theirsLineComparisons: ConflictLineComparison[]
  baseLines: string[]
  isBaseVisible: boolean
}

const props = defineProps<ConflictSideBySideViewProps>()

const emit = defineEmits<{
  acceptOurs: []
  acceptTheirs: []
}>()

const { t } = useI18n()

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
</script>

<template>
  <div
    class="grid gap-3 h-full min-h-[400px]"
    :class="props.isBaseVisible ? 'grid-cols-3' : 'grid-cols-2'"
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
          @click="emit('acceptOurs')"
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
            v-for="(line, lineIndex) in props.oursLineComparisons"
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
      v-if="props.isBaseVisible"
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
            v-for="(line, lineIndex) in props.baseLines"
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
          @click="emit('acceptTheirs')"
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
            v-for="(line, lineIndex) in props.theirsLineComparisons"
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
</template>
