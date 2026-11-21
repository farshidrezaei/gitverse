<script setup lang="ts">
import type { CommitQueueOperationResult } from '~/types/repository'

interface CommitQueueModalProps {
  open: boolean
  mode: 'cherryPick' | 'revert'
  initialCommitHashes?: string[]
}

const props = defineProps<CommitQueueModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  complete: []
}>()

const { t } = useI18n()
const { isExecutingQueue, executeCherryPickQueue, executeRevertQueue } = useCommitQueueController()

const commitHashesInput = ref<string>('')
const shouldCommit = ref<boolean>(true)
const operationOutcome = ref<CommitQueueOperationResult | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      if (props.initialCommitHashes && props.initialCommitHashes.length > 0) {
        commitHashesInput.value = props.initialCommitHashes.join('\n')
      }
      operationOutcome.value = null
    }
  },
  { immediate: true }
)

const parsedCommitHashes = computed<string[]>(() => {
  return commitHashesInput.value
    .split(/[\n,\s]+/)
    .map((hash) => hash.trim())
    .filter((hash) => hash.length > 0)
})

async function handleRunQueue(): Promise<void> {
  if (parsedCommitHashes.value.length === 0) return

  let outcome: CommitQueueOperationResult | null = null
  if (props.mode === 'cherryPick') {
    outcome = await executeCherryPickQueue(parsedCommitHashes.value, shouldCommit.value)
  } else {
    outcome = await executeRevertQueue(parsedCommitHashes.value, shouldCommit.value)
  }

  operationOutcome.value = outcome
  if (outcome?.is_successful) {
    emit('complete')
  }
}

function handleClose(): void {
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    class="max-w-2xl"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl overflow-hidden focus:outline-none flex flex-col max-h-[85vh]',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-default/60 bg-[var(--gv-surface)]">
        <div class="flex items-center gap-3">
          <div
            class="p-2 rounded-xl flex items-center justify-center"
            :class="mode === 'cherryPick' ? 'bg-primary/10 text-primary' : 'bg-warning/10 text-warning'"
          >
            <UIcon :name="mode === 'cherryPick' ? 'i-lucide-git-cherry-pick' : 'i-lucide-rotate-ccw'" class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ mode === 'cherryPick' ? t('commitQueue.cherryPickTitle') : t('commitQueue.revertTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ mode === 'cherryPick' ? t('commitQueue.cherryPickSubtitle') : t('commitQueue.revertSubtitle') }}
            </p>
          </div>
        </div>

        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          class="rounded-lg"
          @click="handleClose"
        />
      </div>

      <!-- Main Configuration Form -->
      <div class="p-6 space-y-4 flex-1 overflow-y-auto">
        <!-- Zero-CLS Skeleton while executing -->
        <div v-if="isExecutingQueue" class="space-y-4">
          <USkeleton class="h-6 w-1/3 rounded" />
          <USkeleton class="h-28 w-full rounded-xl" />
          <div class="flex items-center justify-between">
            <USkeleton class="h-6 w-40 rounded" />
            <USkeleton class="h-8 w-24 rounded-lg" />
          </div>
        </div>

        <div v-else class="space-y-4">
          <!-- Commit List Input -->
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted flex items-center justify-between">
              <span>{{ t('commitQueue.commitsInputLabel') }}</span>
              <span class="text-muted text-xs">
                {{ t('commitQueue.commitCount', { count: parsedCommitHashes.length }) }}
              </span>
            </label>
            <UTextarea
              v-model="commitHashesInput"
              :rows="4"
              class="font-mono text-xs w-full"
              :placeholder="t('commitQueue.commitsInputPlaceholder')"
            />
            <p class="text-[11px] text-muted">
              {{ t('commitQueue.commitsInputHint') }}
            </p>
          </div>

          <!-- Options -->
          <div class="p-3 border border-default/40 rounded-xl bg-muted/5 flex items-center justify-between">
            <div class="space-y-0.5">
              <span class="text-xs font-medium text-highlighted">
                {{ t('commitQueue.autoCommitOption') }}
              </span>
              <p class="text-[11px] text-muted">
                {{ t('commitQueue.autoCommitDescription') }}
              </p>
            </div>
            <USwitch v-model="shouldCommit" />
          </div>

          <!-- Operation Outcome Summary (if executed) -->
          <div
            v-if="operationOutcome"
            class="p-4 rounded-xl border text-xs space-y-2"
            :class="operationOutcome.is_successful ? 'border-success/30 bg-success/10 text-success' : 'border-error/30 bg-error/10 text-error'"
          >
            <div class="flex items-center gap-2 font-semibold">
              <UIcon :name="operationOutcome.is_successful ? 'i-lucide-check-circle-2' : 'i-lucide-alert-triangle'" class="w-4 h-4" />
              <span>{{ operationOutcome.message }}</span>
            </div>

            <!-- If Conflict occurred -->
            <div v-if="!operationOutcome.is_successful && operationOutcome.conflict_files.length > 0" class="space-y-1 pt-1 text-muted">
              <span class="font-medium text-highlighted">{{ t('commitQueue.conflictedFilesTitle') }}:</span>
              <ul class="list-disc ps-5 font-mono text-[11px] space-y-0.5">
                <li v-for="file in operationOutcome.conflict_files" :key="file">
                  {{ file }}
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      <!-- Action Footer -->
      <div class="flex items-center justify-between px-6 py-3.5 border-t border-default/60 bg-[var(--gv-surface)]">
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          @click="handleClose"
        >
          {{ t('common.close') }}
        </UButton>

        <UButton
          :color="mode === 'cherryPick' ? 'primary' : 'warning'"
          size="sm"
          :disabled="parsedCommitHashes.length === 0 || isExecutingQueue"
          :loading="isExecutingQueue"
          @click="handleRunQueue"
        >
          {{ mode === 'cherryPick' ? t('commitQueue.startCherryPick') : t('commitQueue.startRevert') }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
