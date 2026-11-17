<script setup lang="ts">
import type { FileDiff } from '~/types/repository'
import type { SelectedDiffTarget } from '~/composables/useStagingController'

interface DiffHeaderToolbarProps {
  diff: FileDiff | null
  selectedTarget: SelectedDiffTarget | null
  viewMode: 'inline' | 'side-by-side'
}

const props = defineProps<DiffHeaderToolbarProps>()

const emit = defineEmits<{
  'update:viewMode': [mode: 'inline' | 'side-by-side']
  stage: [filePath: string]
  unstage: [filePath: string]
}>()

const { t } = useI18n()
</script>

<template>
  <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3">
    <!-- File Information & Status Badges -->
    <div v-if="props.selectedTarget || props.diff" class="flex items-center gap-2 min-w-0">
      <UIcon name="i-lucide-file-diff" class="h-4 w-4 shrink-0 text-primary" />
      <span
        class="font-mono font-semibold text-highlighted text-[11px] truncate"
        :title="props.selectedTarget ? props.selectedTarget.filePath : props.diff?.new_path || ''"
      >
        {{ props.selectedTarget ? props.selectedTarget.filePath : props.diff?.new_path }}
      </span>

      <UBadge
        v-if="props.selectedTarget?.isStaged"
        size="xs"
        color="success"
        variant="subtle"
        class="shrink-0"
      >
        {{ t('diff.stagedChanges') }}
      </UBadge>
      <UBadge
        v-else-if="props.selectedTarget"
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

      <template v-if="props.diff && !props.diff.is_binary">
        <UBadge
          v-if="props.diff.total_additions > 0"
          size="xs"
          color="success"
          variant="soft"
          class="shrink-0 font-mono"
        >
          +{{ props.diff.total_additions }}
        </UBadge>
        <UBadge
          v-if="props.diff.total_deletions > 0"
          size="xs"
          color="error"
          variant="soft"
          class="shrink-0 font-mono"
        >
          -{{ props.diff.total_deletions }}
        </UBadge>
      </template>
    </div>

    <!-- Fallback empty header label -->
    <div v-else class="flex items-center gap-1.5 text-muted">
      <UIcon name="i-lucide-file-code" class="h-4 w-4" />
      <span class="font-medium text-[11px]">{{ t('diff.noFileSelected') }}</span>
    </div>

    <!-- Controls: View Mode Switcher & Quick Actions -->
    <div v-if="props.selectedTarget || props.diff" class="flex items-center gap-2">
      <!-- Quick Stage / Unstage Action (Working Tree Only) -->
      <template v-if="props.selectedTarget">
        <UButton
          v-if="props.selectedTarget.isStaged"
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-minus"
          :label="t('diff.unstageFile')"
          @click="emit('unstage', props.selectedTarget.filePath)"
        />
        <UButton
          v-else
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-plus"
          :label="t('diff.stageFile')"
          @click="emit('stage', props.selectedTarget.filePath)"
        />

        <div class="h-3.5 w-px bg-default/80" aria-hidden="true" />
      </template>

      <!-- Segmented Pill Switcher (Inline vs Side-by-Side) -->
      <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs">
        <button
          type="button"
          class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
          :class="props.viewMode === 'inline'
            ? 'bg-primary text-inverted shadow-xs'
            : 'text-muted hover:text-highlighted'"
          :title="t('diff.viewInline')"
          @click="emit('update:viewMode', 'inline')"
        >
          <UIcon name="i-lucide-rows-2" class="h-3 w-3" />
          <span>{{ t('diff.viewInline') }}</span>
        </button>

        <button
          type="button"
          class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
          :class="props.viewMode === 'side-by-side'
            ? 'bg-primary text-inverted shadow-xs'
            : 'text-muted hover:text-highlighted'"
          :title="t('diff.viewSideBySide')"
          @click="emit('update:viewMode', 'side-by-side')"
        >
          <UIcon name="i-lucide-columns-2" class="h-3 w-3" />
          <span>{{ t('diff.viewSideBySide') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
