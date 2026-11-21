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

interface JobColumn {
  id: string
  title: string
  icon: string
  color: string
  jobs: WorkflowJob[]
}

const columns = computed<JobColumn[]>(() => {
  if (!props.jobs || props.jobs.length === 0) return []

  const stageMap = new Map<string, WorkflowJob[]>()
  
  for (const job of props.jobs) {
    const lowerName = job.name.toLowerCase()
    let stageName = t('provider.actions.stageBuild')

    if (lowerName.includes('lint') || lowerName.includes('check') || lowerName.includes('analyze') || lowerName.includes('style')) {
      stageName = t('provider.actions.stageLint')
    } else if (lowerName.includes('test') || lowerName.includes('unit') || lowerName.includes('e2e') || lowerName.includes('integration')) {
      stageName = t('provider.actions.stageTest')
    } else if (lowerName.includes('deploy') || lowerName.includes('release') || lowerName.includes('publish') || lowerName.includes('docker') || lowerName.includes('push')) {
      stageName = t('provider.actions.stageDeploy')
    } else {
      stageName = t('provider.actions.stageBuild')
    }

    if (!stageMap.has(stageName)) {
      stageMap.set(stageName, [])
    }
    stageMap.get(stageName)!.push(job)
  }

  const standardStages: { name: string; icon: string; color: string }[] = [
    { name: t('provider.actions.stageLint'), icon: 'i-lucide-shield-check', color: 'text-sky-500' },
    { name: t('provider.actions.stageBuild'), icon: 'i-lucide-box', color: 'text-amber-500' },
    { name: t('provider.actions.stageTest'), icon: 'i-lucide-flask-conical', color: 'text-violet-500' },
    { name: t('provider.actions.stageDeploy'), icon: 'i-lucide-rocket', color: 'text-emerald-500' },
  ]

  const result: JobColumn[] = []
  for (const stage of standardStages) {
    if (stageMap.has(stage.name)) {
      result.push({
        id: stage.name,
        title: stage.name,
        icon: stage.icon,
        color: stage.color,
        jobs: stageMap.get(stage.name)!,
      })
      stageMap.delete(stage.name)
    }
  }

  for (const [key, jobsList] of stageMap.entries()) {
    result.push({
      id: key,
      title: key,
      icon: 'i-lucide-layers',
      color: 'text-primary',
      jobs: jobsList,
    })
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
  if (status === 'queued') return 'i-lucide-clock text-neutral-400'
  if (conclusion === 'success') return 'i-lucide-check-circle text-emerald-500'
  if (conclusion === 'failure') return 'i-lucide-x-circle text-rose-500'
  if (conclusion === 'cancelled' || conclusion === 'skipped') return 'i-lucide-circle-slash text-neutral-400'
  return 'i-lucide-help-circle text-muted-foreground'
}

function getJobBadgeVariant(status: string, conclusion: string | null): { color: 'success' | 'error' | 'warning' | 'neutral'; label: string } {
  if (status === 'in_progress') return { color: 'warning', label: t('provider.actions.statusRunning') }
  if (status === 'queued') return { color: 'neutral', label: t('provider.actions.statusQueued') }
  if (conclusion === 'success') return { color: 'success', label: t('provider.actions.statusSuccess') }
  if (conclusion === 'failure') return { color: 'error', label: t('provider.actions.statusFailed') }
  if (conclusion === 'cancelled') return { color: 'neutral', label: t('provider.actions.statusCancelled') }
  if (conclusion === 'skipped') return { color: 'neutral', label: t('provider.actions.statusSkipped') }
  return { color: 'neutral', label: status }
}
</script>

<template>
  <div class="relative w-full h-full min-h-[380px] p-8 overflow-auto select-none bg-[var(--gv-surface-subtle)]/30 rounded-2xl border border-default/40">
    <!-- Graph Background Grid dots -->
    <div
      class="absolute inset-0 pointer-events-none opacity-25"
      style="background-image: radial-gradient(var(--gv-border) 1px, transparent 1px); background-size: 24px 24px;"
    />

    <!-- Workflow Columns Layout -->
    <div class="relative z-10 flex items-start gap-12 min-w-max pb-8">
      <div
        v-for="(column, colIndex) in columns"
        :key="column.id"
        class="flex flex-col items-center gap-5 w-80"
      >
        <!-- Stage Modern Glass Header -->
        <div class="w-full flex items-center justify-between px-3.5 py-2 rounded-xl bg-[var(--gv-surface)] border border-default/60 shadow-xs text-xs font-medium">
          <div class="flex items-center gap-2 truncate">
            <UIcon :name="column.icon" :class="['w-4 h-4 shrink-0', column.color]" />
            <span class="font-semibold text-foreground truncate">{{ column.title }}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <span class="text-[10px] text-muted-foreground font-mono">
              {{ column.jobs.filter(j => j.conclusion === 'success').length }}/{{ column.jobs.length }}
            </span>
            <span
              :class="[
                'w-2 h-2 rounded-full',
                column.jobs.some(j => j.conclusion === 'failure')
                  ? 'bg-rose-500'
                  : column.jobs.some(j => j.status === 'in_progress')
                    ? 'bg-amber-500 animate-pulse'
                    : 'bg-emerald-500'
              ]"
            />
          </div>
        </div>

        <!-- Stage Job Nodes -->
        <div class="w-full flex flex-col gap-3.5">
          <div
            v-for="job in column.jobs"
            :key="job.id"
            role="button"
            tabindex="0"
            :class="[
              'group relative flex flex-col p-4 rounded-xl border transition-all duration-200 cursor-pointer text-start',
              'hover:shadow-lg hover:-translate-y-0.5',
              job.conclusion === 'failure' ? 'hover:border-rose-500/60' : 'hover:border-primary/60',
              selectedJobId === job.id
                ? 'bg-[var(--gv-surface)] border-primary shadow-md ring-2 ring-primary/20'
                : 'bg-[var(--gv-surface)] border-default/60 shadow-2xs'
            ]"
            @click="emit('selectJob', job)"
            @keydown.enter="emit('selectJob', job)"
          >
            <!-- Header Row -->
            <div class="flex items-center justify-between gap-3 mb-2.5">
              <div class="flex items-center gap-2.5 min-w-0">
                <UIcon :name="getJobStatusIcon(job.status, job.conclusion)" class="w-4 h-4 shrink-0" />
                <span class="text-xs font-semibold text-foreground truncate group-hover:text-primary transition-colors">
                  {{ job.name }}
                </span>
              </div>
              <UBadge
                size="xs"
                variant="subtle"
                :color="getJobBadgeVariant(job.status, job.conclusion).color"
              >
                {{ getJobBadgeVariant(job.status, job.conclusion).label }}
              </UBadge>
            </div>

            <!-- Steps Progress / Duration Row -->
            <div class="flex items-center justify-between text-[11px] text-muted-foreground pt-2.5 border-t border-default/30">
              <div class="flex items-center gap-1.5">
                <UIcon name="i-lucide-list-checks" class="w-3.5 h-3.5 text-muted-foreground/80" />
                <span>
                  {{ job.steps.length > 0 ? `${job.steps.filter(s => s.conclusion === 'success').length}/${job.steps.length} ${t('provider.actions.jobSteps')}` : t('provider.actions.jobSteps') }}
                </span>
              </div>
              <div v-if="job.started_at_timestamp" class="flex items-center gap-1">
                <UIcon name="i-lucide-clock" class="w-3.5 h-3.5 text-muted-foreground/80" />
                <span class="font-mono text-[10px]">{{ formatDuration(job.started_at_timestamp, job.completed_at_timestamp) }}</span>
              </div>
            </div>

            <!-- Connector dot on right if not last column -->
            <div
              v-if="colIndex < columns.length - 1"
              class="absolute -end-1.5 top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-[var(--gv-surface)] border-2 border-primary/60 shadow-xs"
            />
            <!-- Connector dot on left if not first column -->
            <div
              v-if="colIndex > 0"
              class="absolute -start-1.5 top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-[var(--gv-surface)] border-2 border-default/80 shadow-xs"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
