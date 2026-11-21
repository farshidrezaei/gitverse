<script setup lang="ts">
import type { WorkflowJob } from '~/types/provider'

interface Props {
  job: WorkflowJob | null
  isOpen: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'close'): void
}>()

const { t } = useI18n()

function formatDuration(startedAtTimestamp: number | null, completedAtTimestamp: number | null): string {
  if (!startedAtTimestamp) return ''
  const end = completedAtTimestamp || Math.floor(Date.now() / 1000)
  const seconds = Math.max(0, end - startedAtTimestamp)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}m ${remainingSeconds}s`
}

function getStepIcon(status: string, conclusion: string | null): string {
  if (status === 'in_progress') return 'i-lucide-loader-2 animate-spin text-amber-500'
  if (conclusion === 'success') return 'i-lucide-check text-emerald-500'
  if (conclusion === 'failure') return 'i-lucide-x text-rose-500'
  if (conclusion === 'skipped' || conclusion === 'cancelled') return 'i-lucide-circle-slash text-neutral-400'
  return 'i-lucide-clock text-muted-foreground'
}
</script>

<template>
  <div
    v-if="isOpen && job"
    class="w-80 h-full flex flex-col bg-[var(--gv-surface)] border-s border-default/70 shrink-0 select-none animate-in slide-in-from-end duration-200"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-default/70 bg-[var(--gv-surface-subtle)]/50">
      <div class="flex items-center gap-2 min-w-0">
        <UIcon name="i-lucide-terminal" class="w-4 h-4 text-primary shrink-0" />
        <h3 class="text-sm font-semibold text-foreground truncate">
          {{ job.name }}
        </h3>
      </div>
      <UButton
        icon="i-lucide-x"
        size="xs"
        variant="ghost"
        color="neutral"
        :aria-label="t('common.close')"
        @click="emit('close')"
      />
    </div>

    <!-- Metadata Bar -->
    <div class="px-4 py-2.5 border-b border-default/40 bg-[var(--gv-surface)] text-xs flex items-center justify-between text-muted-foreground">
      <div class="flex items-center gap-1.5">
        <UBadge
          size="xs"
          variant="subtle"
          :color="job.conclusion === 'success' ? 'success' : job.conclusion === 'failure' ? 'error' : 'neutral'"
        >
          {{ job.conclusion || job.status }}
        </UBadge>
        <span v-if="job.started_at_timestamp">
          {{ formatDuration(job.started_at_timestamp, job.completed_at_timestamp) }}
        </span>
      </div>
      <UButton
        v-if="job.html_url"
        :to="job.html_url"
        target="_blank"
        icon="i-lucide-external-link"
        size="xs"
        variant="ghost"
        color="neutral"
        :title="t('provider.actions.openInBrowser')"
      />
    </div>

    <!-- Steps List -->
    <div class="flex-1 overflow-y-auto p-3 space-y-1.5">
      <div v-if="job.steps.length === 0" class="text-xs text-muted-foreground p-4 text-center">
        {{ t('provider.actions.noStepsAvailable') }}
      </div>
      <div
        v-for="step in job.steps"
        :key="step.number"
        class="flex items-center justify-between p-2 rounded-lg bg-[var(--gv-surface-subtle)]/40 border border-default/30 text-xs"
      >
        <div class="flex items-center gap-2 min-w-0">
          <UIcon :name="getStepIcon(step.status, step.conclusion)" class="w-3.5 h-3.5 shrink-0" />
          <span class="truncate font-mono text-[11px] text-foreground">
            {{ step.name }}
          </span>
        </div>
        <span v-if="step.started_at_timestamp" class="text-[10px] text-muted-foreground shrink-0 ms-2">
          {{ formatDuration(step.started_at_timestamp, step.completed_at_timestamp) }}
        </span>
      </div>
    </div>
  </div>
</template>
