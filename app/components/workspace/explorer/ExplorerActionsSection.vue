<script setup lang="ts">
import type { WorkflowRun, ProviderAccount } from '~/types/provider'
import type { RemoteSummary } from '~/types/repository'
import { parseRepositoryFromUrl } from '~/utils/providerUrl'
import { formatRelativeTime } from '~/utils/dateFormat'

interface ExplorerActionsSectionProps {
  remotes?: RemoteSummary[]
  currentRepositoryPath?: string
}

const props = defineProps<ExplorerActionsSectionProps>()

const emit = defineEmits<{
  (e: 'openWorkflowRun', run: WorkflowRun, account: ProviderAccount, repositoryFullName: string): void
  (e: 'openAddRemoteModal'): void
}>()

const { t } = useI18n()
const providerController = useProviderController()
const tabController = useWorkspaceTabController()

const runs = ref<WorkflowRun[]>([])
const isLoading = ref<boolean>(false)
const matchedAccount = ref<ProviderAccount | null>(null)
const matchedRepositoryFullName = ref<string | null>(null)
const statusFilter = ref<string>('all')

const statusFilterOptions = computed(() => [
  { value: 'all', label: t('provider.actions.filterAll') },
  { value: 'completed', label: t('provider.actions.filterCompleted') },
  { value: 'in_progress', label: t('provider.actions.filterInProgress') },
  { value: 'queued', label: t('provider.actions.filterQueued') },
])

async function detectRepositoryAndLoad(): Promise<void> {
  await providerController.loadAccounts()
  if (providerController.accounts.value.length === 0) return

  let targetLocation: { host: string; fullName: string } | null = null

  if (props.remotes && props.remotes.length > 0) {
    for (const remote of props.remotes) {
      const url = remote.fetch_url || remote.push_url
      if (url) {
        const parsed = parseRepositoryFromUrl(url)
        if (parsed) {
          targetLocation = parsed
          break
        }
      }
    }
  }

  if (targetLocation) {
    matchedRepositoryFullName.value = targetLocation.fullName
    const isGitLab = targetLocation.host.includes('gitlab')
    const found = providerController.accounts.value.find(a => isGitLab ? a.provider === 'gitlab' : a.provider === 'github')
    matchedAccount.value = found || providerController.accounts.value[0] || null
  } else {
    matchedAccount.value = providerController.accounts.value[0] || null
  }

  if (matchedAccount.value && matchedRepositoryFullName.value) {
    await loadRuns()
  }
}

async function loadRuns(): Promise<void> {
  if (!matchedAccount.value || !matchedRepositoryFullName.value) return
  isLoading.value = true
  try {
    const filter = statusFilter.value === 'all' ? undefined : statusFilter.value
    const result = await providerController.fetchWorkflowRuns(
      matchedAccount.value.id,
      matchedRepositoryFullName.value,
      filter,
    )
    runs.value = result
  } finally {
    isLoading.value = false
  }
}

function handleRunClick(run: WorkflowRun): void {
  if (!matchedAccount.value || !matchedRepositoryFullName.value) return
  emit('openWorkflowRun', run, matchedAccount.value, matchedRepositoryFullName.value)
  tabController.openWorkflowRunTab(
    run,
    matchedAccount.value,
    matchedRepositoryFullName.value,
    props.currentRepositoryPath,
  )
}

function getRunStatusIcon(status: string, conclusion: string | null): string {
  if (status === 'in_progress') return 'i-lucide-loader-2 animate-spin text-amber-500'
  if (status === 'queued') return 'i-lucide-clock text-muted-foreground'
  if (conclusion === 'success') return 'i-lucide-check-circle-2 text-emerald-500'
  if (conclusion === 'failure') return 'i-lucide-x-circle text-rose-500'
  if (conclusion === 'cancelled' || conclusion === 'skipped') return 'i-lucide-circle-slash text-neutral-400'
  return 'i-lucide-help-circle text-muted-foreground'
}

watch(
  () => [props.remotes, props.currentRepositoryPath],
  () => {
    detectRepositoryAndLoad()
  },
  { immediate: true },
)

watch(statusFilter, () => {
  loadRuns()
})
</script>

<template>
  <div class="flex flex-col text-xs select-none">
    <!-- Filter Bar -->
    <div v-if="matchedAccount" class="flex items-center justify-between px-3 py-1.5 border-b border-default/40 bg-[var(--gv-surface-subtle)]/30">
      <div class="flex items-center gap-1.5 min-w-0">
        <UIcon name="i-lucide-play-circle" class="w-3.5 h-3.5 text-primary shrink-0" />
        <span class="font-mono text-[11px] truncate text-muted-foreground">
          {{ matchedRepositoryFullName }}
        </span>
      </div>
      <UButton
        icon="i-lucide-refresh-cw"
        size="xs"
        variant="ghost"
        color="neutral"
        :loading="isLoading"
        :aria-label="t('common.refresh')"
        @click="loadRuns"
      />
    </div>

    <!-- Status Filters Tabs -->
    <div v-if="matchedAccount" class="px-2 py-1.5 border-b border-default/40 flex items-center gap-1 overflow-x-auto">
      <button
        v-for="opt in statusFilterOptions"
        :key="opt.value"
        type="button"
        :class="[
          'px-2 py-0.5 rounded text-[11px] font-medium transition-colors shrink-0',
          statusFilter === opt.value
            ? 'bg-primary text-primary-foreground'
            : 'text-muted-foreground hover:bg-neutral-500/10 hover:text-foreground'
        ]"
        @click="statusFilter = opt.value"
      >
        {{ opt.label }}
      </button>
    </div>

    <!-- Loading Skeleton (Zero CLS Standard) -->
    <div v-if="isLoading && runs.length === 0" class="p-2 space-y-2">
      <div v-for="i in 4" :key="i" class="p-2.5 rounded-lg border border-default/30 space-y-2">
        <div class="flex items-center justify-between">
          <USkeleton class="h-3.5 w-32 rounded" />
          <USkeleton class="h-3 w-12 rounded" />
        </div>
        <USkeleton class="h-3 w-48 rounded" />
      </div>
    </div>

    <!-- Empty State: No Remotes Configured -->
    <div v-else-if="!isLoading && (!props.remotes || props.remotes.length === 0)" class="p-6 text-center text-muted-foreground space-y-2">
      <UIcon name="i-lucide-globe" class="w-8 h-8 mx-auto opacity-30 mb-2" />
      <div class="space-y-0.5">
        <p class="font-medium text-xs text-highlighted">{{ t('provider.noRemotesConfigured') }}</p>
        <p class="text-[11px] text-muted">{{ t('provider.noRemotesDescription') }}</p>
      </div>
      <UButton
        size="xs"
        color="primary"
        variant="subtle"
        icon="i-lucide-plus"
        :label="t('remote.addRemote')"
        @click="emit('openAddRemoteModal')"
      />
    </div>

    <!-- Empty State / No Runs -->
    <div v-else-if="runs.length === 0 && matchedAccount" class="p-6 text-center text-muted-foreground space-y-1">
      <UIcon name="i-lucide-play-circle" class="w-8 h-8 mx-auto opacity-30 mb-2" />
      <p class="font-medium text-xs">{{ t('provider.actions.noRunsFound') }}</p>
      <p class="text-[11px] text-muted-foreground/80">{{ t('provider.actions.noRunsFoundDescription') }}</p>
    </div>

    <!-- Not Configured / No Account Matched -->
    <div v-else-if="!matchedAccount" class="p-6 text-center text-muted-foreground space-y-2">
      <UIcon name="i-lucide-git-pull-request" class="w-8 h-8 mx-auto opacity-30 mb-2" />
      <p class="font-medium text-xs">{{ t('provider.actions.notConfigured') }}</p>
      <p class="text-[11px] text-muted-foreground/80">{{ t('provider.actions.notConfiguredDescription') }}</p>
    </div>

    <!-- Workflow Runs List -->
    <div v-else class="divide-y divide-default/30">
      <div
        v-for="run in runs"
        :key="run.id"
        role="button"
        tabindex="0"
        class="group p-2.5 hover:bg-neutral-500/10 cursor-pointer transition-colors flex flex-col gap-1 text-start"
        @click="handleRunClick(run)"
        @keydown.enter="handleRunClick(run)"
      >
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-1.5 min-w-0">
            <UIcon :name="getRunStatusIcon(run.status, run.conclusion)" class="w-3.5 h-3.5 shrink-0" />
            <span class="font-medium text-foreground truncate group-hover:text-primary transition-colors">
              {{ run.display_title }}
            </span>
          </div>
          <span class="text-[10px] text-muted-foreground shrink-0 font-mono">
            #{{ run.run_number }}
          </span>
        </div>

        <div class="flex items-center justify-between text-[11px] text-muted-foreground ps-5">
          <div class="flex items-center gap-1.5 truncate">
            <UIcon name="i-lucide-git-branch" class="w-3 h-3 shrink-0" />
            <span class="font-mono truncate">{{ run.head_branch }}</span>
          </div>
          <span class="shrink-0 text-[10px]">
            {{ formatRelativeTime(run.created_at_timestamp) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
