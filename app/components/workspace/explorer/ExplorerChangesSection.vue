<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { DetailedRepositoryStatus, FileChangeStatus } from '~/types/repository'
import type { SelectedDiffTarget } from '~/composables/useStagingController'
import { buildFileTree } from '~/utils/fileTree'
import StagingTreeNodeItem from '../StagingTreeNodeItem.vue'
import FileStatusBadge from '~/components/shared/FileStatusBadge.vue'
import ConflictResolverModal from '../staging/ConflictResolverModal.vue'
import ConventionalCommitBuilderModal from '../staging/ConventionalCommitBuilderModal.vue'
import { useConflictController, type ConflictResolutionStrategy } from '~/composables/useConflictController'
import { useExternalToolsController } from '~/composables/useExternalToolsController'

interface ExplorerChangesSectionProps {
  repositoryStatus: DetailedRepositoryStatus | null
  selectedDiffTarget: SelectedDiffTarget | null
  stagingViewMode: 'flat' | 'tree'
  isLoading: boolean
  commitMessage: string
  isAmend: boolean
  isBypassHooks?: boolean
  isOperating?: boolean
}

const props = withDefaults(defineProps<ExplorerChangesSectionProps>(), {
  isBypassHooks: false,
  isOperating: false,
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
const { openInEditor, openInFileManager } = useExternalToolsController()

const isConventionalBuilderModalOpen = ref<boolean>(false)

// Conflict Resolver Modal state
const isConflictResolverOpen = ref<boolean>(false)
const activeConflictFilePath = ref<string>('')
const {
  activeConflictFile,
  loadConflictDetails,
  resolveConflict: resolveConflictByStrategy,
  writeConflictResolvedContent,
  continueOperation,
} = useConflictController()

async function openConflictResolver(filePath: string): Promise<void> {
  activeConflictFilePath.value = filePath
  await loadConflictDetails(filePath)
  isConflictResolverOpen.value = true
}

async function resolveConflict(filePath: string, strategy: ConflictResolutionStrategy): Promise<void> {
  await resolveConflictByStrategy(filePath, strategy)
  emit('refresh')
}

async function handleSaveConflictResolution(resolvedContent: string): Promise<void> {
  if (!activeConflictFilePath.value) return
  const isSaved = await writeConflictResolvedContent(activeConflictFilePath.value, resolvedContent)
  if (isSaved) {
    isConflictResolverOpen.value = false
    emit('refresh')
  }
}

async function handleSaveAndContinue(resolvedContent: string): Promise<void> {
  if (!activeConflictFilePath.value) return
  const isSaved = await writeConflictResolvedContent(activeConflictFilePath.value, resolvedContent)
  if (isSaved) {
    isConflictResolverOpen.value = false
    emit('refresh')
    const remainingCount = (props.repositoryStatus?.conflicted_files?.length || 1) - 1
    if (remainingCount <= 0) {
      await continueOperation()
    }
  }
}

function handleApplyConventionalCommit(formattedMessage: string): void {
  emit('update:commitMessage', formattedMessage)
}

// Tree view memoized conversions
const stagedTreeNodes = computed(() => {
  return buildFileTree(props.repositoryStatus?.staged_files || [])
})

const unstagedFiles = computed(() => {
  return (props.repositoryStatus?.unstaged_files || []).filter(
    file => file.status !== 'untracked'
  )
})

const untrackedFiles = computed(() => {
  return (props.repositoryStatus?.unstaged_files || []).filter(
    file => file.status === 'untracked'
  )
})

const unstagedTreeNodes = computed(() => {
  return buildFileTree(unstagedFiles.value)
})

const untrackedTreeNodes = computed(() => {
  return buildFileTree(untrackedFiles.value)
})

function getFlatFileContextMenuItems(
  filePath: string,
  isStaged: boolean,
  isUntracked?: boolean,
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

const totalChangesCount = computed(() => {
  const staged = props.repositoryStatus?.staged_files?.length || 0
  const unstaged = props.repositoryStatus?.unstaged_files?.length || 0
  const conflicted = props.repositoryStatus?.conflicted_files?.length || 0
  return staged + unstaged + conflicted
})
</script>

<template>
  <div class="flex flex-col gap-3 p-2 text-xs">
    <!-- View Switcher & Quick Action Toolbar -->
    <div class="flex items-center justify-between gap-1 pb-1 border-b border-default/40">
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

      <div class="flex items-center gap-1">
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
        <UTooltip :text="t('common.refresh')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-refresh-cw"
            :aria-label="t('common.refresh')"
            :loading="props.isLoading"
            @click="emit('refresh')"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Skeletons when initial loading -->
    <div v-if="props.isLoading && !props.repositoryStatus" class="space-y-3 p-1">
      <div class="space-y-2">
        <USkeleton class="h-3.5 w-24" />
        <div v-for="index in 3" :key="index" class="flex items-center gap-2">
          <USkeleton class="h-3 w-3 rounded-full" />
          <USkeleton class="h-3 w-40" />
        </div>
      </div>
    </div>

    <!-- Active Status Content -->
    <div v-else class="space-y-3">
      <!-- Conflicted Files -->
      <div v-if="props.repositoryStatus?.conflicted_files?.length" class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="font-semibold text-error flex items-center gap-1.5 text-xs">
            <UIcon name="i-lucide-alert-triangle" class="h-3.5 w-3.5 text-error" />
            <span>{{ t('conflict.title') }}</span>
            <span>({{ props.repositoryStatus.conflicted_files.length }})</span>
          </span>
        </div>

        <div class="rounded-lg border border-error/30 divide-y divide-error/20 bg-surface/80 overflow-hidden">
          <div
            v-for="file in props.repositoryStatus.conflicted_files"
            :key="file.path"
            class="flex items-center justify-between px-2.5 py-1.5 gap-2 hover:bg-error/5 transition-colors"
          >
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <span class="font-mono font-bold text-error text-[11px] shrink-0">!</span>
              <span class="truncate font-mono text-[11px] text-error font-medium" dir="ltr" :title="file.path">
                {{ file.path }}
              </span>
            </div>
            <div class="flex items-center gap-1 shrink-0">
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
                class="h-5 w-5 p-0"
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
              class="group flex items-center justify-between px-2 py-1.5 cursor-pointer transition-colors"
              :class="props.selectedDiffTarget?.filePath === file.path && props.selectedDiffTarget?.isStaged
                ? 'bg-primary/15 text-primary font-medium'
                : 'hover:bg-muted/30'"
              @click="emit('selectFile', file.path, true)"
              @dblclick="emit('openDiffTab', file.path, true, file.status)"
            >
              <div class="flex items-center gap-1.5 min-w-0">
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
                square
                icon="i-lucide-minus"
                class="h-4 w-4 p-0 flex items-center justify-center shrink-0 opacity-70 group-hover:opacity-100"
                :title="t('git.unstage')"
                @click.stop="emit('unstageFile', file.path)"
              />
            </div>
          </UContextMenu>
        </div>
        <div v-else class="text-[10px] text-muted italic px-1">
          {{ t('repository.noStagedChanges') }}
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
                class="h-5 w-5 p-0"
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
                class="h-5 w-5 p-0"
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
            :items="getFlatFileContextMenuItems(file.path, false)"
          >
            <div
              class="group flex items-center justify-between px-2 py-1.5 cursor-pointer transition-colors"
              :class="props.selectedDiffTarget?.filePath === file.path && !props.selectedDiffTarget?.isStaged
                ? 'bg-primary/15 text-primary font-medium'
                : 'hover:bg-muted/30'"
              @click="emit('selectFile', file.path, false)"
              @dblclick="emit('openDiffTab', file.path, false, file.status)"
            >
              <div class="flex items-center gap-1.5 min-w-0">
                <FileStatusBadge :status="file.status" size="xs" />
                <span class="truncate font-mono text-[11px]" dir="ltr">
                  {{ file.path }}
                </span>
              </div>
              <div class="flex items-center gap-0.5 shrink-0 opacity-70 group-hover:opacity-100">
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  square
                  icon="i-lucide-plus"
                  class="h-4 w-4 p-0 flex items-center justify-center shrink-0"
                  :title="t('git.stage')"
                  @click.stop="emit('stageFile', file.path)"
                />
              </div>
            </div>
          </UContextMenu>
        </div>
        <div v-else class="text-[10px] text-muted italic px-1">
          {{ t('repository.noUnstagedChanges') }}
        </div>
      </div>

      <!-- Untracked Section -->
      <div v-if="untrackedFiles.length">
        <div class="flex items-center justify-between mb-1.5 gap-1">
          <span class="font-semibold text-highlighted flex items-center gap-1.5 text-xs truncate min-w-0">
            <span class="h-2 w-2 rounded-full bg-neutral-400 shrink-0" />
            <span class="truncate">{{ t('git.status.untracked') }}</span>
            <span class="text-muted shrink-0">({{ untrackedFiles.length }})</span>
          </span>
        </div>

        <!-- Untracked Tree View -->
        <div
          v-if="props.stagingViewMode === 'tree'"
          class="rounded-lg border border-default/70 p-2 bg-surface/80 space-y-0.5"
        >
          <StagingTreeNodeItem
            v-for="node in untrackedTreeNodes"
            :key="node.id"
            :node="node"
            :is-staged="false"
            :selected-path="props.selectedDiffTarget?.filePath"
            :selected-is-staged="props.selectedDiffTarget?.isStaged"
            @stage="emit('stageFile', $event)"
            @unstage="emit('unstageFile', $event)"
            @discard="emit('discardFile', $event, true)"
            @select="emit('selectFile', $event, false)"
            @open-diff-tab="(path) => emit('openDiffTab', path, false)"
          />
        </div>

        <!-- Untracked Flat View -->
        <div
          v-else
          class="rounded-lg border border-default/70 divide-y divide-default bg-surface/80"
        >
          <UContextMenu
            v-for="file in untrackedFiles"
            :key="file.path"
            :items="getFlatFileContextMenuItems(file.path, false, true)"
          >
            <div
              class="group flex items-center justify-between px-2 py-1.5 cursor-pointer transition-colors"
              :class="props.selectedDiffTarget?.filePath === file.path && !props.selectedDiffTarget?.isStaged
                ? 'bg-primary/15 text-primary font-medium'
                : 'hover:bg-muted/30'"
              @click="emit('selectFile', file.path, false)"
              @dblclick="emit('openDiffTab', file.path, false, file.status)"
            >
              <div class="flex items-center gap-1.5 min-w-0">
                <FileStatusBadge :status="file.status" size="xs" />
                <span class="truncate font-mono text-[11px]" dir="ltr">
                  {{ file.path }}
                </span>
              </div>
              <div class="flex items-center gap-0.5 shrink-0 opacity-70 group-hover:opacity-100">
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  square
                  icon="i-lucide-plus"
                  class="h-4 w-4 p-0 flex items-center justify-center shrink-0"
                  :title="t('git.stage')"
                  @click.stop="emit('stageFile', file.path)"
                />
              </div>
            </div>
          </UContextMenu>
        </div>
      </div>

      <!-- Quick Commit Form Box -->
      <div class="rounded-xl border border-default/70 bg-surface/90 p-2.5 flex flex-col gap-2 mt-2 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-semibold text-highlighted uppercase tracking-wider">
            {{ t('git.commit') }}
          </span>
          <UTooltip :text="t('commit.conventionalHelper')">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-sparkles"
              class="h-5 w-5 p-0 text-primary"
              :aria-label="t('commit.conventionalHelper')"
              @click="isConventionalBuilderModalOpen = true"
            />
          </UTooltip>
        </div>

        <UTextarea
          :model-value="props.commitMessage"
          :placeholder="t('commit.messagePlaceholder')"
          :rows="2"
          class="w-full text-xs font-mono"
          @update:model-value="emit('update:commitMessage', $event)"
        />

        <div class="flex items-center justify-between gap-1 text-[11px]">
          <div class="flex items-center gap-3">
            <UCheckbox
              :model-value="props.isAmend"
              :label="t('git.amend')"
              size="xs"
              @update:model-value="emit('update:isAmend', Boolean($event))"
            />

            <UCheckbox
              :model-value="props.isBypassHooks"
              :label="t('git.bypassHooks')"
              size="xs"
              @update:model-value="emit('update:isBypassHooks', Boolean($event))"
            />
          </div>

          <UButton
            size="xs"
            color="primary"
            icon="i-lucide-check"
            :loading="props.isOperating"
            :disabled="!props.commitMessage.trim() || (!props.repositoryStatus?.staged_files?.length && !props.isAmend)"
            :label="t('git.commit')"
            @click="emit('commit')"
          />
        </div>
      </div>
    </div>

    <!-- Modals -->
    <ConflictResolverModal
      v-model:open="isConflictResolverOpen"
      :conflict-details="activeConflictFile"
      :show-continue-action="Boolean(props.repositoryStatus?.operation_state && props.repositoryStatus.operation_state !== 'none')"
      @resolve="resolveConflict(activeConflictFilePath, $event)"
      @save-content="handleSaveConflictResolution"
      @save-and-continue="handleSaveAndContinue"
    />

    <ConventionalCommitBuilderModal
      v-model:open="isConventionalBuilderModalOpen"
      @apply="handleApplyConventionalCommit"
    />
  </div>
</template>
