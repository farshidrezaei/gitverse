<script setup lang="ts">
import { exportArchiveInputSchema } from '~/schemas/archiveSchema'

interface ExportArchiveModalProps {
  open: boolean
}

const props = defineProps<ExportArchiveModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { isOperating, exportSourceArchive } = useArchiveController()

const outputArchivePath = ref<string>('')
const archiveReference = ref<string>('HEAD')
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      errorMessage.value = null
    }
  }
)

async function handleExport(): Promise<void> {
  errorMessage.value = null
  const parseResult = exportArchiveInputSchema.safeParse({
    outputArchivePath: outputArchivePath.value,
    reference: archiveReference.value,
  })

  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  const result = await exportSourceArchive(
    parseResult.data.outputArchivePath,
    parseResult.data.reference
  )
  if (result) {
    emit('update:open', false)
  }
}

function handleClose(): void {
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    class="max-w-xl"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl overflow-hidden focus:outline-none flex flex-col max-h-[85vh]',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-default/60 bg-[var(--gv-surface)]">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-xl bg-primary/10 text-primary flex items-center justify-center">
            <UIcon name="i-lucide-file-archive" class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('archive.exportArchiveModalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('archive.exportArchiveModalSubtitle') }}
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

      <!-- Main Form -->
      <div class="p-6 space-y-4 flex-1 overflow-y-auto">
        <!-- Error Alert Banner -->
        <div
          v-if="errorMessage"
          class="p-3 rounded-xl border border-error/40 bg-error/10 text-error text-xs flex items-center gap-2"
        >
          <UIcon name="i-lucide-alert-circle" class="w-4 h-4 shrink-0" />
          <span>{{ errorMessage }}</span>
        </div>

        <!-- Zero-CLS Skeleton while in flight -->
        <div v-if="isOperating" class="space-y-4">
          <USkeleton class="h-5 w-1/3 rounded" />
          <USkeleton class="h-10 w-full rounded-xl" />
          <USkeleton class="h-10 w-full rounded-xl" />
        </div>

        <div v-else class="space-y-4">
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('archive.archiveOutputPathLabel') }}
            </label>
            <UInput
              v-model="outputArchivePath"
              :placeholder="t('archive.archiveOutputPathPlaceholder')"
              class="font-mono text-xs w-full"
            />
            <p class="text-[11px] text-muted">
              {{ t('archive.archiveOutputPathHint') }}
            </p>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('archive.archiveReferenceLabel') }}
            </label>
            <UInput
              v-model="archiveReference"
              placeholder="HEAD or v1.0.0 or main"
              class="font-mono text-xs w-full"
            />
            <p class="text-[11px] text-muted">
              {{ t('archive.archiveReferenceHint') }}
            </p>
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
          color="primary"
          size="sm"
          :disabled="!outputArchivePath.trim() || isOperating"
          :loading="isOperating"
          @click="handleExport"
        >
          {{ t('archive.exportArchive') }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
