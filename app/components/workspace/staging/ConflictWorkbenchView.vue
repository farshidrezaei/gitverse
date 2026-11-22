<script setup lang="ts">
import type { ConflictFileDetails } from '~/types/repository'
import type { ParsedConflictChunk } from '~/utils/conflict'

interface ConflictWorkbenchViewProps {
  currentChunk: ParsedConflictChunk | null
  conflictDetails: ConflictFileDetails | null
  isBaseVisible: boolean
}

const props = defineProps<ConflictWorkbenchViewProps>()

const emit = defineEmits<{
  resolveChunk: [strategy: 'ours' | 'theirs' | 'both_ours_first' | 'both_theirs_first']
}>()

const editableModel = defineModel<string>('editableContent', { required: true })

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-col gap-4 flex-1">
    <!-- Chunk Action Bar -->
    <div v-if="props.currentChunk" class="flex flex-wrap items-center justify-between p-3 rounded-xl border border-primary/30 bg-primary/5">
      <div class="flex items-center gap-2">
        <UIcon name="i-lucide-alert-circle" class="h-4 w-4 text-primary shrink-0" />
        <span class="text-xs font-semibold text-highlighted">
          {{ t('conflict.resolveChunk') }} (L{{ props.currentChunk.startLine }} - L{{ props.currentChunk.endLine }}):
        </span>
      </div>
      <div class="flex flex-wrap items-center gap-1.5">
        <UButton
          size="xs"
          color="primary"
          variant="solid"
          icon="i-lucide-arrow-left"
          :label="t('conflict.acceptOurs')"
          @click="emit('resolveChunk', 'ours')"
        />
        <UButton
          size="xs"
          color="warning"
          variant="solid"
          icon="i-lucide-arrow-right"
          :label="t('conflict.acceptTheirs')"
          @click="emit('resolveChunk', 'theirs')"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-combine"
          :label="t('conflict.acceptBothOursFirst')"
          @click="emit('resolveChunk', 'both_ours_first')"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-combine"
          :label="t('conflict.acceptBothTheirsFirst')"
          @click="emit('resolveChunk', 'both_theirs_first')"
        />
      </div>
    </div>

    <!-- 3-Way Top Comparison (Ours | Base | Theirs) -->
    <div
      class="grid gap-3 flex-1 min-h-[220px]"
      :class="props.isBaseVisible ? 'grid-cols-3' : 'grid-cols-2'"
    >
      <!-- Ours Pane -->
      <div class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60">
        <div class="flex items-center justify-between border-b border-default/70 px-3 py-2 bg-primary/10">
          <span class="text-xs font-semibold text-primary flex items-center gap-1.5">
            <UIcon name="i-lucide-arrow-left" class="h-3.5 w-3.5 rtl:rotate-180" />
            {{ t('conflict.paneOurs') }}
          </span>
          <span v-if="props.currentChunk" class="font-mono text-[10px] text-muted">
            {{ props.currentChunk.oursBranchName }}
          </span>
        </div>
        <div class="flex-1 p-3 font-mono text-xs overflow-auto whitespace-pre bg-surface/30 select-text" dir="ltr">
          <template v-if="props.currentChunk">
            {{ props.currentChunk.oursContent || '(empty)' }}
          </template>
          <template v-else>
            <div class="text-muted text-center pt-8">
              {{ t('conflict.noConflictsFound') }}
            </div>
          </template>
        </div>
      </div>

      <!-- Base Pane -->
      <div v-if="props.isBaseVisible" class="flex flex-col border border-default/70 rounded-xl overflow-hidden bg-surface/60">
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
          <span v-if="props.currentChunk" class="font-mono text-[10px] text-muted">
            {{ props.currentChunk.theirsBranchName }}
          </span>
        </div>
        <div class="flex-1 p-3 font-mono text-xs overflow-auto whitespace-pre bg-surface/30 select-text" dir="ltr">
          <template v-if="props.currentChunk">
            {{ props.currentChunk.theirsContent || '(empty)' }}
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
        v-model="editableModel"
        rows="8"
        wrap="off"
        class="w-full flex-1 p-3 font-mono text-xs bg-transparent text-highlighted focus:outline-none select-text leading-relaxed resize-none overflow-x-auto whitespace-pre"
        dir="ltr"
      />
    </div>
  </div>
</template>
