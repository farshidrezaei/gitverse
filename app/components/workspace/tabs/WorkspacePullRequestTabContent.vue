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

const activeDetailTab = ref<'conversation' | 'files' | 'checks'>('conversation')
const isReviewModalOpen = ref(false)
const isMergeModalOpen = ref(false)
const isUpdatingState = ref(false)
const checkRuns = ref<CiCheckRun[]>([])
const isLoadingChecks = ref(false)

const pullRequest = computed<RemotePullRequest>(() => properties.tab.pullRequest)

async function loadChecks(): Promise<void> {
  if (properties.tab.account.provider_kind !== 'github') return
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
)

async function handleToggleState(action: 'close' | 'reopen'): Promise<void> {
  isUpdatingState.value = true
  try {
    const updated = await updatePullRequestState(
      properties.tab.account.id,
      properties.tab.repositoryFullName,
      pullRequest.value.number,
      action,
    )
    pullRequest.value.state = updated.state
    pullRequest.value.closed_at = updated.closed_at
  } finally {
    isUpdatingState.value = false
  }
}

function handleReviewSubmitted(): void {
  // Trigger refresh of comments if on conversation tab
}

function handleMerged(mergedPr: RemotePullRequest): void {
  pullRequest.value.state = mergedPr.state
  pullRequest.value.merged_at = mergedPr.merged_at
}
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden bg-[var(--gv-background)]">
    <!-- Top Action & Meta Header -->
    <header class="flex items-center justify-between border-b border-default/70 bg-[var(--gv-surface)]/60 px-5 py-3 shrink-0">
      <div class="flex items-center gap-3 min-w-0">
        <UBadge
          size="md"
          :color="pullRequest.state === 'open' ? 'success' : pullRequest.merged_at ? 'primary' : 'neutral'"
          variant="subtle"
          class="font-semibold capitalize px-2 py-0.5 text-xs shrink-0"
        >
          {{ pullRequest.merged_at ? t('provider.merged') : pullRequest.state }}
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
          v-else-if="!pullRequest.merged_at"
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
                :name="run.conclusion === 'success' ? 'i-lucide-check-circle-2' :
                       run.conclusion === 'failure' ? 'i-lucide-x-circle' : 'i-lucide-loader'"
                class="h-5 w-5 shrink-0"
                :class="run.conclusion === 'success' ? 'text-success' :
                        run.conclusion === 'failure' ? 'text-error' : 'text-warning animate-spin'"
              />
              <div>
                <h4 class="text-sm font-semibold text-highlighted">{{ run.name }}</h4>
                <p v-if="run.summary" class="text-xs text-muted">{{ run.summary }}</p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <UBadge
                size="xs"
                :color="run.conclusion === 'success' ? 'success' : run.conclusion === 'failure' ? 'error' : 'warning'"
                variant="subtle"
                class="capitalize text-[11px]"
              >
                {{ run.conclusion || run.status }}
              </UBadge>
              <UButton
                v-if="run.details_url"
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-external-link"
                :to="run.details_url"
                target="_blank"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Review Modal -->
    <PullRequestReviewModal
      v-model:open="isReviewModalOpen"
      :account-identifier="properties.tab.account.id"
      :repository-full-name="properties.tab.repositoryFullName"
      :pull-request-number="pullRequest.number"
      @submitted="handleReviewSubmitted"
    />

    <!-- Merge Modal -->
    <PullRequestMergeModal
      v-model:open="isMergeModalOpen"
      :account-identifier="properties.tab.account.id"
      :repository-full-name="properties.tab.repositoryFullName"
      :pull-request="pullRequest"
      @merged="handleMerged"
    />
  </div>
</template>
