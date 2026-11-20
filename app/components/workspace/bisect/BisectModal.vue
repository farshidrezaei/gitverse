<script setup lang="ts">
interface BisectModalProps {
  open: boolean
}

const props = defineProps<BisectModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  locateCommit: [commitHash: string]
}>()

const { t } = useI18n()
const {
  bisectState,
  isLoadingBisect,
  fetchBisectStatus,
  startBisect,
  stepBisect,
  resetBisect,
} = useBisectController()

const badCommitInput = ref<string>('HEAD')
const goodCommitInput = ref<string>('')
const showLog = ref<boolean>(false)

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      await fetchBisectStatus()
    }
  },
  { immediate: true }
)

const canStartBisect = computed(() => {
  return badCommitInput.value.trim().length > 0 && goodCommitInput.value.trim().length > 0
})

async function handleStartBisect() {
  if (!canStartBisect.value) return
  await startBisect(badCommitInput.value.trim(), goodCommitInput.value.trim())
}

async function handleStep(decision: 'good' | 'bad' | 'skip') {
  await stepBisect(decision)
}

async function handleReset() {
  await resetBisect()
}

function handleCopyHash(commitHash: string) {
  navigator.clipboard.writeText(commitHash)
}

function handleLocate(commitHash: string) {
  emit('locateCommit', commitHash)
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl max-w-2xl overflow-hidden focus:outline-none',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Header -->
      <div class="px-6 py-4 border-b border-default/60 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
            <UIcon name="i-lucide-git-commit" class="w-5 h-5 text-primary" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('bisect.modalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('bisect.modalSubtitle') }}
            </p>
          </div>
        </div>
        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          @click="emit('update:open', false)"
        />
      </div>

      <!-- Zero-CLS Skeleton Loading -->
      <div v-if="isLoadingBisect && !bisectState" class="p-6 space-y-4">
        <USkeleton class="h-10 w-full rounded-lg" />
        <USkeleton class="h-24 w-full rounded-xl" />
        <div class="flex gap-3 justify-end">
          <USkeleton class="h-9 w-24 rounded-lg" />
          <USkeleton class="h-9 w-28 rounded-lg" />
        </div>
      </div>

      <!-- Active Bisect View -->
      <div v-else-if="bisectState && bisectState.is_active" class="p-6 space-y-6">
        <!-- Culprit Found Banner -->
        <div
          v-if="bisectState.culprit_commit_hash"
          class="p-4 rounded-xl border border-error/40 bg-error/10 space-y-3"
        >
          <div class="flex items-center gap-2 text-error font-medium text-sm">
            <UIcon name="i-lucide-alert-triangle" class="w-5 h-5" />
            <span>{{ t('bisect.culpritFound') }}</span>
          </div>

          <div class="p-3 bg-surface rounded-lg border border-default/50 space-y-1">
            <div class="flex items-center justify-between">
              <span class="font-mono text-xs font-semibold text-highlighted">
                {{ bisectState.culprit_commit_hash }}
              </span>
              <div class="flex items-center gap-1">
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-copy"
                  @click="handleCopyHash(bisectState.culprit_commit_hash!)"
                />
                <UButton
                  size="xs"
                  color="primary"
                  variant="soft"
                  icon="i-lucide-locate"
                  :label="t('bisect.viewCommit')"
                  @click="handleLocate(bisectState.culprit_commit_hash!)"
                />
              </div>
            </div>
            <p v-if="bisectState.culprit_commit_summary" class="text-xs text-muted">
              {{ bisectState.culprit_commit_summary }}
            </p>
          </div>

          <div class="flex justify-end">
            <UButton
              color="neutral"
              variant="outline"
              icon="i-lucide-rotate-ccw"
              :label="t('bisect.finishAndReset')"
              :loading="isLoadingBisect"
              @click="handleReset"
            />
          </div>
        </div>

        <!-- Bisect In-Progress Testing Step -->
        <div v-else class="space-y-4">
          <!-- Step indicator & stats -->
          <div class="flex items-center justify-between p-3.5 bg-neutral-500/5 rounded-xl border border-default/50">
            <div class="flex items-center gap-2">
              <UIcon name="i-lucide-compass" class="w-4 h-4 text-primary" />
              <span class="text-xs font-medium text-highlighted">
                {{ t('bisect.testingCommit') }}
              </span>
            </div>
            <div
              v-if="bisectState.revisions_remaining !== null && bisectState.revisions_remaining !== undefined"
              class="text-xs text-muted"
            >
              {{ t('bisect.stepsRemaining', {
                revisions: bisectState.revisions_remaining,
                steps: bisectState.estimated_steps_remaining ?? '?'
              }) }}
            </div>
          </div>

          <!-- Current checkout commit info -->
          <div class="p-4 bg-surface rounded-xl border border-default/70 space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <UBadge color="primary" variant="subtle" size="sm">
                  HEAD
                </UBadge>
                <span class="font-mono text-xs font-medium text-highlighted">
                  {{ bisectState.current_commit_hash?.slice(0, 10) }}
                </span>
              </div>
              <UButton
                v-if="bisectState.current_commit_hash"
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-copy"
                @click="handleCopyHash(bisectState.current_commit_hash!)"
              />
            </div>
            <p class="text-sm text-highlighted font-medium">
              {{ bisectState.current_commit_summary || t('bisect.unknownSummary') }}
            </p>
          </div>

          <p class="text-xs text-muted">
            {{ t('bisect.testPrompt') }}
          </p>

          <!-- Decision Action Buttons -->
          <div class="grid grid-cols-3 gap-3 pt-2">
            <UButton
              color="success"
              variant="subtle"
              icon="i-lucide-check-circle"
              :label="t('bisect.markGood')"
              :loading="isLoadingBisect"
              class="justify-center"
              @click="handleStep('good')"
            />
            <UButton
              color="error"
              variant="subtle"
              icon="i-lucide-x-circle"
              :label="t('bisect.markBad')"
              :loading="isLoadingBisect"
              class="justify-center"
              @click="handleStep('bad')"
            />
            <UButton
              color="neutral"
              variant="subtle"
              icon="i-lucide-skip-forward"
              :label="t('bisect.skip')"
              :loading="isLoadingBisect"
              class="justify-center"
              @click="handleStep('skip')"
            />
          </div>
        </div>

        <!-- Bisect Log collapsible -->
        <div v-if="bisectState.log_output" class="pt-2 border-t border-default/50 space-y-2">
          <button
            type="button"
            class="flex items-center gap-1.5 text-xs text-muted hover:text-highlighted transition-colors"
            @click="showLog = !showLog"
          >
            <UIcon
              :name="showLog ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
              class="w-3.5 h-3.5"
            />
            <span>{{ t('bisect.toggleLog') }}</span>
          </button>

          <pre
            v-if="showLog"
            class="p-3 bg-neutral-900 text-neutral-200 rounded-lg text-xs font-mono max-h-40 overflow-y-auto whitespace-pre-wrap"
          >{{ bisectState.log_output }}</pre>
        </div>
      </div>

      <!-- Start Bisect Form (when inactive) -->
      <div v-else class="p-6 space-y-5">
        <p class="text-xs text-muted">
          {{ t('bisect.startDescription') }}
        </p>

        <div class="space-y-4">
          <div>
            <label class="block text-xs font-medium text-highlighted mb-1.5">
              {{ t('bisect.badCommitLabel') }}
            </label>
            <UInput
              v-model="badCommitInput"
              :placeholder="t('bisect.badCommitPlaceholder')"
              class="w-full font-mono text-xs"
              icon="i-lucide-x-circle"
            />
            <span class="text-[11px] text-muted mt-1 block">
              {{ t('bisect.badCommitHint') }}
            </span>
          </div>

          <div>
            <label class="block text-xs font-medium text-highlighted mb-1.5">
              {{ t('bisect.goodCommitLabel') }}
            </label>
            <UInput
              v-model="goodCommitInput"
              :placeholder="t('bisect.goodCommitPlaceholder')"
              class="w-full font-mono text-xs"
              icon="i-lucide-check-circle"
            />
            <span class="text-[11px] text-muted mt-1 block">
              {{ t('bisect.goodCommitHint') }}
            </span>
          </div>
        </div>

        <div class="p-3 bg-primary/5 rounded-xl border border-primary/20 text-xs text-muted flex gap-2.5">
          <UIcon name="i-lucide-info" class="w-4 h-4 text-primary shrink-0 mt-0.5" />
          <span>{{ t('bisect.guideTip') }}</span>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-3.5 border-t border-default/60 flex items-center justify-between">
        <div>
          <UButton
            v-if="bisectState && bisectState.is_active && !bisectState.culprit_commit_hash"
            color="error"
            variant="ghost"
            size="sm"
            icon="i-lucide-rotate-ccw"
            :label="t('bisect.abortBisect')"
            :loading="isLoadingBisect"
            @click="handleReset"
          />
        </div>
        <div class="flex items-center gap-2">
          <UButton
            color="neutral"
            variant="outline"
            size="sm"
            :label="t('common.close')"
            @click="emit('update:open', false)"
          />
          <UButton
            v-if="!bisectState?.is_active"
            color="primary"
            size="sm"
            icon="i-lucide-play"
            :label="t('bisect.startAction')"
            :disabled="!canStartBisect"
            :loading="isLoadingBisect"
            @click="handleStartBisect"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
