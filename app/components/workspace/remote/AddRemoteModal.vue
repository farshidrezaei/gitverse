<script setup lang="ts">
interface AddRemoteModalProps {
  open: boolean
  existingRemoteNames?: string[]
}

const props = defineProps<AddRemoteModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  add: [name: string, url: string, shouldFetchImmediately: boolean]
}>()

const { t } = useI18n()

const remoteName = ref<string>('')
const remoteUrl = ref<string>('')
const shouldFetchImmediately = ref<boolean>(true)
const errorMessage = ref<string | null>(null)
const isSubmitting = ref<boolean>(false)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      remoteName.value = props.existingRemoteNames?.includes('origin') ? 'upstream' : 'origin'
      remoteUrl.value = ''
      shouldFetchImmediately.value = true
      errorMessage.value = null
      isSubmitting.value = false
    }
  },
)

function handleConfirm(): void {
  const sanitizedName = remoteName.value.trim()
  const sanitizedUrl = remoteUrl.value.trim()

  if (!sanitizedName) {
    errorMessage.value = t('remote.validationEmptyName')
    return
  }

  if (/\s/.test(sanitizedName)) {
    errorMessage.value = t('remote.validationInvalidName')
    return
  }

  if (props.existingRemoteNames?.includes(sanitizedName)) {
    errorMessage.value = t('remote.validationDuplicateName')
    return
  }

  if (!sanitizedUrl) {
    errorMessage.value = t('remote.validationEmptyUrl')
    return
  }

  isSubmitting.value = true
  emit('add', sanitizedName, sanitizedUrl, shouldFetchImmediately.value)
}
</script>

<template>
  <UModal
    :open="props.open"
    :title="t('remote.addRemoteTitle')"
    :description="t('remote.addRemoteDescription')"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="p-6 space-y-4">
        <!-- Header details -->
        <div class="flex items-center gap-3">
          <div class="h-10 w-10 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center shrink-0">
            <UIcon name="i-lucide-globe" class="h-5 w-5 text-primary" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-highlighted">
              {{ t('remote.addRemoteTitle') }}
            </h3>
            <p class="text-xs text-muted">
              {{ t('remote.addRemoteDescription') }}
            </p>
          </div>
        </div>

        <div class="space-y-3 pt-2">
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
              @keydown.enter="handleConfirm"
            />
          </div>

          <!-- Immediate Fetch Option -->
          <div class="pt-1">
            <UCheckbox
              v-model="shouldFetchImmediately"
              :label="t('remote.fetchAfterAdd')"
              class="text-xs font-medium text-highlighted"
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
        <div class="flex items-center justify-end gap-2 pt-3 border-t border-default">
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="emit('update:open', false)"
          />
          <UButton
            size="sm"
            color="primary"
            icon="i-lucide-plus"
            :loading="isSubmitting"
            :label="t('remote.addRemote')"
            @click="handleConfirm"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
