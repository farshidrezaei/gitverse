<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { DetailedRepositoryStatus, StashRecord, FileChangeStatus } from '~/types/repository'
import type { SelectedDiffTarget } from '~/composables/useStagingController'
import { buildFileTree } from '~/utils/fileTree'
import StagingTreeNodeItem from '../StagingTreeNodeItem.vue'
import FileStatusBadge from '~/components/shared/FileStatusBadge.vue'
import CreateStashModal from './CreateStashModal.vue'
import ConflictResolverModal from './ConflictResolverModal.vue'
import StashPreviewModal from './StashPreviewModal.vue'
import { useStashController } from '~/composables/useStashController'
import { useConflictController } from '~/composables/useConflictController'
import { formatStashDisplayMessage, formatStashRelativeTime } from '~/utils/stash'

interface WorkspaceStagingSidebarProps {
  repositoryStatus: DetailedRepositoryStatus | null
  selectedDiffTarget: SelectedDiffTarget | null
  stagingViewMode: 'flat' | 'tree'
  isLoading: boolean
  commitMessage: string
  isAmend: boolean
  isBypassHooks?: boolean
}

const props = withDefaults(defineProps<WorkspaceStagingSidebarProps>(), {
  isBypassHooks: false,
})

const emit = defineEmits<{
  'update:stagingViewMode': [mode: 'flat' | 'tree']
  'update:commitMessage': [value: string]
  'update:isAmend': [value: boolean]
  'update:isBypassHooks': [value: boolean]
  stageFile: [filePath: string]
  unstageFile: [filePath: string]
  stageAll: []
  unstageAll: []
  discardFile: [filePath: string, isUntracked: boolean]
  discardAllUnstaged: []
  selectFile: [filePath: string, isStaged: boolean]
  openDiffTab: [filePath: string, isStaged: boolean, changeStatus?: FileChangeStatus]
  commit: []
  refresh: []
}>()

const { t } = useI18n()

const {
  stashRecords,
  isLoadingStashes,
  saveStash,
  applyStash,
  popStash,
  dropStash,
} = useStashController()

const { confirmStandard } = useConfirmationDialog()
const { openInEditor, openInFileManager } = useExternalToolsController()

const isCreateStashModalOpen = ref(false)
const isPreviewStashModalOpen = ref<boolean>(false)
const selectedStashIndexForPreview = ref<number | null>(null)
const selectedStashIdentifierForPreview = ref<string | null>(null)
const isStashSectionOpen = ref(true)

function openStashPreview(stashIndex: number, stashIdentifier: string): void {
  selectedStashIndexForPreview.value = stashIndex
  selectedStashIdentifierForPreview.value = stashIdentifier
  isPreviewStashModalOpen.value = true
}

function toggleStashSection(): void {
  isStashSectionOpen.value = !isStashSectionOpen.value
}

async function handleDropStash(stash: StashRecord): Promise<void> {
  const isConfirmed = await confirmStandard({
    title: t('stash.dropConfirmTitle'),
    description: t('stash.dropConfirmDescription', { stash: stash.stash_identifier }),
    confirmLabel: t('stash.drop'),
    variant: 'danger',
  })
  if (isConfirmed) {
    await dropStash(stash.index)
  }
}

const {
  activeConflictFile,
  loadConflictDetails,
  resolveConflict,
  writeConflictResolvedContent,
  abortOperation,
  continueOperation,
} = useConflictController()

const isConflictModalOpen = ref(false)

async function openConflictResolver(filePath: string): Promise<void> {
  const details = await loadConflictDetails(filePath)
  if (details) {
    isConflictModalOpen.value = true
  }
}

async function handleModalResolve(strategy: 'ours' | 'theirs' | 'mark_resolved'): Promise<void> {
  if (!activeConflictFile.value) {
    return
  }
  const isResolved = await resolveConflict(activeConflictFile.value.file_path, strategy)
  if (isResolved) {
    emit('refresh')
  }
}

async function handleModalSaveContent(content: string): Promise<void> {
  if (!activeConflictFile.value) {
    return
  }
  const isSaved = await writeConflictResolvedContent(activeConflictFile.value.file_path, content)
  if (isSaved) {
    emit('refresh')
  }
}

function getOperationStateLabel(operationState?: string): string {
  switch (operationState) {
    case 'merging':
      return t('conflict.operationMerge')
    case 'rebasing':
      return t('conflict.operationRebase')
    case 'cherrypicking':
      return t('conflict.operationCherryPick')
    case 'reverting':
      return t('conflict.operationRevert')
    default:
      return t('conflict.title')
  }
}

const stagedTreeNodes = computed(() => {
  return buildFileTree(props.repositoryStatus?.staged_files || [])
})

const unstagedTreeNodes = computed(() => {
  return buildFileTree(props.repositoryStatus?.unstaged_files || [])
})

function getFlatFileContextMenuItems(
  filePath: string,
  isStaged: boolean,
  isUntracked?: boolean
): ContextMenuItem[][] {
  return [
    [
      isStaged
        ? {
            label: t('git.unstage'),
            icon: 'i-lucide-minus',
            onSelect: () => emit('unstageFile', filePath),
          }
        : {
            label: t('git.stage'),
            icon: 'i-lucide-plus',
            onSelect: () => emit('stageFile', filePath),
          },
      ...(isStaged
        ? []
        : [
            {
              label: t('diff.discardFile'),
              icon: 'i-lucide-undo-2',
              color: 'error' as const,
              onSelect: () => emit('discardFile', filePath, isUntracked || false),
            },
          ]),
    ],
    [
      {
        label: t('externalTools.openFileInEditor'),
        icon: 'i-lucide-code-2',
        onSelect: () => openInEditor('vscode', filePath),
      },
      {
        label: t('externalTools.revealInFileManager'),
        icon: 'i-lucide-folder-open',
        onSelect: () => openInFileManager(filePath),
      },
    ],
    [
      {
        label: t('git.copyPath'),
        icon: 'i-lucide-copy',
        onSelect: async () => {
          await navigator.clipboard.writeText(filePath)
        },
      },
    ],
  ]
}
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden">
    <!-- Staging Header with View Toggle (Flat vs Tree) -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20 gap-1.5">
      <span class="text-xs font-semibold text-highlighted uppercase tracking-wider truncate">
        {{ t('navigation.changes') }}
      </span>

      <div class="flex items-center gap-1 shrink-0">
        <!-- Segmented Pill Switcher: Flat vs Tree (Compact icon-based with tooltips) -->
        <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs">
          <UTooltip :text="t('repository.viewFlat')">
            <button
              type="button"
              class="flex items-center justify-center rounded-md p-1 transition-all"
              :class="props.stagingViewMode === 'flat'
                ? 'bg-primary text-inverted shadow-xs'
                : 'text-muted hover:text-highlighted'"
              :aria-label="t('repository.viewFlat')"
              @click="emit('update:stagingViewMode', 'flat')"
            >
              <UIcon name="i-lucide-list" class="h-3.5 w-3.5" />
            </button>
          </UTooltip>

          <UTooltip :text="t('repository.viewTree')">
            <button
              type="button"
              class="flex items-center justify-center rounded-md p-1 transition-all"
              :class="props.stagingViewMode === 'tree'
                ? 'bg-primary text-inverted shadow-xs'
                : 'text-muted hover:text-highlighted'"
              :aria-label="t('repository.viewTree')"
              @click="emit('update:stagingViewMode', 'tree')"
            >
              <UIcon name="i-lucide-folder-tree" class="h-3.5 w-3.5" />
            </button>
          </UTooltip>
        </div>

        <div class="h-3.5 w-px bg-default/80 mx-0.5" aria-hidden="true" />

        <!-- Refresh Button -->
        <UTooltip :text="t('common.refresh')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-rotate-cw"
            :aria-label="t('common.refresh')"
            @click="emit('refresh')"
          />
        </UTooltip>

        <!-- Stash Button -->
        <UTooltip :text="t('stash.createTitle')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-archive"
            :aria-label="t('stash.createTitle')"
            @click="isCreateStashModalOpen = true"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Scrollable file lists container -->
    <div class="flex-1 overflow-y-auto p-3 text-xs space-y-4">
      <!-- Loading skeleton -->
      <div v-if="props.isLoading && !props.repositoryStatus" class="space-y-4">
        <div>
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-1.5">
              <span class="h-2 w-2 rounded-full bg-success/40" />
              <USkeleton class="h-3.5 w-20" />
            </div>
          </div>
          <div class="rounded-lg border border-default/70 p-2 space-y-2 bg-surface/80">
            <div v-for="skeletonIndex in 2" :key="skeletonIndex" class="flex items-center gap-2">
              <USkeleton class="h-3 w-4" />
              <USkeleton class="h-3 w-40" />
            </div>
          </div>
        </div>
        <div>
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-1.5">
              <span class="h-2 w-2 rounded-full bg-warning/40" />
              <USkeleton class="h-3.5 w-24" />
            </div>
          </div>
          <div class="rounded-lg border border-default/70 p-2 space-y-2 bg-surface/80">
            <div v-for="skeletonIndex in 3" :key="skeletonIndex" class="flex items-center gap-2">
              <USkeleton class="h-3 w-4" />
              <USkeleton class="h-3 w-48" />
            </div>
          </div>
        </div>
      </div>

      <template v-else>
        <!-- Conflicted Files & Active Operation Banner -->
        <div
          v-if="props.repositoryStatus?.conflicted_files?.length || (props.repositoryStatus?.operation_state && props.repositoryStatus.operation_state !== 'none')"
          class="space-y-2 p-2.5 rounded-xl border border-error/50 bg-error/10"
        >
          <!-- Operation Header & Controls -->
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-1.5 min-w-0 text-error">
              <UIcon name="i-lucide-alert-triangle" class="h-4 w-4 shrink-0" />
              <span class="font-bold text-xs truncate">
                {{ getOperationStateLabel(props.repositoryStatus?.operation_state) }}
              </span>
              <UBadge
                v-if="props.repositoryStatus?.conflicted_files?.length"
                size="xs"
                color="error"
                variant="solid"
                class="ms-1"
              >
                {{ props.repositoryStatus.conflicted_files.length }}
              </UBadge>
            </div>

            <!-- Operation Abort / Continue Buttons -->
            <div class="flex items-center gap-1 shrink-0">
              <UButton
                size="xs"
                color="error"
                variant="ghost"
                :label="t('conflict.abortOperation')"
                @click="abortOperation()"
              />
              <UButton
                size="xs"
                color="success"
                variant="solid"
                :label="t('conflict.continueOperation')"
                :disabled="Boolean(props.repositoryStatus?.conflicted_files?.length)"
                @click="continueOperation()"
              />
            </div>
          </div>

          <!-- Conflicted File List with Quick Actions -->
          <div
            v-if="props.repositoryStatus?.conflicted_files?.length"
            class="rounded-lg border border-error/30 divide-y divide-error/20 bg-surface/80 overflow-hidden"
          >
            <div
              v-for="file in props.repositoryStatus.conflicted_files"
              :key="file.path"
              class="flex items-center justify-between px-2.5 py-1.5 gap-2 hover:bg-error/5 transition-colors"
            >
              <div class="flex items-center gap-1.5 min-w-0 flex-1">
                <span class="font-mono font-bold text-error text-[11px] shrink-0">!</span>
                <span
                  class="truncate font-mono text-[11px] text-error font-medium"
                  dir="ltr"
                  :title="file.path"
                >
                  {{ file.path }}
                </span>
              </div>

              <!-- Action buttons for each conflicted file -->
              <div class="flex items-center gap-1 shrink-0">
                <UTooltip :text="t('conflict.acceptOurs')">
                  <UButton
                    size="xs"
                    color="primary"
                    variant="ghost"
                    :label="t('conflict.oursTitle')"
                    class="text-[10px] px-1.5 py-0.5"
                    @click="resolveConflict(file.path, 'ours')"
                  />
                </UTooltip>
                <UTooltip :text="t('conflict.acceptTheirs')">
                  <UButton
                    size="xs"
                    color="warning"
                    variant="ghost"
                    :label="t('conflict.theirsTitle')"
                    class="text-[10px] px-1.5 py-0.5"
                    @click="resolveConflict(file.path, 'theirs')"
                  />
                </UTooltip>
                <UButton
                  size="xs"
                  color="neutral"
                  variant="outline"
                  icon="i-lucide-columns-2"
                  :label="t('conflict.resolveModal')"
                  class="text-[10px] px-1.5 py-0.5"
                  @click="openConflictResolver(file.path)"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Staged Section -->
        <div>
          <div class="flex items-center justify-between mb-1.5 gap-1">
            <span class="font-semibold text-highlighted flex items-center gap-1.5 text-xs truncate min-w-0">
              <span class="h-2 w-2 rounded-full bg-success shrink-0" />
              <span class="truncate">{{ t('git.status.staged') }}</span>
              <span class="text-muted shrink-0">({{ props.repositoryStatus?.staged_files?.length || 0 }})</span>
            </span>

            <div v-if="props.repositoryStatus?.staged_files?.length" class="flex items-center gap-0.5 shrink-0">
              <UTooltip :text="t('git.unstageAll')">
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-minus"
                  :aria-label="t('git.unstageAll')"
                  @click="emit('unstageAll')"
                />
              </UTooltip>
            </div>
          </div>

          <!-- Staged Tree View -->
          <div
            v-if="props.stagingViewMode === 'tree' && props.repositoryStatus?.staged_files?.length"
            class="rounded-lg border border-default/70 p-2 bg-surface/80 space-y-0.5"
          >
            <StagingTreeNodeItem
              v-for="node in stagedTreeNodes"
              :key="node.id"
              :node="node"
              :is-staged="true"
              :selected-path="props.selectedDiffTarget?.filePath"
              :selected-is-staged="props.selectedDiffTarget?.isStaged"
              @stage="emit('stageFile', $event)"
              @unstage="emit('unstageFile', $event)"
              @discard="emit('discardFile', $event, false)"
              @select="emit('selectFile', $event, true)"
              @open-diff-tab="(path) => emit('openDiffTab', path, true)"
            />
          </div>

          <!-- Staged Flat View -->
          <div
            v-else-if="props.stagingViewMode === 'flat' && props.repositoryStatus?.staged_files?.length"
            class="rounded-lg border border-default/70 divide-y divide-default bg-surface/80"
          >
            <UContextMenu
              v-for="file in props.repositoryStatus.staged_files"
              :key="file.path"
              :items="getFlatFileContextMenuItems(file.path, true)"
            >
              <div
                class="group flex items-center justify-between px-2.5 py-1.5 cursor-pointer transition-colors"
                :class="props.selectedDiffTarget?.filePath === file.path && props.selectedDiffTarget?.isStaged
                  ? 'bg-primary/15 text-primary font-medium'
                  : 'hover:bg-muted/30'"
                @click="emit('selectFile', file.path, true)"
                @dblclick="emit('openDiffTab', file.path, true, file.status)"
              >
                <div class="flex items-center gap-2 min-w-0">
                  <FileStatusBadge :status="file.status" size="xs" />
                  <span class="truncate font-mono text-[11px]" dir="ltr">
                    {{ file.path }}
                    <span v-if="file.original_path" class="text-muted text-[10px]">← {{ file.original_path }}</span>
                  </span>
                </div>
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-minus"
                  class="h-5 w-5 shrink-0 opacity-70 group-hover:opacity-100"
                  title="Unstage"
                  @click.stop="emit('unstageFile', file.path)"
                />
              </div>
            </UContextMenu>
          </div>
          <div v-else class="text-[11px] text-muted italic px-1">
            No staged changes
          </div>
        </div>

        <!-- Unstaged Section -->
        <div>
          <div class="flex items-center justify-between mb-1.5 gap-1">
            <span class="font-semibold text-highlighted flex items-center gap-1.5 text-xs truncate min-w-0">
              <span class="h-2 w-2 rounded-full bg-warning shrink-0" />
              <span class="truncate">{{ t('git.status.unstaged') }}</span>
              <span class="text-muted shrink-0">({{ props.repositoryStatus?.unstaged_files?.length || 0 }})</span>
            </span>

            <div v-if="props.repositoryStatus?.unstaged_files?.length" class="flex items-center gap-0.5 shrink-0">
              <UTooltip :text="t('git.discardAll')">
                <UButton
                  size="xs"
                  color="error"
                  variant="ghost"
                  icon="i-lucide-trash"
                  :aria-label="t('git.discardAll')"
                  @click="emit('discardAllUnstaged')"
                />
              </UTooltip>
              <UTooltip :text="t('git.stageAll')">
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-plus"
                  :aria-label="t('git.stageAll')"
                  @click="emit('stageAll')"
                />
              </UTooltip>
            </div>
          </div>

          <!-- Unstaged Tree View -->
          <div
            v-if="props.stagingViewMode === 'tree' && props.repositoryStatus?.unstaged_files?.length"
            class="rounded-lg border border-default/70 p-2 bg-surface/80 space-y-0.5"
          >
            <StagingTreeNodeItem
              v-for="node in unstagedTreeNodes"
              :key="node.id"
              :node="node"
              :is-staged="false"
              :selected-path="props.selectedDiffTarget?.filePath"
              :selected-is-staged="props.selectedDiffTarget?.isStaged"
              @stage="emit('stageFile', $event)"
              @unstage="emit('unstageFile', $event)"
              @discard="emit('discardFile', $event, false)"
              @select="emit('selectFile', $event, false)"
              @open-diff-tab="(path) => emit('openDiffTab', path, false)"
            />
          </div>

          <!-- Unstaged Flat View -->
          <div
            v-else-if="props.stagingViewMode === 'flat' && props.repositoryStatus?.unstaged_files?.length"
            class="rounded-lg border border-default/70 divide-y divide-default bg-surface/80"
          >
            <UContextMenu
              v-for="file in props.repositoryStatus.unstaged_files"
              :key="file.path"
              :items="getFlatFileContextMenuItems(file.path, false, file.status === 'untracked')"
            >
              <div
                class="group flex items-center justify-between px-2.5 py-1.5 cursor-pointer transition-colors"
                :class="props.selectedDiffTarget?.filePath === file.path && !props.selectedDiffTarget?.isStaged
                  ? 'bg-primary/15 text-primary font-medium'
                  : 'hover:bg-muted/30'"
                @click="emit('selectFile', file.path, false)"
                @dblclick="emit('openDiffTab', file.path, false, file.status)"
              >
                <div class="flex items-center gap-2 min-w-0">
                  <FileStatusBadge :status="file.status" size="xs" />
                  <span class="truncate font-mono text-[11px]" dir="ltr">{{ file.path }}</span>
                </div>

                <div class="flex items-center gap-0.5 opacity-70 group-hover:opacity-100">
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-undo-2"
                    class="h-5 w-5 shrink-0"
                    title="Discard changes"
                    @click.stop="emit('discardFile', file.path, file.status === 'untracked')"
                  />
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-plus"
                    class="h-5 w-5 shrink-0"
                    title="Stage"
                    @click.stop="emit('stageFile', file.path)"
                  />
                </div>
              </div>
            </UContextMenu>
          </div>
          <div v-else class="text-[11px] text-muted italic px-1">
            Working tree clean
          </div>
        </div>

        <!-- Stashes Section -->
        <div class="pt-2 border-t border-default/40">
          <div class="flex items-center justify-between mb-1.5">
            <button
              type="button"
              class="flex items-center gap-1.5 text-xs font-semibold text-highlighted hover:text-default transition-colors select-none"
              @click="toggleStashSection"
            >
              <UIcon
                :name="isStashSectionOpen ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
                class="h-3 w-3 text-muted transition-transform rtl:rotate-180"
              />
              <UIcon name="i-lucide-archive" class="h-3.5 w-3.5 text-primary/80" />
              <span>{{ t('stash.title') }}</span>
              <span class="text-muted font-normal">({{ stashRecords.length }})</span>
            </button>

            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-plus"
              :label="t('stash.button')"
              @click="isCreateStashModalOpen = true"
            />
          </div>

          <!-- Stash list -->
          <div
            v-if="isStashSectionOpen && stashRecords.length > 0"
            class="space-y-1.5"
          >
            <div
              v-for="stashItem in stashRecords"
              :key="stashItem.stash_identifier"
              class="group flex items-center justify-between rounded-lg border border-default/70 p-2 bg-surface/80 hover:bg-muted/10 transition-colors"
            >
              <div class="min-w-0 flex-1 me-2">
                <div class="flex items-center gap-1.5 min-w-0">
                  <UBadge
                    size="xs"
                    color="neutral"
                    variant="subtle"
                    class="font-mono text-[10px] shrink-0"
                  >
                    {{ stashItem.stash_identifier }}
                  </UBadge>
                  <span
                    class="font-mono text-[11px] truncate text-highlighted"
                    :title="stashItem.message"
                  >
                    {{ formatStashDisplayMessage(stashItem.message) }}
                  </span>
                  <span class="text-[10px] text-muted shrink-0 ms-auto">
                    {{ formatStashRelativeTime(stashItem.timestamp_seconds) }}
                  </span>
                </div>
              </div>

              <!-- Action buttons: Inspect, Apply, Pop, Drop -->
              <div class="flex items-center gap-0.5 shrink-0 opacity-80 group-hover:opacity-100 transition-opacity">
                <UTooltip :text="t('stash.inspectTooltip')">
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-file-search"
                    :aria-label="t('stash.inspect')"
                    @click="openStashPreview(stashItem.index, stashItem.stash_identifier)"
                  />
                </UTooltip>
                <UTooltip :text="t('stash.applyTooltip')">
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-play"
                    :aria-label="t('stash.apply')"
                    @click="applyStash(stashItem.index)"
                  />
                </UTooltip>
                <UTooltip :text="t('stash.popTooltip')">
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-arrow-up-right"
                    :aria-label="t('stash.pop')"
                    @click="popStash(stashItem.index)"
                  />
                </UTooltip>
                <UTooltip :text="t('stash.dropTooltip')">
                  <UButton
                    size="xs"
                    color="error"
                    variant="ghost"
                    icon="i-lucide-trash-2"
                    :aria-label="t('stash.drop')"
                    @click="handleDropStash(stashItem)"
                  />
                </UTooltip>
              </div>
            </div>
          </div>

          <div
            v-else-if="isStashSectionOpen && stashRecords.length === 0"
            class="text-[11px] text-muted italic px-1 py-0.5"
          >
            {{ t('stash.noStashes') }}
          </div>
        </div>
      </template>
    </div>

    <!-- Commit Box (Docked at Bottom) -->
    <div class="border-t border-default/70 p-3 bg-muted/10 flex flex-col gap-2 shrink-0">
      <UTextarea
        :model-value="props.commitMessage"
        :rows="3"
        placeholder="Commit subject (e.g. feat: add live status tracker)..."
        class="w-full font-mono text-xs"
        @update:model-value="emit('update:commitMessage', $event)"
      />
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5">
            <USwitch
              :model-value="props.isAmend"
              size="xs"
              @update:model-value="emit('update:isAmend', $event)"
            />
            <span class="text-[11px] text-muted">{{ t('git.amend') }}</span>
          </div>

          <div class="flex items-center gap-1.5">
            <USwitch
              :model-value="props.isBypassHooks"
              size="xs"
              @update:model-value="emit('update:isBypassHooks', $event)"
            />
            <UTooltip :text="t('git.bypassHooksTooltip')">
              <span class="text-[11px] text-muted cursor-help">{{ t('git.bypassHooks') }}</span>
            </UTooltip>
          </div>
        </div>
        <UButton
          color="primary"
          variant="solid"
          size="xs"
          icon="i-lucide-git-commit"
          :label="t('git.commit')"
          :disabled="!props.repositoryStatus?.staged_files?.length && !props.isAmend"
          @click="emit('commit')"
        />
      </div>
    </div>

    <!-- Create Stash Modal -->
    <CreateStashModal
      v-model:open="isCreateStashModalOpen"
      @save="saveStash"
    />

    <!-- Stash Preview Modal -->
    <StashPreviewModal
      v-model:open="isPreviewStashModalOpen"
      :stash-index="selectedStashIndexForPreview"
      :stash-identifier="selectedStashIdentifierForPreview"
      @apply="applyStash"
      @pop="popStash"
      @drop="dropStash"
    />

    <!-- Conflict Resolver Modal -->
    <ConflictResolverModal
      v-model:open="isConflictModalOpen"
      :conflict-details="activeConflictFile"
      @resolve="handleModalResolve"
      @save-content="handleModalSaveContent"
    />
  </div>
</template>
