<script setup lang="ts">
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

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      stashMessage.value = ''
      includeUntracked.value = true
      isSubmitting.value = false
    }
  }
)

function handleConfirm(): void {
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
    @update:open="handleClose"
  >
    <div class="p-5 space-y-4">
      <!-- Modal Header -->
      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
          <UIcon name="i-lucide-archive" class="h-5 w-5" />
        </div>
        <div class="min-w-0 flex-1">
          <h3 class="text-sm font-bold text-highlighted">
            {{ t('stash.createTitle') }}
          </h3>
          <p class="text-xs text-muted mt-0.5">
            {{ t('stash.createDescription') }}
          </p>
        </div>
      </div>

      <!-- Form Inputs -->
      <div class="space-y-3 pt-1">
        <div>
          <label class="block text-xs font-semibold text-highlighted mb-1.5">
            {{ t('stash.messageLabel') }}
          </label>
          <UInput
            v-model="stashMessage"
            size="sm"
            :placeholder="t('stash.messagePlaceholder')"
            class="w-full"
            autofocus
            @keydown.enter="handleConfirm"
          />
        </div>

        <div class="flex items-center gap-2 pt-1">
          <UCheckbox
            v-model="includeUntracked"
            :label="t('stash.includeUntracked')"
            size="sm"
          />
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-end gap-2 pt-3 border-t border-default/70">
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
          icon="i-lucide-archive"
          :loading="isSubmitting"
          :label="t('stash.createConfirm')"
          @click="handleConfirm"
        />
      </div>
    </div>
  </UModal>
</template>
