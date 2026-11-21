<script setup lang="ts">
import type { GitBundleVerificationResult } from '~/types/repository'
import { createBundleInputSchema, verifyBundleInputSchema } from '~/schemas/archiveSchema'

interface GitBundleModalProps {
  open: boolean
}

const props = defineProps<GitBundleModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const {
  isOperating,
  bundleVerificationResult,
  createGitBundle,
  verifyGitBundle,
} = useArchiveController()

const activeTab = ref<'create' | 'verify'>('create')
const outputBundlePath = ref<string>('')
const revisionRange = ref<string>('--all')
const verifyFilePath = ref<string>('')
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      errorMessage.value = null
    }
  }
)

async function handleCreateBundle(): Promise<void> {
  errorMessage.value = null
  const parseResult = createBundleInputSchema.safeParse({
    outputBundlePath: outputBundlePath.value,
    revisionRange: revisionRange.value,
  })

  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  const result = await createGitBundle(
    parseResult.data.outputBundlePath,
    parseResult.data.revisionRange
  )
  if (result) {
    emit('update:open', false)
  }
}

async function handleVerifyBundle(): Promise<void> {
  errorMessage.value = null
  const parseResult = verifyBundleInputSchema.safeParse({
    bundlePath: verifyFilePath.value,
  })

  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  await verifyGitBundle(parseResult.data.bundlePath)
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
            <UIcon name="i-lucide-package" class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('archive.bundleModalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('archive.bundleModalSubtitle') }}
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

      <!-- Segmented Tabs -->
      <div class="px-6 pt-4 border-b border-default/40 bg-muted/5 flex items-center gap-4 text-xs font-medium">
        <button
          type="button"
          class="pb-2.5 border-b-2 transition-colors"
          :class="activeTab === 'create' ? 'border-primary text-highlighted' : 'border-transparent text-muted hover:text-highlighted'"
          @click="activeTab = 'create'"
        >
          {{ t('archive.tabCreateBundle') }}
        </button>
        <button
          type="button"
          class="pb-2.5 border-b-2 transition-colors"
          :class="activeTab === 'verify' ? 'border-primary text-highlighted' : 'border-transparent text-muted hover:text-highlighted'"
          @click="activeTab = 'verify'"
        >
          {{ t('archive.tabVerifyBundle') }}
        </button>
      </div>

      <!-- Main Body -->
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

        <!-- Create Bundle Tab Content -->
        <div v-else-if="activeTab === 'create'" class="space-y-4">
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('archive.outputPathLabel') }}
            </label>
            <UInput
              v-model="outputBundlePath"
              :placeholder="t('archive.outputPathPlaceholder')"
              class="font-mono text-xs w-full"
            />
            <p class="text-[11px] text-muted">
              {{ t('archive.outputPathHint') }}
            </p>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('archive.revisionRangeLabel') }}
            </label>
            <UInput
              v-model="revisionRange"
              placeholder="--all or HEAD~5..HEAD or master"
              class="font-mono text-xs w-full"
            />
            <p class="text-[11px] text-muted">
              {{ t('archive.revisionRangeHint') }}
            </p>
          </div>
        </div>

        <!-- Verify Bundle Tab Content -->
        <div v-else-if="activeTab === 'verify'" class="space-y-4">
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('archive.verifyBundleLabel') }}
            </label>
            <div class="flex items-center gap-2">
              <UInput
                v-model="verifyFilePath"
                :placeholder="t('archive.verifyBundlePlaceholder')"
                class="font-mono text-xs flex-1"
              />
              <UButton
                color="primary"
                size="sm"
                :disabled="!verifyFilePath.trim()"
                @click="handleVerifyBundle"
              >
                {{ t('archive.verifyAction') }}
              </UButton>
            </div>
          </div>

          <!-- Verification Outcome -->
          <div
            v-if="bundleVerificationResult"
            class="p-4 rounded-xl border text-xs space-y-2"
            :class="bundleVerificationResult.is_valid ? 'border-success/30 bg-success/10 text-success' : 'border-error/30 bg-error/10 text-error'"
          >
            <div class="flex items-center gap-2 font-semibold">
              <UIcon :name="bundleVerificationResult.is_valid ? 'i-lucide-check-circle-2' : 'i-lucide-alert-triangle'" class="w-4 h-4" />
              <span>{{ bundleVerificationResult.message }}</span>
            </div>

            <div v-if="bundleVerificationResult.references.length > 0" class="pt-1 text-muted space-y-1">
              <span class="font-medium text-highlighted">{{ t('archive.bundleReferences') }}:</span>
              <ul class="list-disc ps-5 font-mono text-[11px] space-y-0.5">
                <li v-for="refName in bundleVerificationResult.references" :key="refName">
                  {{ refName }}
                </li>
              </ul>
            </div>
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
          v-if="activeTab === 'create'"
          color="primary"
          size="sm"
          :disabled="!outputBundlePath.trim() || isOperating"
          :loading="isOperating"
          @click="handleCreateBundle"
        >
          {{ t('archive.createBundleAction') }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
