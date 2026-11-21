<script setup lang="ts">
import type { WorkspaceWorkflowRunTab } from '~/types/workspaceTab'
import type { WorkflowJob } from '~/types/provider'
import WorkflowRunGraph from '../workflow/WorkflowRunGraph.vue'
import WorkflowJobStepsDrawer from '../workflow/WorkflowJobStepsDrawer.vue'

interface Props {
  tab: WorkspaceWorkflowRunTab
}

const props = defineProps<Props>()
const { t } = useI18n()
const providerController = useProviderController()
const { notifySuccess, notifyError } = useNotificationService()

const jobs = ref<WorkflowJob[]>([])
const isLoadingJobs = ref<boolean>(false)
const selectedJob = ref<WorkflowJob | null>(null)
const isJobDrawerOpen = ref<boolean>(false)
const isActionRunning = ref<boolean>(false)

async function loadJobs(): Promise<void> {
  isLoadingJobs.value = true
  try {
    const fetchedJobs = await providerController.fetchWorkflowRunJobs(
      props.tab.account.id,
      props.tab.repositoryFullName,
      props.tab.workflowRun.id,
    )
    jobs.value = fetchedJobs
    if (fetchedJobs.length > 0 && !selectedJob.value) {
      selectedJob.value = fetchedJobs[0] || null
    }
  } finally {
    isLoadingJobs.value = false
  }
}

function handleSelectJob(job: WorkflowJob): void {
  selectedJob.value = job
  isJobDrawerOpen.value = true
}

async function handleRerunAll(): Promise<void> {
  isActionRunning.value = true
  try {
    const success = await providerController.rerunWorkflowRun(
      props.tab.account.id,
      props.tab.repositoryFullName,
      props.tab.workflowRun.id,
      false,
    )
    if (success) {
      notifySuccess(t('provider.actions.rerunTriggered'))
      await loadJobs()
    } else {
      notifyError(t('provider.actions.rerunFailed'))
    }
  } catch (error) {
    notifyError(String(error))
  } finally {
    isActionRunning.value = false
  }
}

async function handleRerunFailed(): Promise<void> {
  isActionRunning.value = true
  try {
    const success = await providerController.rerunWorkflowRun(
      props.tab.account.id,
      props.tab.repositoryFullName,
      props.tab.workflowRun.id,
      true,
    )
    if (success) {
      notifySuccess(t('provider.actions.rerunTriggered'))
      await loadJobs()
    } else {
      notifyError(t('provider.actions.rerunFailed'))
    }
  } catch (error) {
    notifyError(String(error))
  } finally {
    isActionRunning.value = false
  }
}

async function handleCancelRun(): Promise<void> {
  isActionRunning.value = true
  try {
    const success = await providerController.cancelWorkflowRun(
      props.tab.account.id,
      props.tab.repositoryFullName,
      props.tab.workflowRun.id,
    )
    if (success) {
      notifySuccess(t('provider.actions.cancelSuccess'))
      await loadJobs()
    } else {
      notifyError(t('provider.actions.cancelFailed'))
    }
  } catch (error) {
    notifyError(String(error))
  } finally {
    isActionRunning.value = false
  }
}

onMounted(() => {
  loadJobs()
})

const isCompleted = computed(() => {
  return props.tab.workflowRun.status === 'completed'
})

const isFailed = computed(() => {
  return props.tab.workflowRun.conclusion === 'failure'
})

const isRunning = computed(() => {
  return props.tab.workflowRun.status === 'in_progress' || props.tab.workflowRun.status === 'queued'
})
</script>

<template>
  <div class="h-full flex flex-col bg-[var(--gv-surface)] overflow-hidden">
    <!-- Top Action / Workflow Run Header -->
    <div class="px-6 py-4 border-b border-default/70 bg-[var(--gv-surface-subtle)]/30 flex items-center justify-between gap-4 shrink-0">
      <div class="flex items-center gap-3 min-w-0">
        <UIcon
          :name="isFailed ? 'i-lucide-x-circle' : isRunning ? 'i-lucide-loader-2 animate-spin' : 'i-lucide-check-circle-2'"
          :class="[
            'w-6 h-6 shrink-0',
            isFailed ? 'text-rose-500' : isRunning ? 'text-amber-500' : 'text-emerald-500'
          ]"
        />
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="text-base font-semibold text-foreground truncate">
              {{ tab.workflowRun.display_title }}
            </h2>
            <UBadge size="xs" variant="subtle" color="neutral">
              #{{ tab.workflowRun.run_number }}
            </UBadge>
          </div>
          <div class="flex items-center gap-2 text-xs text-muted-foreground mt-0.5">
            <UIcon name="i-lucide-git-branch" class="w-3.5 h-3.5" />
            <span class="font-mono">{{ tab.workflowRun.head_branch }}</span>
            <span>•</span>
            <span class="font-mono">{{ tab.workflowRun.head_commit_sha.slice(0, 7) }}</span>
            <span>•</span>
            <span>{{ tab.workflowRun.event }}</span>
          </div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-2 shrink-0">
        <UButton
          v-if="isRunning"
          icon="i-lucide-circle-slash"
          size="sm"
          variant="outline"
          color="error"
          :loading="isActionRunning"
          @click="handleCancelRun"
        >
          {{ t('provider.actions.cancelRun') }}
        </UButton>

        <UButton
          v-if="isCompleted && isFailed"
          icon="i-lucide-rotate-ccw"
          size="sm"
          variant="outline"
          color="neutral"
          :loading="isActionRunning"
          @click="handleRerunFailed"
        >
          {{ t('provider.actions.rerunFailedJobs') }}
        </UButton>

        <UButton
          v-if="isCompleted"
          icon="i-lucide-refresh-cw"
          size="sm"
          variant="solid"
          color="primary"
          :loading="isActionRunning"
          @click="handleRerunAll"
        >
          {{ t('provider.actions.rerunAllJobs') }}
        </UButton>

        <UButton
          v-if="tab.workflowRun.html_url"
          :to="tab.workflowRun.html_url"
          target="_blank"
          icon="i-lucide-external-link"
          size="sm"
          variant="ghost"
          color="neutral"
          :title="t('provider.actions.openInBrowser')"
        />
      </div>
    </div>

    <!-- Main Content Area: Graph Visualizer + Details Drawer -->
    <div class="flex-1 flex overflow-hidden relative">
      <!-- Loading Skeleton (Zero CLS Standard) -->
      <div v-if="isLoadingJobs" class="flex-1 p-6 space-y-4">
        <div class="flex gap-6">
          <div v-for="col in 3" :key="col" class="w-72 space-y-3">
            <USkeleton class="h-8 w-full rounded-lg" />
            <USkeleton v-for="row in 3" :key="row" class="h-20 w-full rounded-xl" />
          </div>
        </div>
      </div>

      <!-- Graph View -->
      <div v-else class="flex-1 h-full p-4 overflow-hidden">
        <WorkflowRunGraph
          :jobs="jobs"
          :selected-job-id="selectedJob?.id"
          @select-job="handleSelectJob"
        />
      </div>

      <!-- Drawer for Job Steps -->
      <WorkflowJobStepsDrawer
        :job="selectedJob"
        :is-open="isJobDrawerOpen"
        :account-id="tab.account.id"
        :repository-full-name="tab.repositoryFullName"
        @close="isJobDrawerOpen = false"
      />
    </div>
  </div>
</template>
