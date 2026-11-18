<script setup lang="ts">
import type { CommitGraphHistoryResponse } from '~/types/repository'
import CommitGraphToolbar from './graph/CommitGraphToolbar.vue'
import CommitGraphSvgCanvas from './graph/CommitGraphSvgCanvas.vue'
import CommitGraphRow from './graph/CommitGraphRow.vue'

const props = defineProps<{
  graph: CommitGraphHistoryResponse | null
  selectedCommitHash: string | null
  comparisonBaseHash?: string | null
  isLoading: boolean
  isOperating?: boolean
  activeReference?: string | null
}>()

const emit = defineEmits<{
  selectCommit: [commitHash: string]
  compareCommits: [baseHash: string, targetHash: string]
  refresh: []
  push: []
  pull: []
  fetch: []
  createBranch: []
  createBranchAtCommit: [commitSha: string]
  checkoutCommit: [commitSha: string]
  cherryPickCommit: [commitHash: string]
  revertCommit: [commitHash: string]
  resetToCommit: [commitHash: string, mode: 'soft' | 'mixed' | 'hard']
  rebaseInteractive: [commitHash: string]
  resetGraphFilter: []
}>()

const { t } = useI18n()
const searchQuery = ref<string>('')
const scrollContainerRef = ref<HTMLElement | null>(null)

const ROW_HEIGHT = 36
const LANE_WIDTH = 18
const PADDING_LEFT = 16
const OVERSCAN = 8

const scrollTop = ref<number>(0)
const viewportHeight = ref<number>(600)

const filteredNodes = computed(() => {
  if (!props.graph || !props.graph.nodes) return []
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return props.graph.nodes

  return props.graph.nodes.filter((node) => {
    const commit = node.commit
    return (
      commit.hash.toLowerCase().includes(query) ||
      commit.subject.toLowerCase().includes(query) ||
      commit.author_name.toLowerCase().includes(query) ||
      commit.decorations.some((decoration) =>
        decoration.name.toLowerCase().includes(query)
      )
    )
  })
})

const graphWidth = computed(() => {
  const maxLane = props.graph?.max_lane_index || 0
  return Math.max((maxLane + 1) * LANE_WIDTH + PADDING_LEFT * 2, 60)
})

const totalHeight = computed(() => {
  return filteredNodes.value.length * ROW_HEIGHT
})

const startIndex = computed(() => {
  return Math.max(0, Math.floor(scrollTop.value / ROW_HEIGHT) - OVERSCAN)
})

const endIndex = computed(() => {
  const count = Math.ceil(viewportHeight.value / ROW_HEIGHT) + OVERSCAN * 2
  return Math.min(filteredNodes.value.length, startIndex.value + count)
})

const visibleNodes = computed(() => {
  return filteredNodes.value.slice(startIndex.value, endIndex.value)
})

const offsetY = computed(() => {
  return startIndex.value * ROW_HEIGHT
})

function handleScroll(): void {
  if (!scrollContainerRef.value) return
  scrollTop.value = scrollContainerRef.value.scrollTop
}

function updateViewportSize(): void {
  if (scrollContainerRef.value) {
    viewportHeight.value = scrollContainerRef.value.clientHeight || 600
  }
}

onMounted(() => {
  updateViewportSize()
  window.addEventListener('resize', updateViewportSize)
})

onUnmounted(() => {
  window.removeEventListener('resize', updateViewportSize)
})

watch(
  () => props.selectedCommitHash,
  (newHash) => {
    if (!newHash || !scrollContainerRef.value) return
    const targetIndex = filteredNodes.value.findIndex((node) => node.commit.hash === newHash)
    if (targetIndex >= 0) {
      const targetTop = targetIndex * ROW_HEIGHT
      const currentScroll = scrollContainerRef.value.scrollTop
      const visibleHeight = scrollContainerRef.value.clientHeight
      if (targetTop < currentScroll || targetTop > currentScroll + visibleHeight - ROW_HEIGHT) {
        scrollContainerRef.value.scrollTo({
          top: Math.max(0, targetTop - visibleHeight / 2 + ROW_HEIGHT / 2),
          behavior: 'smooth',
        })
      }
    }
  },
  { flush: 'post' }
)
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden select-none">
    <!-- Toolbar -->
    <CommitGraphToolbar
      :search-query="searchQuery"
      :total-commits-count="props.graph?.total_commits_count"
      :active-reference="props.activeReference"
      :is-operating="props.isOperating"
      :is-loading="props.isLoading"
      @update:search-query="searchQuery = $event"
      @reset-graph-filter="emit('resetGraphFilter')"
      @push="emit('push')"
      @pull="emit('pull')"
      @fetch="emit('fetch')"
      @refresh="emit('refresh')"
      @create-branch="emit('createBranch')"
    />

    <!-- Main Graph Viewport Virtual Scroll Container -->
    <div
      ref="scrollContainerRef"
      class="relative flex-1 overflow-auto bg-[var(--gv-surface)]"
      @scroll.passive="handleScroll"
    >
      <!-- Loading Skeleton -->
      <div v-if="props.isLoading" class="p-3 space-y-2">
        <div
          v-for="i in 10"
          :key="`graph-skeleton-${i}`"
          class="flex items-center gap-3 h-7 animate-pulse px-2"
        >
          <USkeleton class="h-3 w-3 rounded-full" />
          <USkeleton class="h-3 w-14 rounded" />
          <USkeleton class="h-3 flex-1 rounded" />
          <USkeleton class="h-3 w-20 rounded" />
          <USkeleton class="h-3 w-12 rounded" />
        </div>
      </div>

      <!-- Empty / No Commits State -->
      <div
        v-else-if="filteredNodes.length === 0"
        class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted"
      >
        <UIcon name="i-lucide-git-commit" class="h-8 w-8 text-muted/60" />
        <span class="text-xs">{{ t('git.noCommitsFound') }}</span>
      </div>

      <!-- Virtualized Scroll Area -->
      <div
        v-else
        class="relative min-w-fit"
        :style="{ height: `${totalHeight}px` }"
      >
        <!-- Virtual Window translated to scroll offset -->
        <div
          class="absolute top-0 start-0 w-full"
          :style="{ transform: `translateY(${offsetY}px)` }"
        >
          <!-- SVG Canvas for DAG lanes and edges (culled to visible range) -->
          <CommitGraphSvgCanvas
            :all-nodes="filteredNodes"
            :start-index="startIndex"
            :end-index="endIndex"
            :selected-commit-hash="props.selectedCommitHash"
            :row-height="ROW_HEIGHT"
            :lane-width="LANE_WIDTH"
            :padding-left="PADDING_LEFT"
            :graph-width="graphWidth"
          />

          <!-- Row items (culled to visible range) -->
          <div class="relative z-0">
            <CommitGraphRow
              v-for="(node, relativeIndex) in visibleNodes"
              :key="node.commit.hash"
              :node="node"
              :index="startIndex + relativeIndex"
              :is-selected="node.commit.hash === props.selectedCommitHash"
              :is-comparison-base="node.commit.hash === props.comparisonBaseHash"
              :active-selected-hash="props.selectedCommitHash"
              :graph-width="graphWidth"
              :row-height="ROW_HEIGHT"
              @select-commit="emit('selectCommit', $event)"
              @compare-commits="(base, target) => emit('compareCommits', base, target)"
              @create-branch-at-commit="emit('createBranchAtCommit', $event)"
              @checkout-commit="emit('checkoutCommit', $event)"
              @cherry-pick-commit="emit('cherryPickCommit', $event)"
              @revert-commit="emit('revertCommit', $event)"
              @reset-to-commit="(hash, mode) => emit('resetToCommit', hash, mode)"
              @rebase-interactive="emit('rebaseInteractive', $event)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
