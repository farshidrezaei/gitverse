<script setup lang="ts">
import type { PullRequestReviewAction, RemotePullRequest } from '~/types/provider'

const props = defineProps<{
  open: boolean
  pullRequest: RemotePullRequest
  accountIdentifier: string
  repositoryFullName: string
}>()

const emit = defineEmits<{
  (event: 'update:open', value: boolean): void
  (event: 'reviewSubmitted'): void
}>()

const { t } = useI18n()
const { submitPullRequestReview } = useProviderController()

const selectedAction = ref<PullRequestReviewAction>('APPROVE')
const reviewCommentBody = ref<string>('')
const isSubmitting = ref<boolean>(false)
const errorMessage = ref<string | null>(null)

function closeModal(): void {
  emit('update:open', false)
  reviewCommentBody.value = ''
  errorMessage.value = null
  selectedAction.value = 'APPROVE'
}

async function handleSubmitReview(): Promise<void> {
  isSubmitting.value = true
  errorMessage.value = null

  try {
    const success = await submitPullRequestReview(
      props.accountIdentifier,
      props.repositoryFullName,
      props.pullRequest.number,
      selectedAction.value,
      reviewCommentBody.value.trim() || undefined,
    )

    if (success) {
      emit('reviewSubmitted')
      closeModal()
    } else {
      errorMessage.value = 'Failed to submit review. Please try again.'
    }
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <UModal
    :open="props.open"
    :title="`${t('provider.submitReview')} #${props.pullRequest.number}`"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="p-6 space-y-4">
        <!-- Error Alert -->
        <div v-if="errorMessage" class="p-3 rounded-lg bg-error/15 border border-error/30 text-error text-xs">
          {{ errorMessage }}
        </div>

        <!-- Action Option Cards -->
        <div class="space-y-2">
          <!-- Comment -->
          <label
            class="flex items-start gap-3 p-3 rounded-xl border cursor-pointer transition-all"
            :class="selectedAction === 'COMMENT' ? 'border-primary bg-primary/10 shadow-xs' : 'border-default/60 hover:bg-muted/10'"
          >
            <input
              v-model="selectedAction"
              type="radio"
              value="COMMENT"
              class="mt-1 text-primary focus:ring-primary"
            >
            <div class="space-y-0.5">
              <span class="text-xs font-bold text-highlighted flex items-center gap-1.5">
                <UIcon name="i-lucide-message-square" class="h-3.5 w-3.5 text-muted" />
                <span>{{ t('provider.reviewComment') }}</span>
              </span>
              <p class="text-[11px] text-muted">{{ t('provider.reviewCommentDescription') }}</p>
            </div>
          </label>

          <!-- Approve -->
          <label
            class="flex items-start gap-3 p-3 rounded-xl border cursor-pointer transition-all"
            :class="selectedAction === 'APPROVE' ? 'border-success bg-success/10 shadow-xs' : 'border-default/60 hover:bg-muted/10'"
          >
            <input
              v-model="selectedAction"
              type="radio"
              value="APPROVE"
              class="mt-1 text-success focus:ring-success"
            >
            <div class="space-y-0.5">
              <span class="text-xs font-bold text-success flex items-center gap-1.5">
                <UIcon name="i-lucide-check-circle" class="h-3.5 w-3.5 text-success" />
                <span>{{ t('provider.reviewApprove') }}</span>
              </span>
              <p class="text-[11px] text-muted">{{ t('provider.reviewApproveDescription') }}</p>
            </div>
          </label>

          <!-- Request Changes -->
          <label
            class="flex items-start gap-3 p-3 rounded-xl border cursor-pointer transition-all"
            :class="selectedAction === 'REQUEST_CHANGES' ? 'border-error bg-error/10 shadow-xs' : 'border-default/60 hover:bg-muted/10'"
          >
            <input
              v-model="selectedAction"
              type="radio"
              value="REQUEST_CHANGES"
              class="mt-1 text-error focus:ring-error"
            >
            <div class="space-y-0.5">
              <span class="text-xs font-bold text-error flex items-center gap-1.5">
                <UIcon name="i-lucide-alert-triangle" class="h-3.5 w-3.5 text-error" />
                <span>{{ t('provider.reviewRequestChanges') }}</span>
              </span>
              <p class="text-[11px] text-muted">{{ t('provider.reviewRequestChangesDescription') }}</p>
            </div>
          </label>
        </div>

        <!-- Optional Review Comment Text -->
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-highlighted">
            {{ t('provider.reviewSummaryOptional') }}
          </label>
          <textarea
            v-model="reviewCommentBody"
            rows="3"
            class="w-full rounded-lg border border-default bg-[var(--gv-surface)] p-2 text-xs text-highlighted focus:border-primary focus:outline-hidden"
            :placeholder="t('provider.reviewSummaryPlaceholder')"
          />
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-end gap-2 pt-2 border-t border-default/50">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="closeModal"
          />
          <UButton
            size="xs"
            :color="selectedAction === 'APPROVE' ? 'success' : selectedAction === 'REQUEST_CHANGES' ? 'error' : 'primary'"
            variant="solid"
            :label="t('provider.submitReview')"
            :loading="isSubmitting"
            @click="handleSubmitReview"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
