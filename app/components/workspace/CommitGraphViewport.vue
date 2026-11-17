<script setup lang="ts">
import type { CommitGraphHistoryResponse, CommitGraphNode } from '~/types/repository'
import { formatRelativeTime } from '~/utils/dateFormat'

const props = defineProps<{
  graph: CommitGraphHistoryResponse | null
  selectedCommitHash: string | null
  isLoading: boolean
  isOperating?: boolean
}>()

const emit = defineEmits<{
  selectCommit: [commitHash: string]
  refresh: []
  push: []
  pull: []
  fetch: []
  createBranch: []
}>()

const { t } = useI18n()
const searchQuery = ref('')

const ROW_HEIGHT = 36
const LANE_WIDTH = 18
const PADDING_LEFT = 16

const filteredNodes = computed(() => {
  if (!props.graph || !props.graph.nodes) return []
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return props.graph.nodes

  return props.graph.nodes.filter(node => {
    const commit = node.commit
    return (
      commit.hash.toLowerCase().includes(query) ||
      commit.subject.toLowerCase().includes(query) ||
      commit.author_name.toLowerCase().includes(query) ||
      commit.decorations.some(d => d.name.toLowerCase().includes(query))
    )
  })
})

const graphWidth = computed(() => {
  const maxLane = props.graph?.max_lane_index || 0
  return Math.max((maxLane + 1) * LANE_WIDTH + PADDING_LEFT * 2, 60)
})

const svgHeight = computed(() => {
  return filteredNodes.value.length * ROW_HEIGHT
})

function getLaneX(lane: number): number {
  return PADDING_LEFT + lane * LANE_WIDTH
}

function getNodeY(index: number): number {
  return index * ROW_HEIGHT + ROW_HEIGHT / 2
}

interface ComputedEdgePath {
  id: string
  pathData: string
  color: string
  isMerge: boolean
}

const computedEdges = computed<ComputedEdgePath[]>(() => {
  const nodes = filteredNodes.value
  const edges: ComputedEdgePath[] = []
  const nodeIndexMap = new Map<string, number>()

  nodes.forEach((node, index) => {
    nodeIndexMap.set(node.commit.hash, index)
  })

  nodes.forEach((node, sourceIndex) => {
    const x1 = getLaneX(node.lane)
    const y1 = getNodeY(sourceIndex)

    node.edges.forEach((edge, edgeIndex) => {
      const targetIndex = nodeIndexMap.get(edge.parent_hash)
      const edgeLane = edge.is_merge_edge ? edge.to_lane : node.lane
      const color = `var(--gv-lane-${edgeLane % 8})`

      if (targetIndex !== undefined) {
        const x2 = getLaneX(edge.to_lane)
        const y2 = getNodeY(targetIndex)

        let pathData = ''
        if (x1 === x2) {
          pathData = `M ${x1} ${y1} L ${x2} ${y2}`
        } else {
          // Smooth S-curve transition between lanes
          const deltaY = y2 - y1
          const controlY1 = y1 + Math.min(deltaY * 0.5, 24)
          const controlY2 = y2 - Math.min(deltaY * 0.5, 24)
          pathData = `M ${x1} ${y1} C ${x1} ${controlY1}, ${x2} ${controlY2}, ${x2} ${y2}`
        }

        edges.push({
          id: `edge-${node.commit.hash}-${edge.parent_hash}-${edgeIndex}`,
          pathData,
          color,
          isMerge: edge.is_merge_edge,
        })
      } else {
        // Parent is outside current window: fade out downwards
        const x2 = getLaneX(edge.to_lane)
        const y2 = y1 + ROW_HEIGHT
        const pathData = x1 === x2
          ? `M ${x1} ${y1} L ${x2} ${y2}`
          : `M ${x1} ${y1} C ${x1} ${y1 + 14}, ${x2} ${y1 + 14}, ${x2} ${y2}`

        edges.push({
          id: `edge-${node.commit.hash}-unloaded-${edgeIndex}`,
          pathData,
          color,
          isMerge: edge.is_merge_edge,
        })
      }
    })
  })

  return edges
})

function handleRowClick(commitHash: string): void {
  emit('selectCommit', commitHash)
}
</script>

<template>
  <div class="gv-island-panel flex h-full flex-col overflow-hidden select-none">
    <!-- Graph Operation & Filter Toolbar -->
    <div class="flex h-10 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3 gap-2">
      <div class="flex items-center gap-1.5">
        <span class="text-xs font-semibold text-highlighted uppercase tracking-wider flex items-center gap-1.5 me-1">
          <UIcon name="i-lucide-git-commit-horizontal" class="h-4 w-4 text-primary" />
          {{ t('navigation.graph') }}
        </span>
        <UBadge
          v-if="graph?.total_commits_count !== undefined"
          size="xs"
          color="neutral"
          variant="subtle"
          class="font-mono text-[10px] me-2"
        >
          {{ graph.total_commits_count }}
        </UBadge>

        <div class="h-3.5 w-px bg-default/80 me-1" aria-hidden="true" />

        <!-- Remote Git Actions -->
        <UButton
          size="xs"
          color="primary"
          variant="solid"
          icon="i-lucide-arrow-up-circle"
          :loading="isOperating"
          :label="t('git.push')"
          @click="emit('push')"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-arrow-down-circle"
          :loading="isOperating"
          :label="t('git.pull')"
          @click="emit('pull')"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-refresh-cw"
          :loading="isOperating"
          :label="t('git.fetch')"
          @click="emit('fetch')"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-git-branch-plus"
          :label="t('git.createBranch')"
          @click="emit('createBranch')"
        />
      </div>

      <div class="flex items-center gap-2">
        <UInput
          v-model="searchQuery"
          size="xs"
          icon="i-lucide-search"
          placeholder="Search commits, branches, authors..."
          class="w-48 sm:w-64"
        />

        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-refresh-cw"
          :title="t('common.refresh')"
          :loading="isLoading"
          @click="emit('refresh')"
        />
      </div>
    </div>

    <!-- Main Commit Table with Integrated SVG Graph Column -->
    <div class="flex-1 overflow-auto relative">
      <!-- Loading State -->
      <div v-if="isLoading && (!graph || graph.nodes.length === 0)" class="flex h-full items-center justify-center gap-2 text-muted">
        <UIcon name="i-lucide-loader-circle" class="h-6 w-6 animate-spin text-primary" />
        <span class="text-xs">{{ t('common.loading') }}</span>
      </div>

      <!-- Empty State -->
      <div v-else-if="filteredNodes.length === 0" class="flex h-full flex-col items-center justify-center gap-2 p-6 text-muted">
        <UIcon name="i-lucide-git-commit" class="h-8 w-8 text-muted/60" />
        <p class="text-xs font-medium text-highlighted">{{ t('common.empty') }}</p>
      </div>

      <!-- Graph & Table Data -->
      <div v-else class="min-w-full inline-block align-top">
        <table class="w-full text-start text-xs border-collapse">
          <thead class="sticky top-0 z-20 bg-muted/90 backdrop-blur-md border-b border-default/70 text-[11px] text-muted select-none">
            <tr>
              <th
                class="py-1.5 px-2 text-start font-medium"
                :style="{ width: `${graphWidth}px`, minWidth: `${graphWidth}px` }"
              >
                Graph
              </th>
              <th class="py-1.5 px-2 text-start font-medium">Description</th>
              <th class="py-1.5 px-2 text-start font-medium w-32">Author</th>
              <th class="py-1.5 px-2 text-start font-medium w-24">Date</th>
              <th class="py-1.5 pe-3 ps-2 text-end font-medium w-20">SHA</th>
            </tr>
          </thead>

          <tbody class="divide-y divide-default/30 font-sans text-xs relative">
            <tr
              v-for="(node, index) in filteredNodes"
              :key="node.commit.hash"
              class="cursor-pointer transition-colors"
              :class="selectedCommitHash === node.commit.hash
                ? 'bg-primary/15 hover:bg-primary/20'
                : 'hover:bg-muted/40'"
              :style="{ height: `${ROW_HEIGHT}px` }"
              @click="handleRowClick(node.commit.hash)"
            >
              <!-- Cell 1: SVG Graph DAG Track -->
              <td class="p-0 relative" :style="{ width: `${graphWidth}px`, minWidth: `${graphWidth}px` }">
                <!-- Render SVG segment for this row -->
                <svg
                  class="absolute top-0 start-0 pointer-events-none"
                  :width="graphWidth"
                  :height="ROW_HEIGHT"
                  :viewBox="`0 ${index * ROW_HEIGHT} ${graphWidth} ${ROW_HEIGHT}`"
                >
                  <!-- Edges intersecting this row segment -->
                  <g class="stroke-2 fill-none" stroke-linecap="round" stroke-linejoin="round">
                    <path
                      v-for="edge in computedEdges"
                      :key="edge.id"
                      :d="edge.pathData"
                      :stroke="edge.color"
                      :stroke-dasharray="edge.isMerge ? '3 3' : 'none'"
                      class="opacity-80"
                    />
                  </g>

                  <!-- Node Circle -->
                  <g>
                    <!-- HEAD Outer Ring -->
                    <circle
                      v-if="node.is_head"
                      :cx="getLaneX(node.lane)"
                      :cy="getNodeY(index)"
                      r="7.5"
                      fill="none"
                      stroke="var(--gv-primary)"
                      stroke-width="1.5"
                      class="animate-pulse"
                    />

                    <!-- Main Commit Node -->
                    <circle
                      :cx="getLaneX(node.lane)"
                      :cy="getNodeY(index)"
                      :r="node.is_merge ? 5.5 : 4.5"
                      :fill="`var(--gv-lane-${node.lane % 8})`"
                      stroke="var(--gv-surface)"
                      stroke-width="2"
                      class="transition-transform group-hover:scale-125"
                    />

                    <!-- Merge Center Indicator Dot -->
                    <circle
                      v-if="node.is_merge"
                      :cx="getLaneX(node.lane)"
                      :cy="getNodeY(index)"
                      r="1.5"
                      fill="var(--gv-surface)"
                    />
                  </g>
                </svg>
              </td>

              <!-- Cell 2: Commit Subject & Ref Decorations -->
              <td class="py-1 px-2 font-medium text-highlighted max-w-md truncate">
                <div class="flex items-center gap-1.5 min-w-0">
                  <!-- Badges for Decorations (Branches, Tags, HEAD) -->
                  <template v-for="decoration in node.commit.decorations" :key="decoration.name">
                    <!-- HEAD Badge -->
                    <UBadge
                      v-if="decoration.reference_type === 'head'"
                      size="xs"
                      color="primary"
                      variant="solid"
                      class="font-mono text-[10px] shrink-0"
                    >
                      <UIcon name="i-lucide-check" class="h-2.5 w-2.5 me-0.5" />
                      HEAD
                    </UBadge>

                    <!-- Local Branch Badge -->
                    <UBadge
                      v-else-if="decoration.reference_type === 'localbranch'"
                      size="xs"
                      color="primary"
                      variant="subtle"
                      class="font-mono text-[10px] shrink-0"
                    >
                      <UIcon name="i-lucide-git-branch" class="h-2.5 w-2.5 me-0.5" />
                      {{ decoration.name }}
                    </UBadge>

                    <!-- Remote Branch Badge -->
                    <UBadge
                      v-else-if="decoration.reference_type === 'remotebranch'"
                      size="xs"
                      color="neutral"
                      variant="outline"
                      class="font-mono text-[10px] shrink-0"
                    >
                      <UIcon name="i-lucide-cloud" class="h-2.5 w-2.5 me-0.5" />
                      {{ decoration.name }}
                    </UBadge>

                    <!-- Tag Badge -->
                    <UBadge
                      v-else-if="decoration.reference_type === 'tag'"
                      size="xs"
                      color="secondary"
                      variant="subtle"
                      class="font-mono text-[10px] shrink-0"
                    >
                      <UIcon name="i-lucide-tag" class="h-2.5 w-2.5 me-0.5" />
                      {{ decoration.name }}
                    </UBadge>
                  </template>

                  <!-- Subject text -->
                  <span class="truncate font-sans" :title="node.commit.subject">
                    {{ node.commit.subject }}
                  </span>
                </div>
              </td>

              <!-- Cell 3: Author -->
              <td class="py-1 px-2 text-toned whitespace-nowrap truncate font-sans" :title="node.commit.author_name">
                {{ node.commit.author_name }}
              </td>

              <!-- Cell 4: Relative Date -->
              <td class="py-1 px-2 text-muted whitespace-nowrap text-[11px] font-sans">
                {{ formatRelativeTime(node.commit.author_timestamp) }}
              </td>

              <!-- Cell 5: Short SHA -->
              <td class="py-1 pe-3 ps-2 text-end text-muted whitespace-nowrap font-mono text-[11px]">
                {{ node.commit.hash.slice(0, 7) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
