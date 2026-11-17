<script setup lang="ts">
import type { CommitGraphNode } from '~/types/repository'

interface CommitGraphSvgCanvasProps {
  allNodes: CommitGraphNode[]
  startIndex: number
  endIndex: number
  selectedCommitHash: string | null
  rowHeight: number
  laneWidth: number
  paddingLeft: number
  graphWidth: number
}

const props = defineProps<CommitGraphSvgCanvasProps>()

interface ComputedEdgePath {
  id: string
  pathData: string
  color: string
  isMerge: boolean
}

function getLaneX(lane: number): number {
  return props.paddingLeft + lane * props.laneWidth
}

function getNodeY(absoluteIndex: number): number {
  return (absoluteIndex - props.startIndex) * props.rowHeight + props.rowHeight / 2
}

const visibleHeight = computed(() => {
  return (props.endIndex - props.startIndex) * props.rowHeight
})

const visibleNodesWithIndex = computed(() => {
  const result: { node: CommitGraphNode; absoluteIndex: number }[] = []
  for (let i = props.startIndex; i < props.endIndex && i < props.allNodes.length; i++) {
    const node = props.allNodes[i]
    if (node) {
      result.push({ node, absoluteIndex: i })
    }
  }
  return result
})

const computedEdges = computed<ComputedEdgePath[]>(() => {
  const nodes = props.allNodes
  if (nodes.length === 0) return []

  const edges: ComputedEdgePath[] = []
  const nodeIndexMap = new Map<string, number>()

  nodes.forEach((node, index) => {
    nodeIndexMap.set(node.commit.hash, index)
  })

  // Cull edge computation to visible range plus overscan buffer
  const cullingStart = Math.max(0, props.startIndex - 5)
  const cullingEnd = Math.min(nodes.length, props.endIndex + 5)

  for (let sourceIndex = cullingStart; sourceIndex < cullingEnd; sourceIndex++) {
    const node = nodes[sourceIndex]
    if (!node) continue

    const x1 = getLaneX(node.lane)
    const y1 = getNodeY(sourceIndex)

    node.edges.forEach((edge, edgeIndex) => {
      const targetIndex = nodeIndexMap.get(edge.parent_hash)
      const edgeLane = edge.is_merge_edge ? edge.to_lane : node.lane
      const color = `var(--gv-lane-${edgeLane % 8})`

      if (targetIndex !== undefined) {
        // Only draw if either source or target is inside visible window plus margin
        const isEitherVisible =
          (sourceIndex >= props.startIndex - 2 && sourceIndex <= props.endIndex + 2) ||
          (targetIndex >= props.startIndex - 2 && targetIndex <= props.endIndex + 2)

        if (!isEitherVisible) return

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
      } else if (sourceIndex >= props.startIndex && sourceIndex <= props.endIndex) {
        // Parent is outside current window: fade out downwards
        const x2 = getLaneX(edge.to_lane)
        const y2 = y1 + props.rowHeight
        const pathData =
          x1 === x2
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
  }

  return edges
})
</script>

<template>
  <svg
    class="absolute top-0 start-0 pointer-events-none z-10 overflow-visible"
    :style="{ width: `${props.graphWidth}px`, height: `${visibleHeight}px` }"
  >
    <defs>
      <filter id="gv-node-glow" x="-20%" y="-20%" width="140%" height="140%">
        <feDropShadow dx="0" dy="0" stdDeviation="2" flood-color="var(--gv-primary)" flood-opacity="0.6" />
      </filter>
    </defs>

    <!-- DAG Edges -->
    <path
      v-for="edge in computedEdges"
      :key="edge.id"
      :d="edge.pathData"
      fill="none"
      :stroke="edge.color"
      :stroke-width="edge.isMerge ? 2 : 2.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      :stroke-dasharray="edge.id.includes('unloaded') ? '3 3' : 'none'"
      class="opacity-80 transition-all duration-150"
    />

    <!-- DAG Nodes (Culled to visible range) -->
    <g
      v-for="item in visibleNodesWithIndex"
      :key="`node-${item.node.commit.hash}`"
    >
      <!-- Glow halo for selected commit or HEAD -->
      <circle
        v-if="item.node.commit.hash === props.selectedCommitHash || item.node.is_head"
        :cx="getLaneX(item.node.lane)"
        :cy="getNodeY(item.absoluteIndex)"
        :r="item.node.is_head ? 7 : 6"
        fill="none"
        :stroke="item.node.is_head ? 'var(--gv-primary)' : `var(--gv-lane-${item.node.lane % 8})`"
        stroke-width="1.5"
        stroke-opacity="0.8"
        class="animate-pulse"
      />

      <!-- Node Outer Ring -->
      <circle
        :cx="getLaneX(item.node.lane)"
        :cy="getNodeY(item.absoluteIndex)"
        :r="item.node.is_merge ? 4.5 : 4"
        :fill="item.node.is_head ? 'var(--gv-primary)' : `var(--gv-lane-${item.node.lane % 8})`"
        stroke="var(--gv-surface)"
        stroke-width="1.5"
        class="transition-transform duration-100"
      />

      <!-- Inner Dot for merge commits -->
      <circle
        v-if="item.node.is_merge"
        :cx="getLaneX(item.node.lane)"
        :cy="getNodeY(item.absoluteIndex)"
        r="1.8"
        fill="var(--gv-surface)"
      />
    </g>
  </svg>
</template>
