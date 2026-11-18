<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { CommitGraphNode, GitCommit } from '~/types/repository'
import { formatRelativeTime } from '~/utils/dateFormat'
import CommitHashPill from '~/components/shared/CommitHashPill.vue'

interface CommitGraphRowProps {
  node: CommitGraphNode
  index: number
  isSelected: boolean
  isComparisonBase?: boolean
  activeSelectedHash?: string | null
  graphWidth: number
  rowHeight: number
}

const props = withDefaults(defineProps<CommitGraphRowProps>(), {
  isComparisonBase: false,
  activeSelectedHash: null,
})

const emit = defineEmits<{
  selectCommit: [commitHash: string]
  compareCommits: [baseHash: string, targetHash: string]
  createBranchAtCommit: [commitSha: string]
  checkoutCommit: [commitSha: string]
  cherryPickCommit: [commitHash: string]
  revertCommit: [commitHash: string]
  resetToCommit: [commitHash: string, mode: 'soft' | 'mixed' | 'hard']
  rebaseInteractive: [commitHash: string]
}>()

const { t } = useI18n()

function handleRowClick(event: MouseEvent): void {
  if (
    (event.ctrlKey || event.metaKey) &&
    props.activeSelectedHash &&
    props.activeSelectedHash !== props.node.commit.hash
  ) {
    emit('compareCommits', props.activeSelectedHash, props.node.commit.hash)
    return
  }
  emit('selectCommit', props.node.commit.hash)
}

function getCommitContextMenuItems(commit: GitCommit): ContextMenuItem[][] {
  const items: ContextMenuItem[][] = [
    [
      {
        label: t('commit.details'),
        icon: 'i-lucide-file-search',
        onSelect: () => emit('selectCommit', commit.hash),
      },
      ...(props.activeSelectedHash && props.activeSelectedHash !== commit.hash
        ? [
            {
              label: t('commit.compareWithSelected', { hash: props.activeSelectedHash.slice(0, 7) }),
              icon: 'i-lucide-arrow-left-right',
              onSelect: () => emit('compareCommits', props.activeSelectedHash!, commit.hash),
            },
          ]
        : []),
      {
        label: t('commit.createBranchHere'),
        icon: 'i-lucide-git-branch-plus',
        onSelect: () => emit('createBranchAtCommit', commit.hash),
      },
      {
        label: t('commit.checkoutCommit'),
        icon: 'i-lucide-log-in',
        onSelect: () => emit('checkoutCommit', commit.hash),
      },
    ],
    [
      {
        label: t('commit.cherryPick'),
        icon: 'i-lucide-git-cherry-pick',
        onSelect: () => emit('cherryPickCommit', commit.hash),
      },
      {
        label: t('commit.revert'),
        icon: 'i-lucide-undo-2',
        onSelect: () => emit('revertCommit', commit.hash),
      },
      {
        label: t('rebase.interactiveRebaseHere'),
        icon: 'i-lucide-list-ordered',
        onSelect: () => emit('rebaseInteractive', commit.hash),
      },
      {
        label: t('commit.resetToHere'),
        icon: 'i-lucide-history',
        children: [
          {
            label: t('commit.resetSoft'),
            onSelect: () => emit('resetToCommit', commit.hash, 'soft'),
          },
          {
            label: t('commit.resetMixed'),
            onSelect: () => emit('resetToCommit', commit.hash, 'mixed'),
          },
          {
            label: t('commit.resetHard'),
            color: 'error' as const,
            onSelect: () => emit('resetToCommit', commit.hash, 'hard'),
          },
        ],
      },
    ],
    [
      {
        label: t('commit.copySha'),
        icon: 'i-lucide-copy',
        onSelect: async () => {
          await navigator.clipboard.writeText(commit.hash)
        },
      },
      {
        label: t('commit.copySubject'),
        icon: 'i-lucide-clipboard-copy',
        onSelect: async () => {
          await navigator.clipboard.writeText(commit.subject)
        },
      },
    ],
  ]
  return items
}
</script>

<template>
  <UContextMenu :items="getCommitContextMenuItems(props.node.commit)">
    <div
      class="group relative flex w-full items-center border-b border-default/20 px-2 transition-colors duration-75 cursor-pointer text-xs"
      :class="{
        'bg-warning/15 text-warning font-medium border-s-2 border-warning': props.isComparisonBase,
        'bg-primary/10 text-primary font-medium': props.isSelected && !props.isComparisonBase,
        'hover:bg-muted/15 text-highlighted': !props.isSelected && !props.isComparisonBase,
      }"
      :style="{
        height: `${props.rowHeight}px`,
        contentVisibility: 'auto',
        containIntrinsicSize: `0 ${props.rowHeight}px`,
      }"
      :data-commit-hash="props.node.commit.hash"
      @click="handleRowClick"
    >
      <!-- Spacer for SVG graph lanes -->
      <div
        class="shrink-0"
        :style="{ width: `${props.graphWidth}px` }"
        aria-hidden="true"
      />

      <!-- Commit Hash Pill -->
      <div class="shrink-0 pe-2.5 font-mono text-[11px]">
        <CommitHashPill :hash="props.node.commit.hash" :show-copy="false" />
      </div>

      <!-- Commit Subject & Branch / Tag Badges -->
      <div class="flex-1 flex items-center gap-1.5 min-w-0 pe-4 overflow-hidden">
        <!-- Branch / Tag / HEAD Badges -->
        <template v-if="props.node.commit.decorations.length > 0">
          <div class="flex items-center gap-1 shrink-0">
            <UBadge
              v-for="decoration in props.node.commit.decorations"
              :key="decoration.name"
              size="xs"
              :color="decoration.is_head ? 'primary' : (decoration.reference_type === 'tag' ? 'warning' : 'neutral')"
              variant="subtle"
              class="font-mono text-[10px] px-1.5 py-0 shrink-0"
            >
              <UIcon
                v-if="decoration.is_head"
                name="i-lucide-disc"
                class="h-2.5 w-2.5 me-0.5 text-primary"
              />
              <UIcon
                v-else-if="decoration.reference_type === 'tag'"
                name="i-lucide-tag"
                class="h-2.5 w-2.5 me-0.5 text-warning"
              />
              <UIcon
                v-else
                name="i-lucide-git-branch"
                class="h-2.5 w-2.5 me-0.5 text-muted"
              />
              <span class="max-w-[140px] truncate">{{ decoration.name }}</span>
            </UBadge>
          </div>
        </template>

        <!-- Subject Text -->
        <span
          class="truncate text-highlighted group-hover:text-default"
          :title="props.node.commit.subject"
        >
          {{ props.node.commit.subject }}
        </span>
      </div>

      <!-- Author Name -->
      <div
        class="w-32 shrink-0 pe-3 text-end truncate text-muted text-[11px]"
        :title="props.node.commit.author_name"
      >
        {{ props.node.commit.author_name }}
      </div>

      <!-- Relative Timestamp -->
      <div class="w-20 shrink-0 text-end text-muted/80 text-[10px] font-mono">
        {{ formatRelativeTime(props.node.commit.author_timestamp) }}
      </div>
    </div>
  </UContextMenu>
</template>
