<script setup lang="ts">
const props = defineProps<{
  open: boolean
  sourceBranch?: string | null
  sourceCommitSha?: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  create: [branchName: string, checkout: boolean]
}>()

const { t } = useI18n()

const branchName = ref('')
const shouldCheckout = ref(true)
const isSubmitting = ref(false)
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      branchName.value = ''
      shouldCheckout.value = true
      errorMessage.value = null
      isSubmitting.value = false
    }
  }
)

function validateBranchName(name: string): boolean {
  if (!name.trim()) {
    errorMessage.value = 'Branch name cannot be empty.'
    return false
  }
  if (/\s/.test(name)) {
    errorMessage.value = 'Branch name cannot contain spaces.'
    return false
  }
  if (!/^[a-zA-Z0-9._\-/]+$/.test(name)) {
    errorMessage.value = 'Branch name contains invalid characters.'
    return false
  }
  errorMessage.value = null
  return true
}

function handleConfirm(): void {
  const sanitized = branchName.value.trim()
  if (!validateBranchName(sanitized)) {
    return
  }

  isSubmitting.value = true
  emit('create', sanitized, shouldCheckout.value)
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
            <UIcon name="i-lucide-git-branch-plus" class="h-5 w-5 text-primary" />
            <span class="text-sm font-semibold text-highlighted">{{ t('git.createBranch') }}</span>
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
          <!-- Source Indicator -->
          <div class="flex items-center justify-between p-2.5 rounded-lg border border-default/60 bg-muted/10 text-xs">
            <span class="text-muted font-medium">{{ t('git.sourceRef') }}</span>
            <div class="flex items-center gap-1.5 font-mono">
              <UIcon name="i-lucide-git-commit" class="h-3.5 w-3.5 text-primary" />
              <span class="text-highlighted font-semibold">{{ sourceBranch || sourceCommitSha?.slice(0, 7) || 'HEAD' }}</span>
            </div>
          </div>

          <!-- Branch Name Field -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-highlighted">
              {{ t('git.branchName') }}
            </label>
            <UInput
              v-model="branchName"
              size="sm"
              icon="i-lucide-git-branch"
              :placeholder="t('git.branchNamePlaceholder')"
              class="w-full font-mono text-xs"
              autofocus
              @keydown.enter.prevent="handleConfirm"
            />
            <p v-if="errorMessage" class="text-[11px] text-error font-medium">
              {{ errorMessage }}
            </p>
          </div>

          <!-- Checkout Checkbox -->
          <UCheckbox
            v-model="shouldCheckout"
            :label="t('git.checkoutNewBranch')"
            size="sm"
          />
        </div>

        <!-- Action Footer -->
        <div class="flex items-center justify-end gap-2 border-t border-default/70 px-5 py-3 bg-muted/10">
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
            :label="t('git.createBranch')"
            :loading="isSubmitting"
            @click="handleConfirm"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
