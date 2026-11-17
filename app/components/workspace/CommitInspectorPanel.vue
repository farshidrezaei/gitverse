<script setup lang="ts">
import type { CommitDetails } from '~/types/repository'

const props = defineProps<{
  commitDetails: CommitDetails | null
  selectedFilePath: string | null
  isLoading: boolean
}>()

const emit = defineEmits<{
  selectFile: [filePath: string]
  createBranch: [commitSha: string]
  checkoutCommit: [commitSha: string]
  close: []
}>()

const { t } = useI18n()
const filterQuery = ref('')
const isCopied = ref(false)

const filteredFiles = computed(() => {
  if (!props.commitDetails?.changed_files) return []
  const query = filterQuery.value.trim().toLowerCase()
  if (!query) return props.commitDetails.changed_files
  return props.commitDetails.changed_files.filter((file) =>
    file.path.toLowerCase().includes(query) ||
    file.original_path?.toLowerCase().includes(query)
  )
})

async function copyCommitSha(): Promise<void> {
  if (!props.commitDetails?.hash) return
  try {
    await navigator.clipboard.writeText(props.commitDetails.hash)
    isCopied.value = true
    setTimeout(() => {
      isCopied.value = false
    }, 2000)
  } catch (error) {
    console.error('Failed to copy commit SHA to clipboard:', error)
  }
}

function getStatusBadge(status: string): { label: string; colorClass: string } {
  switch (status) {
    case 'modified':
      return { label: 'M', colorClass: 'text-warning' }
    case 'added':
      return { label: 'A', colorClass: 'text-success' }
    case 'deleted':
      return { label: 'D', colorClass: 'text-error' }
    case 'renamed':
      return { label: 'R', colorClass: 'text-primary' }
    case 'copied':
      return { label: 'C', colorClass: 'text-info' }
    case 'untracked':
      return { label: '?', colorClass: 'text-muted' }
    case 'conflicted':
      return { label: '!', colorClass: 'text-error font-extrabold' }
    default:
      return { label: status[0]?.toUpperCase() || 'M', colorClass: 'text-toned' }
  }
}

function formatRelativeTime(timestampInSeconds: number): string {
  const diffMilliseconds = Date.now() - timestampInSeconds * 1000
  const diffSeconds = Math.max(0, Math.floor(diffMilliseconds / 1000))
  if (diffSeconds < 60) return `${diffSeconds}s ago`
  const diffMinutes = Math.floor(diffSeconds / 60)
  if (diffMinutes < 60) return `${diffMinutes}m ago`
  const diffHours = Math.floor(diffMinutes / 60)
  if (diffHours < 24) return `${diffHours}h ago`
  const diffDays = Math.floor(diffHours / 24)
  if (diffDays < 30) return `${diffDays}d ago`
  return new Date(timestampInSeconds * 1000).toLocaleDateString()
}
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden">
    <!-- Panel Header: Back to Working Tree & Commit Actions -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
      <div class="flex items-center gap-1.5 min-w-0">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-arrow-left"
          :label="t('commit.backToWorkingTree')"
          class="font-medium text-toned hover:text-highlighted"
          @click="emit('close')"
        />
        <div class="h-3.5 w-px bg-default/80" aria-hidden="true" />
        <span class="text-xs font-semibold text-highlighted uppercase tracking-wider truncate">
          {{ t('commit.details') }}
        </span>
      </div>

      <div v-if="commitDetails" class="flex items-center gap-1">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-git-branch-plus"
          :title="t('commit.createBranchHere')"
          @click="emit('createBranch', commitDetails.hash)"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-log-in"
          :title="t('commit.checkoutCommit')"
          @click="emit('checkoutCommit', commitDetails.hash)"
        />
      </div>
    </div>

    <!-- Content Area: Zero-CLS Skeleton Loading State -->
    <div v-if="isLoading" class="flex-1 overflow-y-auto p-3 space-y-4">
      <!-- Skeleton Header Card -->
      <div class="rounded-lg border border-default/70 p-3 bg-surface/80 space-y-2.5">
        <USkeleton class="h-4 w-3/4" />
        <div class="flex items-center gap-2">
          <USkeleton class="h-6 w-6 rounded-full" />
          <USkeleton class="h-3 w-32" />
          <USkeleton class="h-3 w-16" />
        </div>
        <div class="flex items-center gap-2 pt-1">
          <USkeleton class="h-5 w-24 rounded" />
          <USkeleton class="h-5 w-16 rounded" />
        </div>
      </div>

      <!-- Skeleton Files List -->
      <div class="space-y-2">
        <div class="flex items-center justify-between px-1">
          <USkeleton class="h-3.5 w-24" />
          <USkeleton class="h-3.5 w-12" />
        </div>
        <div class="rounded-lg border border-default/70 p-2 space-y-2 bg-surface/80">
          <div v-for="index in 4" :key="index" class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <USkeleton class="h-3 w-3" />
              <USkeleton class="h-3 w-40" />
            </div>
            <USkeleton class="h-3 w-10" />
          </div>
        </div>
      </div>
    </div>

    <!-- Content Area: Loaded Commit Details -->
    <div v-else-if="commitDetails" class="flex-1 overflow-y-auto p-3 space-y-3.5 text-xs">
      <!-- Commit Metadata Card -->
      <div class="rounded-lg border border-default/70 p-3 bg-surface/80 space-y-2.5 shadow-2xs">
        <!-- Commit Subject -->
        <h3 class="text-xs font-semibold text-highlighted leading-relaxed select-text">
          {{ commitDetails.subject }}
        </h3>

        <!-- Author & Date Row -->
        <div class="flex items-center justify-between text-[11px] text-muted">
          <div class="flex items-center gap-1.5 min-w-0">
            <div class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-primary/20 text-primary font-bold text-[10px]">
              {{ commitDetails.author_name.charAt(0).toUpperCase() }}
            </div>
            <span class="truncate font-medium text-toned" :title="commitDetails.author_email">
              {{ commitDetails.author_name }}
            </span>
          </div>
          <span class="shrink-0" :title="new Date(commitDetails.author_timestamp * 1000).toLocaleString()">
            {{ formatRelativeTime(commitDetails.author_timestamp) }}
          </span>
        </div>

        <!-- Commit Body (if present) -->
        <div
          v-if="commitDetails.body"
          class="rounded border border-default/50 bg-muted/20 p-2 font-mono text-[11px] text-toned whitespace-pre-wrap select-text leading-normal max-h-36 overflow-y-auto"
        >
          {{ commitDetails.body }}
        </div>

        <!-- SHA & Stats Row -->
        <div class="flex flex-wrap items-center justify-between gap-1.5 pt-1 border-t border-default/60">
          <!-- Commit Hash with Copy -->
          <button
            type="button"
            class="flex items-center gap-1 rounded bg-muted/30 px-1.5 py-0.5 font-mono text-[10px] text-toned hover:bg-muted/50 hover:text-highlighted transition-colors cursor-pointer"
            :title="t('commit.copySha')"
            @click="copyCommitSha"
          >
            <UIcon :name="isCopied ? 'i-lucide-check' : 'i-lucide-copy'" class="h-3 w-3" :class="isCopied ? 'text-success' : 'text-muted'" />
            <span>{{ commitDetails.hash.substring(0, 8) }}</span>
            <span v-if="isCopied" class="text-success font-sans text-[10px]">{{ t('commit.copied') }}</span>
          </button>

          <!-- Additions / Deletions Counters -->
          <div class="flex items-center gap-2 font-mono text-[10px]">
            <span v-if="commitDetails.total_additions > 0" class="text-success font-semibold">
              +{{ commitDetails.total_additions }}
            </span>
            <span v-if="commitDetails.total_deletions > 0" class="text-error font-semibold">
              -{{ commitDetails.total_deletions }}
            </span>
            <span class="text-muted">
              {{ commitDetails.changed_files.length }} {{ t('git.commits').toLowerCase() === 'commits' ? 'files' : 'فایل' }}
            </span>
          </div>
        </div>
      </div>

      <!-- Changed Files Section -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="font-semibold text-highlighted flex items-center gap-1.5 text-[11px]">
            <UIcon name="i-lucide-files" class="h-3.5 w-3.5 text-primary" />
            {{ t('commit.filesChanged') }}
            <span class="text-muted">({{ commitDetails.changed_files.length }})</span>
          </span>

          <!-- Quick Filter Input -->
          <div v-if="commitDetails.changed_files.length > 4" class="w-36">
            <UInput
              v-model="filterQuery"
              size="xs"
              icon="i-lucide-search"
              :placeholder="t('commit.searchFiles')"
              class="text-[11px]"
            />
          </div>
        </div>

        <!-- Files List -->
        <div
          v-if="filteredFiles.length > 0"
          class="rounded-lg border border-default/70 divide-y divide-default bg-surface/80 overflow-hidden"
        >
          <div
            v-for="file in filteredFiles"
            :key="file.path"
            class="group flex items-center justify-between px-2.5 py-1.5 cursor-pointer transition-colors"
            :class="selectedFilePath === file.path
              ? 'bg-primary/15 text-primary font-medium'
              : 'hover:bg-muted/30'"
            @click="emit('selectFile', file.path)"
          >
            <div class="flex items-center gap-2 min-w-0">
              <span
                class="font-mono font-bold text-[11px]"
                :class="getStatusBadge(file.status).colorClass"
              >
                {{ getStatusBadge(file.status).label }}
              </span>
              <span class="truncate font-mono text-[11px]" dir="ltr">
                {{ file.path }}
                <span v-if="file.original_path" class="text-muted text-[10px]">← {{ file.original_path }}</span>
              </span>
            </div>

            <!-- Additions / Deletions Badges -->
            <div class="flex items-center gap-1 font-mono text-[10px] shrink-0 ms-2">
              <span v-if="file.additions > 0" class="text-success">+{{ file.additions }}</span>
              <span v-if="file.deletions > 0" class="text-error">-{{ file.deletions }}</span>
            </div>
          </div>
        </div>

        <div v-else class="text-[11px] text-muted italic px-1 py-2 text-center">
          {{ t('commit.noFilesChanged') }}
        </div>
      </div>
    </div>
  </div>
</template>
