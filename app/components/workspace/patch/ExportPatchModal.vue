<script setup lang="ts">
import type { PatchExportScope } from '~/types/repository'

interface ExportPatchModalProps {
  open: boolean
  initialCommitHash?: string | null
}

const props = defineProps<ExportPatchModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { notifySuccess } = useNotificationService()
const { exportPatch, downloadPatchFile, isOperating } = usePatchController()

const selectedScope = ref<PatchExportScope>(props.initialCommitHash ? 'commit' : 'working_tree')
const commitHashInput = ref<string>(props.initialCommitHash || '')
const generatedPatch = ref<string>('')

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      if (props.initialCommitHash) {
        selectedScope.value = 'commit'
        commitHashInput.value = props.initialCommitHash
      }
      await handleGeneratePatch()
    }
  },
  { immediate: true }
)

const scopeOptions = computed(() => [
  { label: t('patch.scopeWorkingTree'), value: 'working_tree' },
  { label: t('patch.scopeStaged'), value: 'staged' },
  { label: t('patch.scopeUnstaged'), value: 'unstaged' },
  { label: t('patch.scopeCommit'), value: 'commit' },
])

async function handleGeneratePatch() {
  const result = await exportPatch(
    selectedScope.value,
    selectedScope.value === 'commit' ? commitHashInput.value.trim() : undefined
  )
  generatedPatch.value = result || ''
}

function handleCopy() {
  if (!generatedPatch.value) return
  navigator.clipboard.writeText(generatedPatch.value)
  notifySuccess(t('patch.notifications.copied'))
}

function handleDownload() {
  if (!generatedPatch.value) return
  const filename =
    selectedScope.value === 'commit' && commitHashInput.value
      ? `commit-${commitHashInput.value.slice(0, 8)}.patch`
      : `${selectedScope.value}-changes.patch`
  downloadPatchFile(generatedPatch.value, filename)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl max-w-2xl overflow-hidden focus:outline-none',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Header -->
      <div class="px-6 py-4 border-b border-default/60 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
            <UIcon name="i-lucide-file-diff" class="w-5 h-5 text-primary" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('patch.exportTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('patch.exportSubtitle') }}
            </p>
          </div>
        </div>
        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          @click="emit('update:open', false)"
        />
      </div>

      <!-- Body -->
      <div class="p-6 space-y-4">
        <!-- Scope Selector -->
        <div>
          <label class="block text-xs font-medium text-highlighted mb-1.5">
            {{ t('patch.scopeLabel') }}
          </label>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
            <UButton
              v-for="option in scopeOptions"
              :key="option.value"
              size="xs"
              :color="selectedScope === option.value ? 'primary' : 'neutral'"
              :variant="selectedScope === option.value ? 'solid' : 'subtle'"
              :label="option.label"
              class="justify-center"
              @click="selectedScope = option.value as PatchExportScope; handleGeneratePatch()"
            />
          </div>
        </div>

        <!-- Commit Hash input if commit scope -->
        <div v-if="selectedScope === 'commit'" class="space-y-1">
          <label class="block text-xs font-medium text-highlighted">
            {{ t('patch.commitHashLabel') }}
          </label>
          <div class="flex gap-2">
            <UInput
              v-model="commitHashInput"
              placeholder="e.g. 4ba8dfc or HEAD~1"
              class="w-full font-mono text-xs"
            />
            <UButton
              size="xs"
              color="primary"
              variant="soft"
              :label="t('common.refresh')"
              :loading="isOperating"
              @click="handleGeneratePatch"
            />
          </div>
        </div>

        <!-- Zero-CLS Skeleton while generating -->
        <div v-if="isOperating" class="space-y-2">
          <USkeleton class="h-6 w-32 rounded" />
          <USkeleton class="h-48 w-full rounded-xl" />
        </div>

        <!-- Patch Preview Content -->
        <div v-else class="space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium text-highlighted">
              {{ t('patch.previewLabel') }}
            </span>
            <span class="text-[11px] text-muted">
              {{ generatedPatch ? `${generatedPatch.split('\n').length} ${t('patch.linesCount')}` : t('patch.emptyPatch') }}
            </span>
          </div>

          <pre
            v-if="generatedPatch"
            class="p-3 bg-neutral-900 text-neutral-200 rounded-xl text-xs font-mono max-h-64 overflow-y-auto whitespace-pre-wrap selection:bg-primary/30"
          >{{ generatedPatch }}</pre>

          <div
            v-else
            class="p-6 rounded-xl border border-default/50 bg-neutral-500/5 text-center text-xs text-muted"
          >
            {{ t('patch.noChangesToExport') }}
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="px-6 py-3.5 border-t border-default/60 flex items-center justify-between">
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          :label="t('common.close')"
          @click="emit('update:open', false)"
        />
        <div class="flex items-center gap-2">
          <UButton
            color="neutral"
            variant="subtle"
            size="sm"
            icon="i-lucide-copy"
            :label="t('patch.copyPatch')"
            :disabled="!generatedPatch || isOperating"
            @click="handleCopy"
          />
          <UButton
            color="primary"
            size="sm"
            icon="i-lucide-download"
            :label="t('patch.downloadFile')"
            :disabled="!generatedPatch || isOperating"
            @click="handleDownload"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
