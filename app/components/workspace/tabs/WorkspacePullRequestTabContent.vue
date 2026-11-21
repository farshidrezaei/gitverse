<script setup lang="ts">
import type { WorkspacePullRequestTab } from '~/types/workspaceTab'
import type { RemotePullRequest, CiCheckRun } from '~/types/provider'
import PullRequestCommentsTimeline from '~/components/workspace/provider/PullRequestCommentsTimeline.vue'
import PullRequestFilesChangedView from '~/components/workspace/provider/PullRequestFilesChangedView.vue'
import PullRequestReviewModal from '~/components/workspace/provider/PullRequestReviewModal.vue'
import PullRequestMergeModal from '~/components/workspace/provider/PullRequestMergeModal.vue'

interface WorkspacePullRequestTabContentProperties {
  tab: WorkspacePullRequestTab
}

const properties = defineProps<WorkspacePullRequestTabContentProperties>()

const emit = defineEmits<{
  close: [tabIdentifier: string]
  checkoutBranch: [branchName: string]
}>()

const { t } = useI18n()
const { updatePullRequestState, fetchCheckRuns } = useProviderController()

const activeDetailTab = computed<'conversation' | 'files' | 'checks'>({
  get: () => properties.tab.activeSubView || 'conversation',
  set: (view) => {
    properties.tab.activeSubView = view
  },
})

const isReviewModalOpen = ref(false)
const isMergeModalOpen = ref(false)
const isUpdatingState = ref(false)
const checkRuns = ref<CiCheckRun[]>([])
const isLoadingChecks = ref(false)

const pullRequest = computed<RemotePullRequest>(() => properties.tab.pullRequest)

function getCheckRunIcon(conclusion: string | null, status: string): string {
  if (conclusion === 'success') return 'i-lucide-check-circle-2'
  if (conclusion === 'failure' || conclusion === 'timed_out' || conclusion === 'action_required') return 'i-lucide-x-circle'
  if (conclusion === 'cancelled') return 'i-lucide-circle-slash'
  if (conclusion === 'skipped' || conclusion === 'neutral') return 'i-lucide-minus-circle'
  if (status === 'in_progress') return 'i-lucide-loader'
  if (status === 'queued' || status === 'waiting' || status === 'pending') return 'i-lucide-clock'
  return 'i-lucide-help-circle'
}

function getCheckRunIconClass(conclusion: string | null, status: string): string {
  if (conclusion === 'success') return 'text-success'
  if (conclusion === 'failure' || conclusion === 'timed_out' || conclusion === 'action_required') return 'text-error'
  if (conclusion === 'cancelled' || conclusion === 'skipped' || conclusion === 'neutral') return 'text-muted'
  if (status === 'in_progress') return 'text-warning animate-spin'
  if (status === 'queued' || status === 'waiting' || status === 'pending') return 'text-muted'
  return 'text-muted'
}

function getCheckRunBadgeColor(conclusion: string | null, status: string): 'success' | 'error' | 'warning' | 'neutral' {
  if (conclusion === 'success') return 'success'
  if (conclusion === 'failure' || conclusion === 'timed_out' || conclusion === 'action_required') return 'error'
  if (conclusion === 'cancelled' || conclusion === 'skipped' || conclusion === 'neutral') return 'neutral'
  if (status === 'in_progress') return 'warning'
  return 'neutral'
}

async function loadChecks(): Promise<void> {
  isLoadingChecks.value = true
  try {
    checkRuns.value = await fetchCheckRuns(
      properties.tab.account.id,
      properties.tab.repositoryFullName,
      pullRequest.value.source_branch,
    )
  } finally {
    isLoadingChecks.value = false
  }
}

watch(
  () => activeDetailTab.value,
  (tab) => {
    if (tab === 'checks' && checkRuns.value.length === 0) {
      void loadChecks()
    }
  },
  { immediate: true },
)

async function handleToggleState(action: 'close' | 'reopen'): Promise<void> {
  isUpdatingState.value = true
  try {
    const targetState: 'open' | 'closed' = action === 'close' ? 'closed' : 'open'
    const isSuccess = await updatePullRequestState(
      properties.tab.account.id,
      properties.tab.repositoryFullName,
      pullRequest.value.number,
      targetState,
    )
    if (isSuccess) {
      pullRequest.value.state = targetState
    }
  } finally {
    isUpdatingState.value = false
  }
}

function handleReviewSubmitted(): void {
  // Trigger refresh of comments if on conversation tab
}

function handleMerged(): void {
  pullRequest.value.state = 'merged'
  pullRequest.value.merged_at_timestamp = Date.now()
}
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden bg-[var(--gv-background)]">
    <!-- Top Action & Meta Header -->
    <header class="flex items-center justify-between border-b border-default/70 bg-[var(--gv-surface)]/60 px-5 py-3 shrink-0">
      <div class="flex items-center gap-3 min-w-0">
        <UBadge
          size="md"
          :color="pullRequest.state === 'open' ? 'success' : pullRequest.state === 'merged' ? 'primary' : 'neutral'"
          variant="subtle"
          class="font-semibold capitalize px-2 py-0.5 text-xs shrink-0"
        >
          {{ pullRequest.state === 'merged' ? t('provider.merged') : pullRequest.state }}
        </UBadge>

        <h1 class="text-base font-bold text-highlighted truncate">
          {{ pullRequest.title }}
          <span class="text-muted font-normal ms-1">#{{ pullRequest.number }}</span>
        </h1>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-2 shrink-0">
        <UButton
          size="xs"
          color="primary"
          variant="soft"
          icon="i-lucide-message-square-plus"
          :label="t('provider.submitReview')"
          @click="isReviewModalOpen = true"
        />

        <UButton
          v-if="pullRequest.state === 'open'"
          size="xs"
          color="primary"
          variant="solid"
          icon="i-lucide-git-merge"
          :label="t('provider.mergePullRequest')"
          @click="isMergeModalOpen = true"
        />

        <UButton
          v-if="pullRequest.state === 'open'"
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-x-circle"
          :loading="isUpdatingState"
          :label="t('provider.closePullRequest')"
          @click="handleToggleState('close')"
        />

        <UButton
          v-else-if="pullRequest.state !== 'merged'"
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-rotate-ccw"
          :loading="isUpdatingState"
          :label="t('provider.reopenPullRequest')"
          @click="handleToggleState('reopen')"
        />

        <div class="h-4 w-px bg-default/40 mx-1" />

        <UTooltip :text="t('tabs.close')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            @click="emit('close', properties.tab.id)"
          />
        </UTooltip>
      </div>
    </header>

    <!-- Branch info & Navigation Subheader -->
    <div class="flex items-center justify-between border-b border-default/50 bg-[var(--gv-surface)]/20 px-5 py-2 shrink-0 text-xs">
      <div class="flex items-center gap-2 text-toned min-w-0">
        <span class="text-muted">{{ t('provider.author') }}:</span>
        <div class="flex items-center gap-1.5 font-medium">
          <UAvatar
            v-if="pullRequest.author_avatar_url"
            :src="pullRequest.author_avatar_url"
            :alt="pullRequest.author_username"
            size="2xs"
          />
          <span class="text-highlighted">{{ pullRequest.author_username }}</span>
        </div>

        <span class="text-muted mx-1">•</span>

        <span class="font-mono text-[11px] bg-muted/20 px-1.5 py-0.5 rounded text-highlighted">
          {{ pullRequest.source_branch }}
        </span>
        <UIcon name="i-lucide-arrow-right" class="h-3 w-3 text-muted shrink-0 rtl:rotate-180" />
        <span class="font-mono text-[11px] bg-muted/20 px-1.5 py-0.5 rounded text-highlighted">
          {{ pullRequest.target_branch }}
        </span>

        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-git-branch"
          class="ms-2 text-xs"
          :label="t('provider.checkoutBranch')"
          @click="emit('checkoutBranch', pullRequest.source_branch)"
        />
      </div>

      <!-- Segmented Navigation for PR views -->
      <div class="flex items-center gap-1 bg-muted/20 p-0.5 rounded-lg border border-default/40">
        <button
          type="button"
          class="flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-all"
          :class="activeDetailTab === 'conversation' ? 'bg-[var(--gv-surface)] text-highlighted shadow-xs font-semibold' : 'text-muted hover:text-default'"
          @click="activeDetailTab = 'conversation'"
        >
          <UIcon name="i-lucide-messages-square" class="h-3.5 w-3.5" />
          <span>{{ t('provider.conversation') }}</span>
        </button>

        <button
          type="button"
          class="flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-all"
          :class="activeDetailTab === 'files' ? 'bg-[var(--gv-surface)] text-highlighted shadow-xs font-semibold' : 'text-muted hover:text-default'"
          @click="activeDetailTab = 'files'"
        >
          <UIcon name="i-lucide-file-diff" class="h-3.5 w-3.5" />
          <span>{{ t('provider.filesChanged') }}</span>
        </button>

        <button
          type="button"
          class="flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-all"
          :class="activeDetailTab === 'checks' ? 'bg-[var(--gv-surface)] text-highlighted shadow-xs font-semibold' : 'text-muted hover:text-default'"
          @click="activeDetailTab = 'checks'"
        >
          <UIcon name="i-lucide-shield-check" class="h-3.5 w-3.5" />
          <span>{{ t('provider.ciChecks') }}</span>
        </button>
      </div>
    </div>

    <!-- Main Dynamic Tab Content Viewport -->
    <div class="flex-1 overflow-hidden relative">
      <!-- 1. Conversation Tab -->
      <div v-if="activeDetailTab === 'conversation'" class="h-full overflow-y-auto p-6 max-w-5xl mx-auto">
        <PullRequestCommentsTimeline
          :account-identifier="properties.tab.account.id"
          :repository-full-name="properties.tab.repositoryFullName"
          :pull-request-number="pullRequest.number"
          :pull-request="pullRequest"
        />
      </div>

      <!-- 2. Files Changed Tab -->
      <div v-else-if="activeDetailTab === 'files'" class="h-full p-4">
        <PullRequestFilesChangedView
          :account-identifier="properties.tab.account.id"
          :repository-full-name="properties.tab.repositoryFullName"
          :pull-request-number="pullRequest.number"
        />
      </div>

      <!-- 3. Checks Tab -->
      <div v-else-if="activeDetailTab === 'checks'" class="h-full overflow-y-auto p-6 max-w-4xl mx-auto">
        <div v-if="isLoadingChecks" class="space-y-3">
          <div
            v-for="skeletonIndex in 6"
            :key="`check-skeleton-${skeletonIndex}`"
            class="flex items-center justify-between p-3.5 rounded-xl border border-default/60 bg-[var(--gv-surface)] animate-pulse"
          >
            <div class="flex items-center gap-3">
              <USkeleton class="h-5 w-5 rounded-full" />
              <div class="space-y-1">
                <USkeleton class="h-4 w-36 rounded" />
                <USkeleton class="h-3 w-48 rounded" />
              </div>
            </div>
            <USkeleton class="h-5 w-20 rounded-full" />
          </div>
        </div>

        <div v-else-if="checkRuns.length === 0" class="flex flex-col items-center justify-center py-20 text-muted space-y-2">
          <UIcon name="i-lucide-shield-alert" class="h-10 w-10 opacity-40" />
          <p class="text-sm font-medium">{{ t('provider.noChecksFound') }}</p>
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="run in checkRuns"
            :key="run.id"
            class="flex items-center justify-between p-3.5 rounded-xl border border-default/60 bg-[var(--gv-surface)]"
          >
            <div class="flex items-center gap-3">
              <UIcon
                :name="getCheckRunIcon(run.conclusion, run.status)"
                class="h-5 w-5 shrink-0"
                :class="getCheckRunIconClass(run.conclusion, run.status)"
              />
              <div>
                <h4 class="text-sm font-semibold text-highlighted">{{ run.name }}</h4>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <UBadge
                size="xs"
                :color="getCheckRunBadgeColor(run.conclusion, run.status)"
                variant="subtle"
                class="capitalize text-[11px]"
              >
                {{ run.conclusion || run.status }}
              </UBadge>
              <UButton
                v-if="run.html_url"
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-external-link"
                :to="run.html_url"
                target="_blank"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Review Modal -->
    <PullRequestReviewModal
      v-if="pullRequest"
      v-model:open="isReviewModalOpen"
      :account-identifier="properties.tab.account.id"
      :repository-full-name="properties.tab.repositoryFullName"
      :pull-request="pullRequest"
      @review-submitted="handleReviewSubmitted"
    />

    <!-- Merge Modal -->
    <PullRequestMergeModal
      v-if="pullRequest"
      v-model:open="isMergeModalOpen"
      :account-identifier="properties.tab.account.id"
      :repository-full-name="properties.tab.repositoryFullName"
      :pull-request="pullRequest"
      @merged="handleMerged"
    />
  </div>
</template>
