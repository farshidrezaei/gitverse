<script setup lang="ts">
import type { WorkflowJob } from '~/types/provider'

interface Props {
  job: WorkflowJob | null
  isOpen: boolean
  accountId?: string
  repositoryFullName?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'close'): void
}>()

const { t } = useI18n()
const providerController = useProviderController()

const activeTab = ref<'steps' | 'logs'>('steps')
const logsText = ref<string>('')
const isLoadingLogs = ref<boolean>(false)
const searchQuery = ref<string>('')

async function loadJobLogs(): Promise<void> {
  if (!props.job || !props.accountId || !props.repositoryFullName) return
  isLoadingLogs.value = true
  try {
    const raw = await providerController.fetchWorkflowJobLogs(
      props.accountId,
      props.repositoryFullName,
      props.job.id,
    )
    logsText.value = raw
  } finally {
    isLoadingLogs.value = false
  }
}

watch(
  () => [props.job?.id, activeTab.value],
  ([newJobId, tab]) => {
    if (newJobId && tab === 'logs') {
      loadJobLogs()
    }
  },
  { immediate: true },
)

const filteredLogLines = computed(() => {
  if (!logsText.value) return []
  const lines = logsText.value.split('\n')
  if (!searchQuery.value.trim()) return lines
  const q = searchQuery.value.toLowerCase()
  return lines.filter(l => l.toLowerCase().includes(q))
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

function getStepIcon(status: string, conclusion: string | null): string {
  if (status === 'in_progress') return 'i-lucide-loader-2 animate-spin text-amber-500'
  if (conclusion === 'success') return 'i-lucide-check-circle-2 text-emerald-500'
  if (conclusion === 'failure') return 'i-lucide-x-circle text-rose-500'
  if (conclusion === 'skipped' || conclusion === 'cancelled') return 'i-lucide-circle-slash text-neutral-400'
  return 'i-lucide-clock text-muted-foreground'
}

function isErrorLine(line: string): boolean {
  const l = line.toLowerCase()
  return l.includes('error') || l.includes('fatal') || l.includes('failed') || l.includes('exception')
}
</script>

<template>
  <div
    v-if="isOpen && job"
    class="w-96 h-full flex flex-col bg-[var(--gv-surface)] border-s border-default/70 shrink-0 select-none animate-in slide-in-from-end duration-200 shadow-xl"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-default/70 bg-[var(--gv-surface-subtle)]/40">
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
    <div class="px-4 py-2 border-b border-default/40 bg-[var(--gv-surface)] text-xs flex items-center justify-between text-muted-foreground">
      <div class="flex items-center gap-2">
        <UBadge
          size="xs"
          variant="subtle"
          :color="job.conclusion === 'success' ? 'success' : job.conclusion === 'failure' ? 'error' : 'neutral'"
        >
          {{ job.conclusion || job.status }}
        </UBadge>
        <span v-if="job.started_at_timestamp" class="font-mono text-[11px]">
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

    <!-- View Mode Tabs (Steps vs Live Terminal Log) -->
    <div class="flex border-b border-default/40 bg-[var(--gv-surface-subtle)]/20 px-3 pt-2 gap-2">
      <button
        type="button"
        :class="[
          'px-3 py-1.5 text-xs font-semibold border-b-2 transition-all cursor-pointer flex items-center gap-1.5',
          activeTab === 'steps'
            ? 'border-primary text-primary'
            : 'border-transparent text-muted-foreground hover:text-foreground'
        ]"
        @click="activeTab = 'steps'"
      >
        <UIcon name="i-lucide-list-tree" class="w-3.5 h-3.5" />
        <span>{{ t('provider.actions.jobSteps') }} ({{ job.steps.length }})</span>
      </button>

      <button
        type="button"
        :class="[
          'px-3 py-1.5 text-xs font-semibold border-b-2 transition-all cursor-pointer flex items-center gap-1.5',
          activeTab === 'logs'
            ? 'border-primary text-primary'
            : 'border-transparent text-muted-foreground hover:text-foreground'
        ]"
        @click="activeTab = 'logs'"
      >
        <UIcon name="i-lucide-terminal-square" class="w-3.5 h-3.5" />
        <span>{{ t('provider.actions.viewLogs') }}</span>
      </button>
    </div>

    <!-- TAB 1: Steps View -->
    <div v-if="activeTab === 'steps'" class="flex-1 overflow-y-auto p-3 space-y-2">
      <div v-if="job.steps.length === 0" class="text-xs text-muted-foreground p-6 text-center space-y-1">
        <UIcon name="i-lucide-file-code" class="w-6 h-6 mx-auto opacity-40 mb-1" />
        <p>{{ t('provider.actions.noStepsAvailable') }}</p>
      </div>

      <div
        v-for="step in job.steps"
        :key="step.number"
        :class="[
          'flex items-center justify-between p-2.5 rounded-xl border text-xs transition-colors',
          step.conclusion === 'failure'
            ? 'bg-rose-500/10 border-rose-500/40'
            : 'bg-[var(--gv-surface-subtle)]/40 border-default/30 hover:border-default/60'
        ]"
      >
        <div class="flex items-center gap-2.5 min-w-0">
          <UIcon :name="getStepIcon(step.status, step.conclusion)" class="w-4 h-4 shrink-0" />
          <span
            :class="[
              'truncate font-mono text-[11px]',
              step.conclusion === 'failure' ? 'text-rose-500 font-semibold' : 'text-foreground'
            ]"
          >
            {{ step.name }}
          </span>
        </div>
        <span v-if="step.started_at_timestamp" class="text-[10px] text-muted-foreground font-mono shrink-0 ms-2">
          {{ formatDuration(step.started_at_timestamp, step.completed_at_timestamp) }}
        </span>
      </div>
    </div>

    <!-- TAB 2: Live Terminal Logs & Failure Inspector -->
    <div v-else class="flex-1 flex flex-col overflow-hidden bg-black/90 text-neutral-200 font-mono text-[11px]">
      <!-- Search & Refresh Toolbar -->
      <div class="flex items-center justify-between px-3 py-1.5 border-b border-neutral-800 bg-neutral-900/60 shrink-0">
        <div class="flex items-center gap-1.5 flex-1 min-w-0 me-2">
          <UIcon name="i-lucide-search" class="w-3.5 h-3.5 text-neutral-400 shrink-0" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('provider.actions.searchLogs')"
            class="bg-transparent border-none text-xs text-neutral-200 placeholder:text-neutral-500 focus:outline-none w-full"
          />
        </div>
        <UButton
          icon="i-lucide-refresh-cw"
          size="xs"
          variant="ghost"
          color="neutral"
          :loading="isLoadingLogs"
          :title="t('common.refresh')"
          @click="loadJobLogs"
        />
      </div>

      <!-- Logs Content Area -->
      <div v-if="isLoadingLogs" class="flex-1 p-4 space-y-2">
        <USkeleton v-for="i in 8" :key="i" class="h-4 w-full bg-neutral-800 rounded" />
      </div>

      <div
        v-else-if="filteredLogLines.length === 0"
        class="flex-1 flex items-center justify-center p-6 text-center text-neutral-500"
      >
        <span>{{ t('provider.actions.noLogsFound') }}</span>
      </div>

      <div
        v-else
        class="flex-1 overflow-y-auto p-3 space-y-0.5 select-text leading-relaxed"
      >
        <div
          v-for="(line, index) in filteredLogLines"
          :key="index"
          :class="[
            'px-1 py-0.5 rounded break-all whitespace-pre-wrap',
            isErrorLine(line)
              ? 'bg-rose-950/60 text-rose-300 font-semibold border-s-2 border-rose-500'
              : 'hover:bg-neutral-800/40 text-neutral-300'
          ]"
        >
          {{ line }}
        </div>
      </div>
    </div>
  </div>
</template>
