<script setup lang="ts">
import { useActionGuard } from '~/composables/useActionGuard'

const { preflightModal, closePreflightModal } = useActionGuard()
const { t } = useI18n()

async function handleProceed(): Promise<void> {
  if (preflightModal.value.isExecutingResolution) return
  preflightModal.value.isExecutingResolution = true
  try {
    await preflightModal.value.onProceedWithResolution()
    closePreflightModal()
  } catch (error) {
    console.error('Failed to execute preflight suggested resolution:', error)
  } finally {
    preflightModal.value.isExecutingResolution = false
  }
}

function handleCancel(): void {
  closePreflightModal()
}
</script>

<template>
  <UModal
    :open="preflightModal.isOpen"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content:
        'sm:max-w-md p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="(value: boolean) => { if (!value) handleCancel() }"
  >
    <template #content>
      <div class="flex flex-col select-none">
        <!-- Header -->
        <div
          class="flex items-center gap-3 border-b border-default/70 p-5 bg-muted/15"
        >
          <div
            class="flex items-center justify-center size-10 rounded-xl shrink-0 bg-amber-500/10 text-amber-500 border border-amber-500/20"
          >
            <UIcon name="i-lucide-alert-triangle" class="size-5" />
          </div>
          <div class="min-w-0 flex-1">
            <h3
              class="text-sm font-bold text-highlighted truncate leading-tight"
            >
              {{ t(preflightModal.titleKey) }}
            </h3>
            <p class="text-xs text-muted-foreground mt-0.5">
              {{ t('common.warning') }}
            </p>
          </div>
          <UButton
            icon="i-lucide-x"
            color="neutral"
            variant="ghost"
            size="xs"
            class="rounded-lg"
            :aria-label="t('common.close')"
            @click="handleCancel"
          />
        </div>

        <!-- Body -->
        <div class="p-5 space-y-4">
          <p class="text-sm text-foreground leading-relaxed">
            {{ t(preflightModal.descriptionKey) }}
          </p>
        </div>

        <!-- Footer -->
        <div
          class="flex items-center justify-end gap-2.5 border-t border-default/70 p-4 bg-muted/10"
        >
          <UButton
            color="neutral"
            variant="ghost"
            size="sm"
            class="rounded-xl px-4"
            :disabled="preflightModal.isExecutingResolution"
            @click="handleCancel"
          >
            {{ t('common.cancel') }}
          </UButton>
          <UButton
            color="primary"
            variant="solid"
            size="sm"
            class="rounded-xl px-4"
            :loading="preflightModal.isExecutingResolution"
            @click="handleProceed"
          >
            {{ t(preflightModal.actionButtonKey) }}
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>
