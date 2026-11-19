<script setup lang="ts">
import { createStashInputSchema } from '~/schemas/stashSchema'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  save: [message: string, includeUntracked: boolean]
}>()

const { t } = useI18n()

const stashMessage = ref('')
const includeUntracked = ref(true)
const isSubmitting = ref(false)
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      stashMessage.value = ''
      includeUntracked.value = true
      errorMessage.value = null
      isSubmitting.value = false
    }
  }
)

function handleConfirm(): void {
  const result = createStashInputSchema.safeParse({
    message: stashMessage.value.trim() || undefined,
    shouldIncludeUntracked: includeUntracked.value,
  })

  if (!result.success) {
    const firstIssue = result.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : 'Invalid input'
    return
  }

  isSubmitting.value = true
  emit('save', stashMessage.value.trim(), includeUntracked.value)
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
            <UIcon name="i-lucide-archive" class="h-5 w-5 text-primary" />
            <span class="text-sm font-semibold text-highlighted">{{ t('stash.createTitle') }}</span>
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
          <p class="text-xs text-muted">
            {{ t('stash.createDescription') }}
          </p>

          <div class="space-y-1.5">
            <label class="block text-xs font-semibold text-highlighted">
              {{ t('stash.messageLabel') }}
            </label>
            <UInput
              v-model="stashMessage"
              size="sm"
              :placeholder="t('stash.messagePlaceholder')"
              class="w-full"
              autofocus
              @keydown.enter.prevent="handleConfirm"
            />
            <p v-if="errorMessage" class="text-[11px] text-error font-medium">
              {{ errorMessage }}
            </p>
          </div>

          <div class="pt-1">
            <UCheckbox
              v-model="includeUntracked"
              :label="t('stash.includeUntracked')"
              class="text-xs font-medium text-highlighted"
            />
          </div>
        </div>

        <!-- Footer -->
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
            icon="i-lucide-archive"
            :loading="isSubmitting"
            :label="t('stash.createConfirm')"
            @click="handleConfirm"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
