<script setup lang="ts">
import type { PullRequestChangedFile } from '~/types/provider'

const props = defineProps<{
  accountIdentifier: string
  repositoryFullName: string
  pullRequestNumber: number
}>()

const { t } = useI18n()
const { fetchPullRequestFiles } = useProviderController()

const changedFiles = ref<PullRequestChangedFile[]>([])
const selectedFilePath = ref<string | null>(null)
const isLoadingFiles = ref<boolean>(false)
const searchQuery = ref<string>('')

interface ParsedDiffLine {
  type: 'hunk-header' | 'addition' | 'deletion' | 'context'
  content: string
  oldLineNumber?: number
  newLineNumber?: number
}

const filteredFiles = computed(() => {
  if (!searchQuery.value.trim()) return changedFiles.value
  const query = searchQuery.value.toLowerCase().trim()
  return changedFiles.value.filter((file) =>
    file.file_path.toLowerCase().includes(query),
  )
})

const selectedFile = computed(() => {
  if (!selectedFilePath.value) return changedFiles.value[0] || null
  return changedFiles.value.find((file) => file.file_path === selectedFilePath.value) || changedFiles.value[0] || null
})

const totalAdditions = computed(() => {
  return changedFiles.value.reduce((acc, file) => acc + file.additions_count, 0)
})

const totalDeletions = computed(() => {
  return changedFiles.value.reduce((acc, file) => acc + file.deletions_count, 0)
})

const parsedLines = computed<ParsedDiffLine[]>(() => {
  if (!selectedFile.value?.patch) return []

  const rawLines = selectedFile.value.patch.split('\n')
  const result: ParsedDiffLine[] = []

  let currentOldLine = 1
  let currentNewLine = 1

  for (const rawLine of rawLines) {
    if (rawLine.startsWith('@@')) {
      // Parse @@ -start,count +start,count @@
      const match = rawLine.match(/@@\s+-(\d+)(?:,\d+)?\s+\+(\d+)(?:,\d+)?\s+@@/)
      if (match && match[1] && match[2]) {
        currentOldLine = Number.parseInt(match[1], 10)
        currentNewLine = Number.parseInt(match[2], 10)
      }
      result.push({
        type: 'hunk-header',
        content: rawLine,
      })
    } else if (rawLine.startsWith('+')) {
      result.push({
        type: 'addition',
        content: rawLine.slice(1),
        newLineNumber: currentNewLine,
      })
      currentNewLine++
    } else if (rawLine.startsWith('-')) {
      result.push({
        type: 'deletion',
        content: rawLine.slice(1),
        oldLineNumber: currentOldLine,
      })
      currentOldLine++
    } else {
      const trimmed = rawLine.startsWith(' ') ? rawLine.slice(1) : rawLine
      result.push({
        type: 'context',
        content: trimmed,
        oldLineNumber: currentOldLine,
        newLineNumber: currentNewLine,
      })
      currentOldLine++
      currentNewLine++
    }
  }

  return result
})

async function loadFiles(): Promise<void> {
  if (!props.accountIdentifier || !props.repositoryFullName || !props.pullRequestNumber) return
  isLoadingFiles.value = true
  try {
    changedFiles.value = await fetchPullRequestFiles(
      props.accountIdentifier,
      props.repositoryFullName,
      props.pullRequestNumber,
    )
    if (changedFiles.value.length > 0 && !selectedFilePath.value) {
      selectedFilePath.value = changedFiles.value[0]?.file_path || null
    }
  } finally {
    isLoadingFiles.value = false
  }
}

watch(
  () => props.pullRequestNumber,
  () => {
    selectedFilePath.value = null
    void loadFiles()
  },
  { immediate: true },
)
</script>

<template>
  <div class="flex h-[520px] rounded-xl border border-default/70 bg-[var(--gv-surface)] overflow-hidden shadow-xs">
    <!-- Left: Files Tree/List -->
    <div class="w-1/3 border-e border-default/60 flex flex-col min-w-0 bg-muted/5">
      <!-- Header with stats and search -->
      <div class="p-2.5 border-b border-default/50 space-y-2">
        <div class="flex items-center justify-between text-xs font-semibold">
          <span class="text-highlighted">{{ changedFiles.length }} {{ t('diff.filesChanged') }}</span>
          <div class="flex items-center gap-2 font-mono text-[11px]">
            <span class="text-success font-bold">+{{ totalAdditions }}</span>
            <span class="text-error font-bold">-{{ totalDeletions }}</span>
          </div>
        </div>

        <UInput
          v-model="searchQuery"
          size="xs"
          icon="i-lucide-search"
          :placeholder="t('diff.filterFiles')"
          class="w-full"
        />
      </div>

      <!-- File list container -->
      <div class="flex-1 overflow-y-auto p-1.5 space-y-0.5">
        <!-- Zero-CLS Skeleton for file list -->
        <div v-if="isLoadingFiles" class="space-y-1.5 p-1 animate-pulse">
          <div
            v-for="i in 6"
            :key="`file-skeleton-${i}`"
            class="flex items-center justify-between p-1.5 rounded-lg bg-surface/30"
          >
            <div class="flex items-center gap-1.5">
              <USkeleton class="h-3.5 w-3.5 rounded" />
              <USkeleton class="h-3 w-32 rounded" />
            </div>
            <USkeleton class="h-3 w-10 rounded" />
          </div>
        </div>

        <div v-else-if="filteredFiles.length === 0" class="py-10 text-center text-muted text-xs">
          {{ t('diff.noFilesFound') }}
        </div>

        <template v-else>
          <div
            v-for="file in filteredFiles"
            :key="file.file_path"
            class="flex items-center justify-between p-2 rounded-lg cursor-pointer text-xs transition-colors group"
            :class="selectedFile?.file_path === file.file_path
              ? 'bg-primary/15 text-primary font-semibold shadow-xs'
              : 'hover:bg-muted/20 text-toned'"
            @click="selectedFilePath = file.file_path"
          >
            <div class="flex items-center gap-2 min-w-0">
              <UIcon
                :name="file.status === 'added' ? 'i-lucide-file-plus' :
                       file.status === 'deleted' ? 'i-lucide-file-minus' :
                       file.status === 'renamed' ? 'i-lucide-file-symlink' : 'i-lucide-file-edit'"
                class="h-3.5 w-3.5 shrink-0"
                :class="file.status === 'added' ? 'text-success' :
                        file.status === 'deleted' ? 'text-error' :
                        file.status === 'renamed' ? 'text-info' : 'text-warning'"
              />
              <span class="truncate font-mono text-[11px]" :title="file.file_path">
                {{ file.file_path }}
              </span>
            </div>

            <div class="flex items-center gap-1 font-mono text-[10px] shrink-0 ms-2">
              <span v-if="file.additions_count > 0" class="text-success font-bold">+{{ file.additions_count }}</span>
              <span v-if="file.deletions_count > 0" class="text-error font-bold">-{{ file.deletions_count }}</span>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Right: Diff Viewport -->
    <div class="flex-1 flex flex-col min-w-0 bg-[var(--gv-background)]">
      <!-- File Header Bar -->
      <div v-if="selectedFile" class="flex items-center justify-between border-b border-default/60 px-4 py-2 bg-muted/10 shrink-0">
        <div class="flex items-center gap-2 min-w-0">
          <span class="font-mono text-xs font-bold text-highlighted truncate">{{ selectedFile.file_path }}</span>
          <UBadge
            size="xs"
            :color="selectedFile.status === 'added' ? 'success' : selectedFile.status === 'deleted' ? 'error' : 'warning'"
            variant="subtle"
            class="capitalize text-[10px] py-0"
          >
            {{ selectedFile.status }}
          </UBadge>
        </div>

        <div class="flex items-center gap-2 font-mono text-xs shrink-0">
          <span class="text-success font-semibold">+{{ selectedFile.additions_count }}</span>
          <span class="text-error font-semibold">-{{ selectedFile.deletions_count }}</span>
        </div>
      </div>

      <!-- Diff lines container -->
      <div class="flex-1 overflow-auto p-0 font-mono text-[11px] leading-relaxed">
        <div v-if="isLoadingFiles" class="p-4 space-y-2 animate-pulse">
          <USkeleton class="h-4 w-48 rounded mb-4" />
          <div v-for="index in 12" :key="`line-skeleton-${index}`" class="flex items-center gap-3">
            <USkeleton class="h-3 w-8 rounded" />
            <USkeleton class="h-3 rounded" :class="index % 3 === 0 ? 'w-2/3' : index % 2 === 0 ? 'w-4/5' : 'w-1/2'" />
          </div>
        </div>

        <div v-else-if="!selectedFile" class="flex flex-col items-center justify-center h-full text-muted space-y-1">
          <UIcon name="i-lucide-file-code" class="h-8 w-8 opacity-40" />
          <p>{{ t('diff.noFileSelected') }}</p>
        </div>

        <div v-else-if="!selectedFile.patch" class="flex flex-col items-center justify-center h-full text-muted space-y-1">
          <UIcon name="i-lucide-binary" class="h-8 w-8 opacity-40" />
          <p>{{ t('diff.binaryOrLargeFileDiff') }}</p>
        </div>

        <div v-else class="divide-y divide-default/20">
          <div
            v-for="(line, lineIndex) in parsedLines"
            :key="`diff-line-${lineIndex}`"
            class="flex items-start transition-colors px-1 py-0.5"
            :class="[
              line.type === 'hunk-header' ? 'bg-[var(--gv-diff-hunk-bg)]/40 text-[var(--gv-diff-hunk-text)] font-semibold border-y border-default/30 py-1' :
              line.type === 'addition' ? 'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)]' :
              line.type === 'deletion' ? 'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)]' :
              'hover:bg-muted/10 text-toned'
            ]"
          >
            <!-- Old Line Number -->
            <div
              v-if="line.type !== 'hunk-header'"
              class="w-10 pe-2 text-end text-[10px] text-muted/60 select-none shrink-0"
            >
              {{ line.oldLineNumber || '' }}
            </div>

            <!-- New Line Number -->
            <div
              v-if="line.type !== 'hunk-header'"
              class="w-10 pe-2 text-end text-[10px] text-muted/60 select-none shrink-0"
            >
              {{ line.newLineNumber || '' }}
            </div>

            <!-- Gutter Prefix -->
            <div
              v-if="line.type !== 'hunk-header'"
              class="w-4 text-center font-bold select-none shrink-0"
              :class="line.type === 'addition' ? 'text-success' : line.type === 'deletion' ? 'text-error' : 'text-muted'"
            >
              {{ line.type === 'addition' ? '+' : line.type === 'deletion' ? '-' : ' ' }}
            </div>

            <!-- Content -->
            <div class="flex-1 whitespace-pre-wrap break-all px-1">
              {{ line.content }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
