<script setup lang="ts">
import { validateGitBranchName } from '~/utils/branchValidation'

interface CreateWorktreeModalProps {
  open: boolean
  defaultDirectoryPath?: string
  existingBranchNames?: string[]
}

const props = defineProps<CreateWorktreeModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  create: [path: string, branchName: string | undefined, isNewBranch: boolean]
}>()

const { t } = useI18n()

const worktreePath = ref<string>('')
const selectedBranch = ref<string>('')
const isNewBranch = ref<boolean>(true)
const newBranchName = ref<string>('')
const isSubmitting = ref<boolean>(false)
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      const baseName = props.defaultDirectoryPath ? `${props.defaultDirectoryPath}-worktree` : ''
      worktreePath.value = baseName
      selectedBranch.value = props.existingBranchNames?.[0] || 'main'
      newBranchName.value = 'feature-worktree'
      isNewBranch.value = true
      errorMessage.value = null
      isSubmitting.value = false
    }
  },
)

function handleConfirm(): void {
  const sanitizedPath = worktreePath.value.trim()
  if (!sanitizedPath) {
    errorMessage.value = t('worktree.pathRequired')
    return
  }

  if (isNewBranch.value) {
    const branchToCreate = newBranchName.value.trim()
    const validation = validateGitBranchName(branchToCreate, {
      existingBranchNames: props.existingBranchNames,
    })
    if (!validation.isValid && validation.errorKey) {
      errorMessage.value = t(validation.errorKey)
      return
    }
    isSubmitting.value = true
    emit('create', sanitizedPath, branchToCreate, true)
  } else {
    isSubmitting.value = true
    emit('create', sanitizedPath, selectedBranch.value, false)
  }
}

function handleClose(): void {
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-lg p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col select-none text-xs">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-git-fork" class="h-5 w-5 text-primary" />
            <span class="text-sm font-semibold text-highlighted">{{ t('worktree.createTitle') }}</span>
          </div>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            @click="handleClose"
          />
        </div>

        <!-- Form Body -->
        <div class="p-5 space-y-4">
          <!-- Worktree Destination Path -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-highlighted">
              {{ t('worktree.directoryPathLabel') }}
            </label>
            <div class="flex items-center gap-2">
              <UInput
                v-model="worktreePath"
                class="w-full font-mono text-xs"
                placeholder="/path/to/new-worktree"
                size="sm"
              />
            </div>
            <p class="text-[11px] text-muted">{{ t('worktree.directoryPathHelp') }}</p>
          </div>

          <!-- Branch Mode Selection -->
          <div class="space-y-2 pt-1 border-t border-default/50">
            <div class="flex items-center gap-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="isNewBranch"
                  type="radio"
                  :value="true"
                  class="accent-primary"
                >
                <span class="font-medium text-highlighted">{{ t('worktree.newBranchRadio') }}</span>
              </label>

              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="isNewBranch"
                  type="radio"
                  :value="false"
                  class="accent-primary"
                >
                <span class="font-medium text-highlighted">{{ t('worktree.existingBranchRadio') }}</span>
              </label>
            </div>

            <!-- New Branch Input -->
            <div v-if="isNewBranch" class="space-y-1.5 pt-1">
              <label class="text-xs font-semibold text-highlighted">
                {{ t('worktree.newBranchNameLabel') }}
              </label>
              <UInput
                v-model="newBranchName"
                class="w-full font-mono text-xs"
                placeholder="feature/isolated-task"
                size="sm"
              />
            </div>

            <!-- Existing Branch Select -->
            <div v-else class="space-y-1.5 pt-1">
              <label class="text-xs font-semibold text-highlighted">
                {{ t('worktree.selectExistingBranchLabel') }}
              </label>
              <select
                v-model="selectedBranch"
                class="w-full rounded-lg border border-default bg-surface px-3 py-1.5 text-xs text-highlighted focus:outline-primary"
              >
                <option
                  v-for="branch in props.existingBranchNames || []"
                  :key="branch"
                  :value="branch"
                >
                  {{ branch }}
                </option>
              </select>
            </div>
          </div>

          <!-- Error Alert Banner -->
          <div
            v-if="errorMessage"
            class="flex items-center gap-2 p-2.5 rounded-lg border border-error/40 bg-error/10 text-error text-[11px]"
          >
            <UIcon name="i-lucide-alert-circle" class="h-4 w-4 shrink-0" />
            <span>{{ errorMessage }}</span>
          </div>
        </div>

        <!-- Footer Actions -->
        <div class="flex items-center justify-end gap-2 border-t border-default/70 px-5 py-3 bg-muted/15">
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="handleClose"
          />
          <UButton
            size="sm"
            color="primary"
            variant="solid"
            icon="i-lucide-plus"
            :loading="isSubmitting"
            :label="t('worktree.createButton')"
            @click="handleConfirm"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
