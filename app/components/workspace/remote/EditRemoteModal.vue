<script setup lang="ts">
import type { RemoteSummary } from '~/types/repository'
import { validateRemoteWithSchema } from '~/schemas/remoteSchema'

interface EditRemoteModalProps {
  open: boolean
  remote: RemoteSummary | null
  existingRemoteNames?: string[]
}

const props = defineProps<EditRemoteModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  save: [oldName: string, newName: string, newUrl: string]
}>()

const { t } = useI18n()

const remoteName = ref<string>('')
const remoteUrl = ref<string>('')
const errorMessage = ref<string | null>(null)
const isSubmitting = ref<boolean>(false)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen && props.remote) {
      remoteName.value = props.remote.name
      remoteUrl.value = props.remote.fetch_url || props.remote.push_url || ''
      errorMessage.value = null
      isSubmitting.value = false
    }
  },
)

function handleClose(): void {
  emit('update:open', false)
}

function handleConfirm(): void {
  if (!props.remote) return

  const sanitizedName = remoteName.value.trim()
  const sanitizedUrl = remoteUrl.value.trim()

  const validationResult = validateRemoteWithSchema(sanitizedName, sanitizedUrl, {
    existingRemoteNames: props.existingRemoteNames,
    currentRemoteName: props.remote.name,
  })

  if (!validationResult.isValid && validationResult.errorKey) {
    errorMessage.value = t(validationResult.errorKey)
    return
  }

  isSubmitting.value = true
  emit('save', props.remote.name, sanitizedName, sanitizedUrl)
}
</script>

<template>
  <UModal
    :open="props.open"
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
            <UIcon name="i-lucide-pencil" class="h-5 w-5 text-primary" />
            <span class="text-sm font-semibold text-highlighted">{{ t('remote.editRemoteTitle') }}</span>
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
          <!-- Remote Name Input -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-highlighted">
              {{ t('remote.remoteNameLabel') }}
            </label>
            <UInput
              v-model="remoteName"
              icon="i-lucide-tag"
              size="sm"
              class="w-full font-mono text-xs"
              :placeholder="t('remote.remoteNamePlaceholder')"
              autofocus
            />
          </div>

          <!-- Remote URL Input -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-highlighted">
              {{ t('remote.remoteUrlLabel') }}
            </label>
            <UInput
              v-model="remoteUrl"
              icon="i-lucide-link"
              size="sm"
              class="w-full font-mono text-xs"
              :placeholder="t('remote.remoteUrlPlaceholder')"
              @keydown.enter.prevent="handleConfirm"
            />
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
            icon="i-lucide-check"
            :loading="isSubmitting"
            :label="t('common.save')"
            @click="handleConfirm"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
