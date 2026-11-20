<script setup lang="ts">
import type { PullRequestMergeMethod, RemotePullRequest } from '~/types/provider'

const props = defineProps<{
  open: boolean
  pullRequest: RemotePullRequest
  accountIdentifier: string
  repositoryFullName: string
}>()

const emit = defineEmits<{
  (event: 'update:open', value: boolean): void
  (event: 'merged'): void
}>()

const { t } = useI18n()
const { mergePullRequest } = useProviderController()

const selectedMethod = ref<PullRequestMergeMethod>('merge')
const commitTitle = ref<string>('')
const commitMessage = ref<string>('')
const shouldDeleteSourceBranch = ref<boolean>(false)
const isSubmitting = ref<boolean>(false)
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen && props.pullRequest) {
      commitTitle.value = `Merge pull request #${props.pullRequest.number} from ${props.pullRequest.source_branch}`
      commitMessage.value = props.pullRequest.title
      selectedMethod.value = 'merge'
      shouldDeleteSourceBranch.value = false
      errorMessage.value = null
    }
  },
  { immediate: true },
)

function closeModal(): void {
  emit('update:open', false)
  errorMessage.value = null
}

async function handleConfirmMerge(): Promise<void> {
  isSubmitting.value = true
  errorMessage.value = null

  try {
    const result = await mergePullRequest(
      props.accountIdentifier,
      props.repositoryFullName,
      props.pullRequest.number,
      selectedMethod.value,
      commitTitle.value.trim() || undefined,
      commitMessage.value.trim() || undefined,
      shouldDeleteSourceBranch.value,
    )

    if (result && result.is_merged) {
      emit('merged')
      closeModal()
    } else {
      errorMessage.value = result?.message || 'Failed to merge pull request. Check permissions.'
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
    :title="`${t('provider.mergePullRequest')} #${props.pullRequest.number}`"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="p-6 space-y-4">
        <!-- Error Alert -->
        <div v-if="errorMessage" class="p-3 rounded-lg bg-error/15 border border-error/30 text-error text-xs">
          {{ errorMessage }}
        </div>

        <!-- Strategy Options -->
        <div class="space-y-2">
          <label
            class="flex items-start gap-3 p-3 rounded-xl border cursor-pointer transition-all"
            :class="selectedMethod === 'merge' ? 'border-primary bg-primary/10 shadow-xs' : 'border-default/60 hover:bg-muted/10'"
          >
            <input
              v-model="selectedMethod"
              type="radio"
              value="merge"
              class="mt-1 text-primary focus:ring-primary"
            >
            <div class="space-y-0.5">
              <span class="text-xs font-bold text-highlighted flex items-center gap-1.5">
                <UIcon name="i-lucide-git-merge" class="h-3.5 w-3.5 text-primary" />
                <span>{{ t('provider.mergeCommit') }}</span>
              </span>
              <p class="text-[11px] text-muted">{{ t('provider.mergeCommitDescription') }}</p>
            </div>
          </label>

          <label
            class="flex items-start gap-3 p-3 rounded-xl border cursor-pointer transition-all"
            :class="selectedMethod === 'squash' ? 'border-primary bg-primary/10 shadow-xs' : 'border-default/60 hover:bg-muted/10'"
          >
            <input
              v-model="selectedMethod"
              type="radio"
              value="squash"
              class="mt-1 text-primary focus:ring-primary"
            >
            <div class="space-y-0.5">
              <span class="text-xs font-bold text-highlighted flex items-center gap-1.5">
                <UIcon name="i-lucide-git-commit" class="h-3.5 w-3.5 text-primary" />
                <span>{{ t('provider.squashAndMerge') }}</span>
              </span>
              <p class="text-[11px] text-muted">{{ t('provider.squashAndMergeDescription') }}</p>
            </div>
          </label>

          <label
            class="flex items-start gap-3 p-3 rounded-xl border cursor-pointer transition-all"
            :class="selectedMethod === 'rebase' ? 'border-primary bg-primary/10 shadow-xs' : 'border-default/60 hover:bg-muted/10'"
          >
            <input
              v-model="selectedMethod"
              type="radio"
              value="rebase"
              class="mt-1 text-primary focus:ring-primary"
            >
            <div class="space-y-0.5">
              <span class="text-xs font-bold text-highlighted flex items-center gap-1.5">
                <UIcon name="i-lucide-git-pull-request-arrow" class="h-3.5 w-3.5 text-primary" />
                <span>{{ t('provider.rebaseAndMerge') }}</span>
              </span>
              <p class="text-[11px] text-muted">{{ t('provider.rebaseAndMergeDescription') }}</p>
            </div>
          </label>
        </div>

        <!-- Commit details -->
        <div class="space-y-3 pt-1">
          <div class="space-y-1">
            <label class="text-xs font-medium text-highlighted">
              {{ t('provider.commitTitle') }}
            </label>
            <UInput
              v-model="commitTitle"
              size="xs"
              class="w-full"
            />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-medium text-highlighted">
              {{ t('provider.commitMessage') }}
            </label>
            <textarea
              v-model="commitMessage"
              rows="3"
              class="w-full rounded-lg border border-default bg-[var(--gv-surface)] p-2 text-xs text-highlighted focus:border-primary focus:outline-hidden font-mono"
            />
          </div>

          <!-- Delete source branch checkbox -->
          <label class="flex items-center gap-2 text-xs text-muted cursor-pointer pt-1">
            <input
              v-model="shouldDeleteSourceBranch"
              type="checkbox"
              class="rounded border-default text-primary focus:ring-primary"
            >
            <span>{{ t('provider.deleteSourceBranch') }} (<code class="text-[11px] text-primary">{{ props.pullRequest.source_branch }}</code>)</span>
          </label>
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-end gap-2 pt-3 border-t border-default/50">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="closeModal"
          />
          <UButton
            size="xs"
            color="primary"
            variant="solid"
            :label="t('provider.confirmMerge')"
            icon="i-lucide-git-merge"
            :loading="isSubmitting"
            @click="handleConfirmMerge"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
