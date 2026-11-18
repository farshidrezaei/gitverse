<script setup lang="ts">
import { validateGitBranchName, sanitizeBranchNameInput } from '~/utils/branchValidation'

const props = defineProps<{
  open: boolean
  currentBranchName: string
  existingBranchNames?: string[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  rename: [oldName: string, newName: string]
}>()

const { t } = useI18n()

const newBranchName = ref('')
const isSubmitting = ref(false)
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      newBranchName.value = props.currentBranchName
      errorMessage.value = null
      isSubmitting.value = false
    }
  }
)

function validateBranchName(name: string): boolean {
  const validationResult = validateGitBranchName(name, {
    currentBranchName: props.currentBranchName,
    existingBranchNames: props.existingBranchNames,
  })

  if (!validationResult.isValid && validationResult.errorKey) {
    errorMessage.value = t(validationResult.errorKey)
    return false
  }

  errorMessage.value = null
  return true
}

function handleBranchNameInput(value: string | number): void {
  const sanitized = sanitizeBranchNameInput(String(value ?? ''))
  newBranchName.value = sanitized
  if (errorMessage.value) {
    validateBranchName(sanitized)
  }
}

function handleConfirm(): void {
  const sanitized = newBranchName.value.trim()
  if (!validateBranchName(sanitized)) {
    return
  }

  isSubmitting.value = true
  emit('rename', props.currentBranchName, sanitized)
  emit('update:open', false)
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
      content: 'sm:max-w-md p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col select-none">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-pencil-line" class="h-5 w-5 text-primary" />
            <span class="text-sm font-semibold text-highlighted">{{ t('branch.renameTitle') }}</span>
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
          <!-- Current Name Indicator -->
          <div class="flex items-center justify-between p-2.5 rounded-lg border border-default/60 bg-muted/10 text-xs">
            <span class="text-muted font-medium">{{ t('branch.currentNameLabel') }}</span>
            <div class="flex items-center gap-1.5 font-mono">
              <UIcon name="i-lucide-git-branch" class="h-3.5 w-3.5 text-primary" />
              <span class="text-highlighted font-semibold">{{ currentBranchName }}</span>
            </div>
          </div>

          <!-- New Branch Name Field -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-highlighted">
              {{ t('branch.newNameLabel') }}
            </label>
            <UInput
              :model-value="newBranchName"
              size="sm"
              icon="i-lucide-git-branch"
              :placeholder="currentBranchName"
              class="w-full font-mono text-xs"
              autofocus
              @update:model-value="handleBranchNameInput"
              @keydown.enter.prevent="handleConfirm"
            />
            <p v-if="errorMessage" class="text-[11px] text-error font-medium">
              {{ errorMessage }}
            </p>
          </div>

          <!-- Action Buttons -->
          <div class="flex items-center justify-end gap-2 pt-2 border-t border-default/70">
            <UButton
              size="sm"
              color="neutral"
              variant="outline"
              :label="t('common.cancel')"
              @click="handleClose"
            />
            <UButton
              size="sm"
              color="primary"
              :loading="isSubmitting"
              :label="t('branch.renameAction')"
              @click="handleConfirm"
            />
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>
