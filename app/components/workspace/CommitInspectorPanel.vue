<script setup lang="ts">
import type { CommitDetails } from '~/types/repository'
import CommitDetailsHeader from './commit/CommitDetailsHeader.vue'
import CommitChangedFileList from './commit/CommitChangedFileList.vue'

const props = defineProps<{
  commitDetails: CommitDetails | null
  selectedFilePath: string | null
  isLoading: boolean
}>()

const emit = defineEmits<{
  selectFile: [filePath: string]
  createBranch: [commitSha: string]
  checkoutCommit: [commitSha: string]
  cherryPick: [commitSha: string]
  revert: [commitSha: string]
  reset: [commitSha: string, mode: 'soft' | 'mixed' | 'hard']
  close: []
}>()

const { t } = useI18n()
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden">
    <!-- Zero-CLS Skeleton Loading -->
    <div v-if="props.isLoading" class="p-4 space-y-3">
      <div class="flex items-center gap-2">
        <USkeleton class="h-4 w-24 rounded" />
        <USkeleton class="h-4 w-16 rounded" />
      </div>
      <USkeleton class="h-6 w-3/4 rounded" />
      <USkeleton class="h-3 w-1/2 rounded" />
      <div class="space-y-1.5 pt-4">
        <USkeleton v-for="i in 5" :key="`skeleton-file-${i}`" class="h-5 w-full rounded" />
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="!props.commitDetails"
      class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted"
    >
      <UIcon name="i-lucide-git-commit" class="h-8 w-8 text-muted/60" />
      <span class="text-xs">{{ t('commit.noCommitSelected') }}</span>
    </div>

    <!-- Loaded Commit Content -->
    <template v-else>
      <CommitDetailsHeader
        :commit-details="props.commitDetails"
        @create-branch="emit('createBranch', $event)"
        @checkout-commit="emit('checkoutCommit', $event)"
        @cherry-pick="emit('cherryPick', $event)"
        @revert="emit('revert', $event)"
        @reset="(hash, mode) => emit('reset', hash, mode)"
        @close="emit('close')"
      />

      <CommitChangedFileList
        :files="props.commitDetails.changed_files"
        :selected-file-path="props.selectedFilePath"
        @select-file="emit('selectFile', $event)"
      />
    </template>
  </div>
</template>
