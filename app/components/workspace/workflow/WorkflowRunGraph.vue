<script setup lang="ts">
import type { WorkflowJob } from '~/types/provider'

interface Props {
  jobs: WorkflowJob[]
  selectedJobId?: string | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'selectJob', job: WorkflowJob): void
}>()

const { t } = useI18n()

// Organize jobs into sequential columns/stages
interface JobColumn {
  id: string
  title: string
  jobs: WorkflowJob[]
}

const columns = computed<JobColumn[]>(() => {
  if (!props.jobs || props.jobs.length === 0) return []

  // Check if jobs have explicit stages (GitLab or workflow names)
  const stageMap = new Map<string, WorkflowJob[]>()
  
  for (const job of props.jobs) {
    // Derive stage from job name (e.g., "build / lint" or stage property or matrix)
    let stageName = t('provider.actions.stageBuild')
    if (job.name.toLowerCase().includes('test') || job.name.toLowerCase().includes('e2e') || job.name.toLowerCase().includes('unit')) {
      stageName = t('provider.actions.stageTest')
    } else if (job.name.toLowerCase().includes('lint') || job.name.toLowerCase().includes('check') || job.name.toLowerCase().includes('analyze')) {
      stageName = t('provider.actions.stageLint')
    } else if (job.name.toLowerCase().includes('deploy') || job.name.toLowerCase().includes('release') || job.name.toLowerCase().includes('publish')) {
      stageName = t('provider.actions.stageDeploy')
    } else {
      stageName = t('provider.actions.stageBuild')
    }

    if (!stageMap.has(stageName)) {
      stageMap.set(stageName, [])
    }
    stageMap.get(stageName)!.push(job)
  }

  // Predefined column ordering
  const standardOrder = [
    t('provider.actions.stageLint'),
    t('provider.actions.stageBuild'),
    t('provider.actions.stageTest'),
    t('provider.actions.stageDeploy'),
  ]

  const result: JobColumn[] = []
  for (const orderKey of standardOrder) {
    if (stageMap.has(orderKey)) {
      result.push({
        id: orderKey,
        title: orderKey,
        jobs: stageMap.get(orderKey)!,
      })
      stageMap.delete(orderKey)
    }
  }

  // Any remaining custom stages
  for (const [key, jobsList] of stageMap.entries()) {
    result.push({
      id: key,
      title: key,
      jobs: jobsList,
    })
  }

  // Fallback if empty or only 1 single column with many jobs: chunk into pairs
  if (result.length === 1 && result[0] && result[0].jobs.length > 3) {
    const all = result[0].jobs
    return [
      { id: 'col-1', title: t('provider.actions.stageLint'), jobs: all.slice(0, Math.ceil(all.length / 2)) },
      { id: 'col-2', title: t('provider.actions.stageTest'), jobs: all.slice(Math.ceil(all.length / 2)) },
    ]
  }

  return result
})

function formatDuration(startedAtTimestamp: number | null, completedAtTimestamp: number | null): string {
  if (!startedAtTimestamp) return ''
  const end = completedAtTimestamp || Math.floor(Date.now() / 1000)
  const seconds = Math.max(0, end - startedAtTimestamp)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}m ${remainingSeconds}s`
}

function getJobStatusIcon(status: string, conclusion: string | null): string {
  if (status === 'in_progress') return 'i-lucide-loader-2 animate-spin text-amber-500'
  if (status === 'queued') return 'i-lucide-clock text-muted-foreground'
  if (conclusion === 'success') return 'i-lucide-check-circle-2 text-emerald-500'
  if (conclusion === 'failure') return 'i-lucide-x-circle text-rose-500'
  if (conclusion === 'cancelled' || conclusion === 'skipped') return 'i-lucide-circle-slash text-neutral-400'
  return 'i-lucide-help-circle text-muted-foreground'
}

function getJobStatusBadgeColor(status: string, conclusion: string | null): 'neutral' | 'success' | 'error' | 'warning' {
  if (status === 'in_progress') return 'warning'
  if (conclusion === 'success') return 'success'
  if (conclusion === 'failure') return 'error'
  return 'neutral'
}

function handleJobClick(job: WorkflowJob): void {
  emit('selectJob', job)
}
</script>

<template>
  <div class="relative w-full h-full min-h-[320px] p-6 overflow-x-auto overflow-y-auto select-none bg-[var(--gv-surface-subtle)]/40 rounded-xl border border-default/40">
    <!-- Graph Background Grid dots -->
    <div
      class="absolute inset-0 pointer-events-none opacity-20"
      style="background-image: radial-gradient(var(--gv-border) 1px, transparent 1px); background-size: 20px 20px;"
    />

    <!-- Workflow Columns Layout -->
    <div class="relative z-10 flex items-start gap-12 min-w-max pb-8">
      <div
        v-for="(column, colIndex) in columns"
        :key="column.id"
        class="flex flex-col items-center gap-4 w-72"
      >
        <!-- Stage Header -->
        <div class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg bg-[var(--gv-surface)]/80 border border-default/50 text-xs font-semibold text-muted-foreground">
          <div class="flex items-center gap-1.5 truncate">
            <span class="w-2 h-2 rounded-full bg-primary/70" />
            <span class="truncate">{{ column.title }}</span>
          </div>
          <UBadge size="xs" variant="subtle" color="neutral">
            {{ column.jobs.length }}
          </UBadge>
        </div>

        <!-- Stage Job Nodes -->
        <div class="w-full flex flex-col gap-3">
          <div
            v-for="job in column.jobs"
            :key="job.id"
            role="button"
            tabindex="0"
            :class="[
              'group relative flex flex-col p-3 rounded-xl border transition-all duration-200 cursor-pointer text-start',
              'hover:shadow-md hover:border-primary/50',
              selectedJobId === job.id
                ? 'bg-primary/5 border-primary shadow-sm ring-1 ring-primary/40'
                : 'bg-[var(--gv-surface)] border-default/70'
            ]"
            @click="handleJobClick(job)"
            @keydown.enter="handleJobClick(job)"
          >
            <!-- Header Row -->
            <div class="flex items-center justify-between gap-2 mb-2">
              <div class="flex items-center gap-2 min-w-0">
                <UIcon :name="getJobStatusIcon(job.status, job.conclusion)" class="w-4 h-4 shrink-0" />
                <span class="text-xs font-medium text-foreground truncate group-hover:text-primary transition-colors">
                  {{ job.name }}
                </span>
              </div>
              <UBadge
                size="xs"
                variant="subtle"
                :color="getJobStatusBadgeColor(job.status, job.conclusion)"
              >
                {{ job.conclusion || job.status }}
              </UBadge>
            </div>

            <!-- Steps Progress / Duration Row -->
            <div class="flex items-center justify-between text-[11px] text-muted-foreground pt-1.5 border-t border-default/30">
              <div class="flex items-center gap-1">
                <UIcon name="i-lucide-list-checks" class="w-3.5 h-3.5" />
                <span>
                  {{ job.steps.length > 0 ? `${job.steps.filter(s => s.conclusion === 'success').length}/${job.steps.length}` : t('provider.actions.jobSteps') }}
                </span>
              </div>
              <div v-if="job.started_at_timestamp" class="flex items-center gap-1">
                <UIcon name="i-lucide-clock" class="w-3.5 h-3.5" />
                <span>{{ formatDuration(job.started_at_timestamp, job.completed_at_timestamp) }}</span>
              </div>
            </div>

            <!-- Connector dot on right if not last column -->
            <div
              v-if="colIndex < columns.length - 1"
              class="absolute -end-1.5 top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-[var(--gv-surface)] border-2 border-primary/50"
            />
            <!-- Connector dot on left if not first column -->
            <div
              v-if="colIndex > 0"
              class="absolute -start-1.5 top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-[var(--gv-surface)] border-2 border-default/80"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
