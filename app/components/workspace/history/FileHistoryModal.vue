<script setup lang="ts">
import { formatStashRelativeTime } from '~/utils/stash'

interface FileHistoryModalProps {
  open: boolean
  initialFilePath?: string
}

const props = defineProps<FileHistoryModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  locateCommit: [commitHash: string]
}>()

const { t } = useI18n()
const { notifySuccess } = useNotificationService()
const {
  timeline,
  isLoadingTimeline,
  fetchFileHistory,
  fetchFileContentAtCommit,
} = useFileHistoryController()

const targetFilePath = ref<string>(props.initialFilePath || '')
const selectedRevisionIndex = ref<number>(0)
const currentContent = ref<string>('')
const isLoadingContent = ref<boolean>(false)

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      if (props.initialFilePath) {
        targetFilePath.value = props.initialFilePath
      }
      if (targetFilePath.value.trim()) {
        await loadHistory()
      }
    }
  },
  { immediate: true }
)

watch(
  () => props.initialFilePath,
  (newPath) => {
    if (newPath) {
      targetFilePath.value = newPath
    }
  }
)

watch(
  selectedRevisionIndex,
  async (newIndex) => {
    const revision = timeline.value?.revisions[newIndex]
    if (revision && timeline.value?.file_path) {
      isLoadingContent.value = true
      currentContent.value = await fetchFileContentAtCommit(
        revision.commit_hash,
        timeline.value.file_path
      )
      isLoadingContent.value = false
    }
  }
)

async function loadHistory() {
  if (!targetFilePath.value.trim()) return
  selectedRevisionIndex.value = 0
  const result = await fetchFileHistory(targetFilePath.value.trim())
  const firstRevision = result?.revisions[0]
  if (result && firstRevision) {
    isLoadingContent.value = true
    currentContent.value = await fetchFileContentAtCommit(
      firstRevision.commit_hash,
      result.file_path
    )
    isLoadingContent.value = false
  } else {
    currentContent.value = ''
  }
}

const currentRevision = computed(() => {
  return timeline.value?.revisions[selectedRevisionIndex.value] || null
})

function handleCopyHash(commitHash: string) {
  navigator.clipboard.writeText(commitHash)
  notifySuccess(t('patch.notifications.copied'))
}

function handleCopyContent() {
  if (!currentContent.value) return
  navigator.clipboard.writeText(currentContent.value)
  notifySuccess(t('fileHistory.notifications.copiedContent'))
}

function handleLocateCommit(commitHash: string) {
  emit('locateCommit', commitHash)
  emit('update:open', false)
}

function getStatusColor(status: string): 'success' | 'primary' | 'warning' | 'error' | 'neutral' {
  if (status.startsWith('A')) return 'success'
  if (status.startsWith('M')) return 'primary'
  if (status.startsWith('R')) return 'warning'
  if (status.startsWith('D')) return 'error'
  return 'neutral'
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl max-w-4xl overflow-hidden focus:outline-none',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Header -->
      <div class="px-6 py-4 border-b border-default/60 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
            <UIcon name="i-lucide-history" class="w-5 h-5 text-primary" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('fileHistory.modalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('fileHistory.modalSubtitle') }}
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

      <!-- File Path Bar -->
      <div class="px-6 py-3 bg-neutral-500/5 border-b border-default/40 flex items-center gap-2">
        <UIcon name="i-lucide-file-code" class="w-4 h-4 text-muted shrink-0" />
        <UInput
          v-model="targetFilePath"
          :placeholder="t('fileHistory.filePathPlaceholder')"
          class="w-full text-xs font-mono"
          @keydown.enter="loadHistory"
        />
        <UButton
          size="xs"
          color="primary"
          variant="subtle"
          icon="i-lucide-search"
          :label="t('common.search')"
          :loading="isLoadingTimeline"
          @click="loadHistory"
        />
      </div>

      <!-- Zero-CLS Skeleton Loading -->
      <div v-if="isLoadingTimeline" class="p-6 space-y-4">
        <USkeleton class="h-16 w-full rounded-xl" />
        <USkeleton class="h-64 w-full rounded-xl" />
      </div>

      <!-- Content View -->
      <div v-else-if="timeline && timeline.revisions.length > 0" class="p-6 space-y-4 max-h-[72vh] overflow-y-auto">
        <!-- Revision Timeline Slider & Scrubber -->
        <div class="p-4 rounded-xl border border-default/60 bg-surface space-y-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold text-highlighted">
                {{ t('fileHistory.scrubberTitle') }}
              </span>
              <UBadge color="primary" variant="subtle" size="xs">
                {{ selectedRevisionIndex + 1 }} / {{ timeline.total_revisions }}
              </UBadge>
            </div>
            <div class="flex items-center gap-1.5">
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-chevron-right"
                :disabled="selectedRevisionIndex >= timeline.revisions.length - 1"
                @click="selectedRevisionIndex++"
              />
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-chevron-left"
                :disabled="selectedRevisionIndex <= 0"
                @click="selectedRevisionIndex--"
              />
            </div>
          </div>

          <!-- Slider -->
          <input
            v-model.number="selectedRevisionIndex"
            type="range"
            min="0"
            :max="timeline.revisions.length - 1"
            class="w-full accent-primary cursor-pointer h-2 bg-neutral-500/20 rounded-lg"
          >

          <!-- Selected Revision Card -->
          <div v-if="currentRevision" class="p-3 bg-neutral-500/5 rounded-lg border border-default/50 space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <UBadge :color="getStatusColor(currentRevision.change_status)" variant="subtle" size="xs">
                  {{ currentRevision.change_status }}
                </UBadge>
                <span class="font-mono text-xs font-semibold text-highlighted">
                  {{ currentRevision.commit_hash.slice(0, 10) }}
                </span>
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-copy"
                  class="h-5 w-5 p-0"
                  @click="handleCopyHash(currentRevision.commit_hash)"
                />
              </div>

              <div class="flex items-center gap-2 text-xs text-muted">
                <span>{{ currentRevision.author_name }}</span>
                <span>•</span>
                <span>{{ formatStashRelativeTime(currentRevision.author_timestamp) }}</span>
                <UButton
                  size="xs"
                  color="primary"
                  variant="ghost"
                  icon="i-lucide-locate"
                  :label="t('fileHistory.locate')"
                  @click="handleLocateCommit(currentRevision.commit_hash)"
                />
              </div>
            </div>

            <p class="text-xs text-highlighted font-medium">
              {{ currentRevision.commit_summary }}
            </p>
          </div>
        </div>

        <!-- Historical File Content -->
        <div class="rounded-xl border border-default/60 bg-surface overflow-hidden space-y-2 p-4">
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium text-highlighted">
              {{ t('fileHistory.contentAtRevision') }}
            </span>
            <UButton
              size="xs"
              color="neutral"
              variant="subtle"
              icon="i-lucide-copy"
              :label="t('fileHistory.copyContent')"
              :disabled="!currentContent"
              @click="handleCopyContent"
            />
          </div>

          <div v-if="isLoadingContent" class="py-8 text-center">
            <USkeleton class="h-32 w-full rounded-lg" />
          </div>

          <pre
            v-else-if="currentContent"
            class="p-4 bg-neutral-900 text-neutral-200 rounded-xl text-xs font-mono max-h-72 overflow-y-auto whitespace-pre-wrap selection:bg-primary/30"
          >{{ currentContent }}</pre>

          <div v-else class="p-6 text-center text-xs text-muted">
            {{ t('fileHistory.emptyContent') }}
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="p-12 text-center space-y-2">
        <UIcon name="i-lucide-file-question" class="w-10 h-10 mx-auto text-muted" />
        <h3 class="text-sm font-medium text-highlighted">
          {{ t('fileHistory.noHistoryTitle') }}
        </h3>
        <p class="text-xs text-muted max-w-sm mx-auto">
          {{ t('fileHistory.noHistorySubtitle') }}
        </p>
      </div>

      <!-- Footer -->
      <div class="px-6 py-3.5 border-t border-default/60 flex items-center justify-end">
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          :label="t('common.close')"
          @click="emit('update:open', false)"
        />
      </div>
    </template>
  </UModal>
</template>
