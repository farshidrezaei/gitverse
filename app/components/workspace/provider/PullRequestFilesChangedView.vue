<script setup lang="ts">
import type { PullRequestChangedFile } from '~/types/provider'
import { buildFileTree, type FileTreeNode } from '~/utils/fileTree'
import PullRequestFileTreeNode from './PullRequestFileTreeNode.vue'
import MarkdownCommentEditor from './MarkdownCommentEditor.vue'

const props = defineProps<{
  accountIdentifier: string
  repositoryFullName: string
  pullRequestNumber: number
}>()

const { t } = useI18n()
const { fetchPullRequestFiles, createPullRequestComment } = useProviderController()

const changedFiles = ref<PullRequestChangedFile[]>([])
const selectedFilePath = ref<string | null>(null)
const isLoadingFiles = ref<boolean>(false)
const searchQuery = ref<string>('')
const fileViewLayout = ref<'flat' | 'tree'>('flat')
const diffViewMode = ref<'unified' | 'split'>('unified')
const isWordWrap = ref<boolean>(false)

// Inline comment & line selection state
const isSelectingRange = ref<boolean>(false)
const selectionStartLine = ref<number | null>(null)
const selectionEndLine = ref<number | null>(null)
const activeCommentLine = ref<number | null>(null)
const isSubmittingInlineComment = ref<boolean>(false)

interface ParsedDiffLine {
  type: 'hunk-header' | 'addition' | 'deletion' | 'context'
  content: string
  oldLineNumber?: number
  newLineNumber?: number
}

interface SplitDiffRow {
  left?: ParsedDiffLine
  right?: ParsedDiffLine
}

const filteredFiles = computed(() => {
  if (!searchQuery.value.trim()) return changedFiles.value
  const query = searchQuery.value.toLowerCase().trim()
  return changedFiles.value.filter((file) =>
    file.file_path.toLowerCase().includes(query),
  )
})

const fileTreeNodes = computed<FileTreeNode<{ path: string; file: PullRequestChangedFile }>[]>(() => {
  const treeSource = filteredFiles.value.map((file) => ({
    path: file.file_path,
    file,
  }))
  return buildFileTree(treeSource)
})

const selectedFile = computed(() => {
  if (!selectedFilePath.value) return changedFiles.value[0] || null
  return changedFiles.value.find((file) => file.file_path === selectedFilePath.value) || changedFiles.value[0] || null
})

const fileExtension = computed<string>(() => {
  const path = selectedFile.value?.file_path || ''
  const parts = path.split('.')
  return parts.length > 1 ? (parts.pop() || '').toLowerCase() : ''
})

const fileIconName = computed<string>(() => {
  if (['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(fileExtension.value)) {
    return 'i-lucide-image'
  }
  if (['mp3', 'wav', 'ogg', 'aac', 'flac', 'm4a'].includes(fileExtension.value)) {
    return 'i-lucide-music'
  }
  if (['mp4', 'webm', 'mov', 'avi', 'mkv'].includes(fileExtension.value)) {
    return 'i-lucide-video'
  }
  return 'i-lucide-file-diff'
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

const splitRows = computed<SplitDiffRow[]>(() => {
  const rows: SplitDiffRow[] = []
  const lines = parsedLines.value

  let index = 0
  while (index < lines.length) {
    const current = lines[index]
    if (!current) break

    if (current.type === 'hunk-header') {
      rows.push({ left: current, right: current })
      index++
      continue
    }

    if (current.type === 'deletion') {
      const next = lines[index + 1]
      if (next && next.type === 'addition') {
        // Paired change (modified line)
        rows.push({ left: current, right: next })
        index += 2
        continue
      }
      rows.push({ left: current, right: undefined })
      index++
      continue
    }

    if (current.type === 'addition') {
      rows.push({ left: undefined, right: current })
      index++
      continue
    }

    // Context line (identical on both sides)
    rows.push({ left: current, right: current })
    index++
  }

  return rows
})

// Line range selection for review comments
function startLineSelection(line: ParsedDiffLine): void {
  const lineNum = line.newLineNumber || line.oldLineNumber
  if (!lineNum) return
  selectionStartLine.value = lineNum
  selectionEndLine.value = lineNum
  isSelectingRange.value = true
}

function handleLineHover(line: ParsedDiffLine): void {
  if (!isSelectingRange.value) return
  const lineNum = line.newLineNumber || line.oldLineNumber
  if (lineNum) {
    selectionEndLine.value = lineNum
  }
}

function endLineSelection(): void {
  if (!isSelectingRange.value) return
  isSelectingRange.value = false
  if (selectionStartLine.value && selectionEndLine.value) {
    activeCommentLine.value = Math.max(selectionStartLine.value, selectionEndLine.value)
  }
}

function isLineInSelection(line: ParsedDiffLine): boolean {
  const lineNum = line.newLineNumber || line.oldLineNumber
  if (!lineNum || selectionStartLine.value === null || selectionEndLine.value === null) return false
  const min = Math.min(selectionStartLine.value, selectionEndLine.value)
  const max = Math.max(selectionStartLine.value, selectionEndLine.value)
  return lineNum >= min && lineNum <= max
}

function handleRowHover(row: SplitDiffRow): void {
  if (!isSelectingRange.value) return
  const target = row.right || row.left
  if (target) {
    handleLineHover(target)
  }
}

function isRowInSelection(row: SplitDiffRow): boolean {
  if (row.right && isLineInSelection(row.right)) return true
  if (row.left && isLineInSelection(row.left)) return true
  return false
}

function isCommentActiveOnRow(row: SplitDiffRow): boolean {
  if (activeCommentLine.value === null) return false
  if (row.right?.newLineNumber === activeCommentLine.value) return true
  if (row.left?.oldLineNumber === activeCommentLine.value) return true
  return false
}

function cancelComment(): void {
  activeCommentLine.value = null
  selectionStartLine.value = null
  selectionEndLine.value = null
  isSelectingRange.value = false
}

async function submitInlineComment(commentBody: string): Promise<void> {
  if (!commentBody.trim() || !selectedFile.value) return
  isSubmittingInlineComment.value = true

  const startLine = selectionStartLine.value && selectionEndLine.value
    ? Math.min(selectionStartLine.value, selectionEndLine.value)
    : activeCommentLine.value

  const endLine = selectionStartLine.value && selectionEndLine.value
    ? Math.max(selectionStartLine.value, selectionEndLine.value)
    : activeCommentLine.value

  const lineReferencePrefix = startLine === endLine
    ? `> **Comment on \`${selectedFile.value.file_path}\` line ${startLine}**\n\n`
    : `> **Comment on \`${selectedFile.value.file_path}\` lines ${startLine}-${endLine}**\n\n`

  try {
    await createPullRequestComment(
      props.accountIdentifier,
      props.repositoryFullName,
      props.pullRequestNumber,
      `${lineReferencePrefix}${commentBody}`,
    )
    cancelComment()
  } finally {
    isSubmittingInlineComment.value = false
  }
}

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
    cancelComment()
    void loadFiles()
  },
  { immediate: true },
)
</script>

<template>
  <div class="flex h-full min-h-[520px] rounded-xl border border-default/70 bg-[var(--gv-surface)] overflow-hidden shadow-xs">
    <!-- Left: Files Tree/List Navigation -->
    <div class="w-1/3 min-w-[260px] max-w-[360px] border-e border-default/60 flex flex-col min-w-0 bg-muted/5">
      <!-- Header with stats, tree/list toggle and search -->
      <div class="p-2.5 border-b border-default/50 space-y-2">
        <div class="flex items-center justify-between text-xs font-semibold">
          <span class="text-highlighted">{{ changedFiles.length }} {{ t('diff.filesChanged') }}</span>
          <div class="flex items-center gap-2 font-mono text-[11px]">
            <span class="text-success font-bold">+{{ totalAdditions }}</span>
            <span class="text-error font-bold">-{{ totalDeletions }}</span>
          </div>
        </div>

        <div class="flex items-center gap-1.5">
          <UInput
            v-model="searchQuery"
            size="xs"
            icon="i-lucide-search"
            :placeholder="t('diff.filterFiles')"
            class="flex-1"
          />

          <!-- View Layout: Flat vs Tree -->
          <div class="flex items-center border border-default/60 rounded-md p-0.5 bg-[var(--gv-surface)]">
            <button
              type="button"
              class="p-1 rounded text-xs transition-colors"
              :class="fileViewLayout === 'flat' ? 'bg-primary/20 text-primary font-bold' : 'text-muted hover:text-default'"
              :title="t('diff.flatList')"
              @click="fileViewLayout = 'flat'"
            >
              <UIcon name="i-lucide-list" class="h-3.5 w-3.5" />
            </button>
            <button
              type="button"
              class="p-1 rounded text-xs transition-colors"
              :class="fileViewLayout === 'tree' ? 'bg-primary/20 text-primary font-bold' : 'text-muted hover:text-default'"
              :title="t('diff.treeView')"
              @click="fileViewLayout = 'tree'"
            >
              <UIcon name="i-lucide-folder-tree" class="h-3.5 w-3.5" />
            </button>
          </div>
        </div>
      </div>

      <!-- File list or tree container -->
      <div class="flex-1 overflow-y-auto p-1.5 space-y-0.5">
        <!-- Zero-CLS Skeleton for file list -->
        <div v-if="isLoadingFiles" class="space-y-1.5 p-1 animate-pulse">
          <div
            v-for="i in 8"
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

        <!-- Tree View Mode -->
        <template v-else-if="fileViewLayout === 'tree'">
          <PullRequestFileTreeNode
            :nodes="fileTreeNodes"
            :selected-file-path="selectedFilePath"
            @select-file="selectedFilePath = $event"
          />
        </template>

        <!-- Flat List Mode -->
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
    <div
      class="flex-1 flex flex-col min-w-0 bg-[var(--gv-background)] select-text"
      @mouseup="endLineSelection"
    >
      <!-- File Header Bar & Mode Controls (Matches DiffHeaderToolbar standard) -->
      <div v-if="selectedFile" class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3 gap-2">
        <div class="flex items-center gap-2 min-w-0 flex-1 overflow-hidden">
          <UIcon :name="fileIconName" class="h-4 w-4 shrink-0 text-primary" />
          <span
            class="font-mono font-semibold text-highlighted text-[11px] truncate shrink min-w-0"
            :title="selectedFile.file_path"
          >
            {{ selectedFile.file_path }}
          </span>

          <UBadge
            size="xs"
            :color="selectedFile.status === 'added' ? 'success' : selectedFile.status === 'deleted' ? 'error' : 'warning'"
            variant="subtle"
            class="capitalize text-[10px] py-0 shrink-0"
          >
            {{ selectedFile.status }}
          </UBadge>

          <div class="flex items-center gap-1.5 font-mono text-xs shrink-0 ms-2">
            <span class="text-success font-semibold">+{{ selectedFile.additions_count }}</span>
            <span class="text-error font-semibold">-{{ selectedFile.deletions_count }}</span>
          </div>
        </div>

        <div class="flex items-center gap-1.5 shrink-0">
          <!-- Text Diff Controls (Inline vs Side-by-Side & Word Wrap) -->
          <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs shrink-0">
            <button
              type="button"
              class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all whitespace-nowrap shrink-0 cursor-pointer"
              :class="diffViewMode === 'unified'
                ? 'bg-primary text-inverted shadow-xs'
                : 'text-muted hover:text-highlighted'"
              :title="t('diff.viewInline')"
              @click="diffViewMode = 'unified'"
            >
              <UIcon name="i-lucide-rows-2" class="h-3 w-3 shrink-0" />
              <span class="hidden md:inline">{{ t('diff.viewInline') }}</span>
            </button>

            <button
              type="button"
              class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all whitespace-nowrap shrink-0 cursor-pointer"
              :class="diffViewMode === 'split'
                ? 'bg-primary text-inverted shadow-xs'
                : 'text-muted hover:text-highlighted'"
              :title="t('diff.viewSideBySide')"
              @click="diffViewMode = 'split'"
            >
              <UIcon name="i-lucide-columns-2" class="h-3 w-3 shrink-0" />
              <span class="hidden md:inline">{{ t('diff.viewSideBySide') }}</span>
            </button>
          </div>

          <!-- Word Wrap Toggle Button -->
          <UTooltip :text="t('diff.wordWrap')">
            <UButton
              size="xs"
              :color="isWordWrap ? 'primary' : 'neutral'"
              :variant="isWordWrap ? 'subtle' : 'ghost'"
              square
              icon="i-lucide-wrap-text"
              class="h-6 w-6 flex items-center justify-center p-0"
              :aria-label="t('diff.wordWrap')"
              @click="isWordWrap = !isWordWrap"
            />
          </UTooltip>
        </div>
      </div>

      <!-- Diff lines container -->
      <div class="flex-1 overflow-auto p-0 font-mono text-[11px] leading-relaxed">
        <div v-if="isLoadingFiles" class="p-4 space-y-2 animate-pulse">
          <USkeleton class="h-4 w-48 rounded mb-4" />
          <div v-for="index in 14" :key="`line-skeleton-${index}`" class="flex items-center gap-3">
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

        <!-- 1. Unified Diff View -->
        <div v-else-if="diffViewMode === 'unified'" class="divide-y divide-default/15">
          <template v-for="(line, lineIndex) in parsedLines" :key="`unified-line-${lineIndex}`">
            <div
              class="flex items-start transition-colors px-1 py-0.5 group relative"
              :class="[
                line.type === 'hunk-header' ? 'bg-[var(--gv-diff-hunk-bg)]/40 text-[var(--gv-diff-hunk-text)] font-semibold border-y border-default/30 py-1' :
                isLineInSelection(line) ? 'bg-[var(--gv-diff-modify-bg)]/80 text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]' :
                line.type === 'addition' ? 'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)]' :
                line.type === 'deletion' ? 'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)]' :
                'hover:bg-muted/10 text-toned'
              ]"
              @mouseenter="handleLineHover(line)"
            >
              <!-- Old Line Number -->
              <div
                v-if="line.type !== 'hunk-header'"
                class="w-10 pe-2 text-end text-[10px] text-muted/60 select-none shrink-0"
              >
                {{ line.oldLineNumber || '' }}
              </div>

              <!-- New Line Number & Quick Add Comment Trigger -->
              <div
                v-if="line.type !== 'hunk-header'"
                class="w-10 pe-2 text-end text-[10px] text-muted/60 select-none shrink-0 relative group/gutter"
              >
                <button
                  type="button"
                  class="absolute start-0 top-0.5 h-4 w-4 rounded bg-primary text-white flex items-center justify-center opacity-0 group-hover/gutter:opacity-100 transition-opacity shadow-xs cursor-pointer z-10"
                  :title="t('provider.commentOnLine')"
                  @mousedown.stop="startLineSelection(line)"
                >
                  <UIcon name="i-lucide-plus" class="h-3 w-3" />
                </button>
                <span>{{ line.newLineNumber || '' }}</span>
              </div>

              <!-- Gutter Prefix -->
              <div
                v-if="line.type !== 'hunk-header'"
                class="w-4 text-center font-bold select-none shrink-0"
                :class="line.type === 'addition' ? 'text-success' : line.type === 'deletion' ? 'text-error' : 'text-muted'"
              >
                {{ line.type === 'addition' ? '+' : line.type === 'deletion' ? '-' : ' ' }}
              </div>

              <!-- Line Content -->
              <div
                class="flex-1 px-1 font-mono"
                :class="isWordWrap ? 'whitespace-pre-wrap break-all' : 'whitespace-pre overflow-x-visible'"
              >
                {{ line.content }}
              </div>
            </div>

            <!-- Inline Review Comment Form directly below selected range -->
            <div
              v-if="activeCommentLine !== null && (line.newLineNumber === activeCommentLine || line.oldLineNumber === activeCommentLine)"
              class="p-3 bg-[var(--gv-surface)] border-y border-[var(--gv-diff-modify-border)] shadow-inner space-y-2"
            >
              <div class="flex items-center justify-between text-xs font-semibold text-highlighted">
                <span class="flex items-center gap-1.5 text-primary">
                  <UIcon name="i-lucide-message-square" class="h-3.5 w-3.5" />
                  <span v-if="selectionStartLine && selectionEndLine && selectionStartLine !== selectionEndLine">
                    {{ t('provider.commentOnLines', { start: Math.min(selectionStartLine, selectionEndLine), end: Math.max(selectionStartLine, selectionEndLine) }) }}
                  </span>
                  <span v-else>
                    {{ t('provider.commentOnLineNumber', { line: activeCommentLine }) }}
                  </span>
                </span>

                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  square
                  icon="i-lucide-x"
                  @click="cancelComment"
                />
              </div>

              <MarkdownCommentEditor
                :is-submitting="isSubmittingInlineComment"
                @submit="submitInlineComment"
              />
            </div>
          </template>
        </div>

        <!-- 2. Split (Side-by-Side) Diff View -->
        <div v-else class="divide-y divide-default/15">
          <template v-for="(row, rowIndex) in splitRows" :key="`split-row-${rowIndex}`">
            <div
              class="flex items-stretch min-w-full transition-colors"
              :class="isRowInSelection(row) ? 'bg-[var(--gv-diff-modify-bg)]/80 text-[var(--gv-diff-modify-text)] border-s-2 border-[var(--gv-diff-modify-border)]' : ''"
              @mouseenter="handleRowHover(row)"
            >
              <!-- Left Side (Old/Base) -->
              <div
                class="w-1/2 flex items-start border-e border-default/30 px-1 py-0.5 overflow-hidden group/left-gutter relative"
                :class="[
                  row.left?.type === 'hunk-header' ? 'bg-[var(--gv-diff-hunk-bg)]/40 text-[var(--gv-diff-hunk-text)] font-semibold' :
                  row.left?.type === 'deletion' && row.right?.type === 'addition' ? 'bg-[var(--gv-diff-modify-bg)]/60 text-[var(--gv-diff-modify-text)]' :
                  row.left?.type === 'deletion' ? 'bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)]' :
                  row.left ? 'hover:bg-muted/10 text-toned' : 'bg-muted/5'
                ]"
              >
                <!-- Old Line Number with Gutter button if deletion-only line -->
                <div
                  v-if="row.left && row.left.type !== 'hunk-header'"
                  class="w-10 pe-2 text-end text-[10px] text-muted/60 select-none shrink-0 relative group/gutter"
                >
                  <button
                    v-if="!row.right"
                    type="button"
                    class="absolute start-0 top-0.5 h-4 w-4 rounded bg-primary text-white flex items-center justify-center opacity-0 group-hover/gutter:opacity-100 transition-opacity shadow-xs cursor-pointer z-10"
                    :title="t('provider.commentOnLine')"
                    @mousedown.stop="startLineSelection(row.left)"
                  >
                    <UIcon name="i-lucide-plus" class="h-3 w-3" />
                  </button>
                  <span>{{ row.left.oldLineNumber || '' }}</span>
                </div>
                <div v-else-if="!row.left" class="w-10 shrink-0" />

                <span class="w-3 text-center font-bold select-none shrink-0" :class="row.left?.type === 'deletion' ? 'text-error' : 'text-muted'">
                  {{ row.left?.type === 'deletion' ? '-' : '' }}
                </span>
                <span
                  class="flex-1 px-1 font-mono"
                  :class="isWordWrap ? 'whitespace-pre-wrap break-all' : 'whitespace-pre overflow-x-visible'"
                >
                  {{ row.left?.content || '' }}
                </span>
              </div>

              <!-- Right Side (New/Head) -->
              <div
                class="w-1/2 flex items-start px-1 py-0.5 overflow-hidden group/right-gutter relative"
                :class="[
                  row.right?.type === 'hunk-header' ? 'bg-[var(--gv-diff-hunk-bg)]/40 text-[var(--gv-diff-hunk-text)] font-semibold' :
                  row.left?.type === 'deletion' && row.right?.type === 'addition' ? 'bg-[var(--gv-diff-modify-bg)]/60 text-[var(--gv-diff-modify-text)]' :
                  row.right?.type === 'addition' ? 'bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)]' :
                  row.right ? 'hover:bg-muted/10 text-toned' : 'bg-muted/5'
                ]"
              >
                <!-- New Line Number with Gutter button -->
                <div
                  v-if="row.right && row.right.type !== 'hunk-header'"
                  class="w-10 pe-2 text-end text-[10px] text-muted/60 select-none shrink-0 relative group/gutter"
                >
                  <button
                    type="button"
                    class="absolute start-0 top-0.5 h-4 w-4 rounded bg-primary text-white flex items-center justify-center opacity-0 group-hover/gutter:opacity-100 transition-opacity shadow-xs cursor-pointer z-10"
                    :title="t('provider.commentOnLine')"
                    @mousedown.stop="startLineSelection(row.right)"
                  >
                    <UIcon name="i-lucide-plus" class="h-3 w-3" />
                  </button>
                  <span>{{ row.right.newLineNumber || '' }}</span>
                </div>
                <div v-else-if="!row.right" class="w-10 shrink-0" />

                <span class="w-3 text-center font-bold select-none shrink-0" :class="row.right?.type === 'addition' ? 'text-success' : 'text-muted'">
                  {{ row.right?.type === 'addition' ? '+' : '' }}
                </span>
                <span
                  class="flex-1 px-1 font-mono"
                  :class="isWordWrap ? 'whitespace-pre-wrap break-all' : 'whitespace-pre overflow-x-visible'"
                >
                  {{ row.right?.content || '' }}
                </span>
              </div>
            </div>

            <!-- Inline Review Comment Form for Split View -->
            <div
              v-if="isCommentActiveOnRow(row)"
              class="w-full p-3 bg-[var(--gv-surface)] border-y border-[var(--gv-diff-modify-border)] shadow-inner space-y-2"
            >
              <div class="flex items-center justify-between text-xs font-semibold text-highlighted">
                <span class="flex items-center gap-1.5 text-primary">
                  <UIcon name="i-lucide-message-square" class="h-3.5 w-3.5" />
                  <span v-if="selectionStartLine && selectionEndLine && selectionStartLine !== selectionEndLine">
                    {{ t('provider.commentOnLines', { start: Math.min(selectionStartLine, selectionEndLine), end: Math.max(selectionStartLine, selectionEndLine) }) }}
                  </span>
                  <span v-else>
                    {{ t('provider.commentOnLineNumber', { line: activeCommentLine }) }}
                  </span>
                </span>

                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  square
                  icon="i-lucide-x"
                  @click="cancelComment"
                />
              </div>

              <MarkdownCommentEditor
                :is-submitting="isSubmittingInlineComment"
                @submit="submitInlineComment"
              />
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
