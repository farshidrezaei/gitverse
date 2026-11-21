<script setup lang="ts">
import type { PatchCheckResult } from '~/types/repository'
import { applyPatchInputSchema, patchContentSchema } from '~/schemas/patchSchema'

interface ApplyPatchModalProps {
  open: boolean
}

const props = defineProps<ApplyPatchModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { checkPatch, applyPatch, isOperating } = usePatchController()

const patchContentInput = ref<string>('')
const checkResult = ref<PatchCheckResult | null>(null)
const isApplyToStaged = ref<boolean>(false)
const isReverse = ref<boolean>(false)
const errorMessage = ref<string | null>(null)

watch(
  patchContentInput,
  () => {
    // Invalidate previous check result when content changes
    checkResult.value = null
    errorMessage.value = null
  }
)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      errorMessage.value = null
    }
  }
)

async function handleValidate() {
  errorMessage.value = null
  const parseResult = patchContentSchema.safeParse(patchContentInput.value)
  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  checkResult.value = await checkPatch(parseResult.data)
}

async function handleApply() {
  errorMessage.value = null
  const parseResult = applyPatchInputSchema.safeParse({
    patchContent: patchContentInput.value,
    isCached: isApplyToStaged.value,
    isReverse: isReverse.value,
  })

  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  const success = await applyPatch(
    parseResult.data.patchContent,
    parseResult.data.isCached,
    parseResult.data.isReverse
  )
  if (success) {
    emit('update:open', false)
  }
}

function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    patchContentInput.value = (e.target?.result as string) || ''
  }
  reader.readAsText(file)
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
            <UIcon name="i-lucide-file-input" class="w-5 h-5 text-primary" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('patch.applyTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('patch.applySubtitle') }}
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
        <!-- Error Alert Banner -->
        <div
          v-if="errorMessage"
          class="p-3 rounded-xl border border-error/40 bg-error/10 text-error text-xs flex items-center gap-2"
        >
          <UIcon name="i-lucide-alert-circle" class="w-4 h-4 shrink-0" />
          <span>{{ errorMessage }}</span>
        </div>
        <!-- Input instructions and file upload button -->
        <div class="flex items-center justify-between">
          <label class="text-xs font-medium text-highlighted">
            {{ t('patch.pasteOrUploadLabel') }}
          </label>
          <label class="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg bg-neutral-500/10 hover:bg-neutral-500/20 text-highlighted cursor-pointer transition-colors">
            <UIcon name="i-lucide-upload" class="w-3.5 h-3.5" />
            <span>{{ t('patch.chooseFile') }}</span>
            <input
              type="file"
              accept=".patch,.diff,text/plain"
              class="hidden"
              @change="handleFileUpload"
            >
          </label>
        </div>

        <!-- Patch textarea -->
        <UTextarea
          v-model="patchContentInput"
          :placeholder="t('patch.inputPlaceholder')"
          :rows="7"
          class="font-mono text-xs w-full"
        />

        <!-- Validation & Diagnostic Result -->
        <div v-if="isOperating" class="space-y-2">
          <USkeleton class="h-10 w-full rounded-xl" />
        </div>

        <div
          v-else-if="checkResult"
          class="p-4 rounded-xl border text-xs space-y-2"
          :class="checkResult.can_apply ? 'border-success/40 bg-success/10' : 'border-error/40 bg-error/10'"
        >
          <div class="flex items-center gap-2 font-medium" :class="checkResult.can_apply ? 'text-success' : 'text-error'">
            <UIcon
              :name="checkResult.can_apply ? 'i-lucide-check-circle' : 'i-lucide-alert-triangle'"
              class="w-4 h-4"
            />
            <span>
              {{ checkResult.can_apply ? t('patch.checkSuccess') : t('patch.checkFailed') }}
            </span>
          </div>

          <p v-if="checkResult.error_message" class="text-muted font-mono whitespace-pre-wrap">
            {{ checkResult.error_message }}
          </p>

          <div v-if="checkResult.affected_files.length > 0" class="pt-1">
            <span class="text-highlighted font-medium block mb-1">
              {{ t('patch.affectedFiles') }}:
            </span>
            <ul class="list-disc list-inside space-y-0.5 text-muted font-mono">
              <li v-for="file in checkResult.affected_files" :key="file">
                {{ file }}
              </li>
            </ul>
          </div>
        </div>

        <!-- Options checkboxes -->
        <div class="pt-2 border-t border-default/50 flex flex-wrap gap-4 text-xs">
          <UCheckbox
            v-model="isApplyToStaged"
            :label="t('patch.applyToStaged')"
            size="xs"
          />
          <UCheckbox
            v-model="isReverse"
            :label="t('patch.reversePatch')"
            size="xs"
          />
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
            icon="i-lucide-shield-check"
            :label="t('patch.validateAction')"
            :disabled="!patchContentInput.trim() || isOperating"
            :loading="isOperating"
            @click="handleValidate"
          />
          <UButton
            color="primary"
            size="sm"
            icon="i-lucide-play"
            :label="t('patch.applyAction')"
            :disabled="!patchContentInput.trim() || isOperating"
            :loading="isOperating"
            @click="handleApply"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
