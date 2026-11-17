import type { BranchSummary, RemoteBranchSummary } from '~/types/repository'

export interface BranchTreeNode {
  id: string
  name: string
  fullBranchName: string
  isFolder: boolean
  branch?: BranchSummary
  children: BranchTreeNode[]
  hasHeadBranch: boolean
}

/**
 * Builds a hierarchical tree structure from a flat array of local branch summaries.
 */
export function buildBranchTree(branches: BranchSummary[]): BranchTreeNode[] {
  const rootNodes: BranchTreeNode[] = []

  for (const branch of branches) {
    const segments = branch.name.split('/').filter(Boolean)
    let currentLevel = rootNodes
    let accumulatedPath = ''

    for (let index = 0; index < segments.length; index++) {
      const segment = segments[index]
      if (!segment) continue
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${segment}` : segment
      const isLast = index === segments.length - 1

      let existingNode = currentLevel.find(node => node.name === segment)

      if (!existingNode) {
        existingNode = {
          id: accumulatedPath,
          name: segment,
          fullBranchName: accumulatedPath,
          isFolder: !isLast,
          branch: isLast ? branch : undefined,
          children: [],
          hasHeadBranch: isLast ? branch.is_head : false,
        }
        currentLevel.push(existingNode)
      } else if (!isLast) {
        existingNode.isFolder = true
      }

      if (isLast && branch.is_head) {
        existingNode.hasHeadBranch = true
      }

      currentLevel = existingNode.children
    }
  }

  // Ensure any node with children is marked as a folder
  function ensureFolderStatus(nodes: BranchTreeNode[]): void {
    for (const node of nodes) {
      if (node.children.length > 0) {
        node.isFolder = true
        ensureFolderStatus(node.children)
      }
    }
  }
  ensureFolderStatus(rootNodes)

  // Update hasHeadBranch recursively for parent folder nodes
  function computeHeadStatus(nodes: BranchTreeNode[]): boolean {
    let hasHead = false
    for (const node of nodes) {
      if (node.isFolder) {
        node.hasHeadBranch = computeHeadStatus(node.children)
      }
      if (node.hasHeadBranch) {
        hasHead = true
      }
    }
    return hasHead
  }

  // Sort nodes: folders first, then alphabetical by name
  function sortNodes(nodes: BranchTreeNode[]): void {
    nodes.sort((nodeA, nodeB) => {
      if (nodeA.isFolder && !nodeB.isFolder) return -1
      if (!nodeA.isFolder && nodeB.isFolder) return 1
      return nodeA.name.localeCompare(nodeB.name)
    })
    for (const node of nodes) {
      if (node.isFolder) {
        sortNodes(node.children)
      }
    }
  }

  computeHeadStatus(rootNodes)
  sortNodes(rootNodes)
  return rootNodes
}

export interface RemoteBranchTreeNode {
  id: string
  name: string
  fullBranchName: string
  isFolder: boolean
  remoteBranch?: RemoteBranchSummary
  children: RemoteBranchTreeNode[]
}

/**
 * Builds a hierarchical tree structure from remote tracking branch summaries.
 */
export function buildRemoteBranchTree(remoteBranches: RemoteBranchSummary[]): RemoteBranchTreeNode[] {
  const rootNodes: RemoteBranchTreeNode[] = []

  for (const remoteBranch of remoteBranches) {
    const segments = remoteBranch.name.split('/').filter(Boolean)
    let currentLevel = rootNodes
    let accumulatedPath = ''

    for (let index = 0; index < segments.length; index++) {
      const segment = segments[index]
      if (!segment) continue
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${segment}` : segment
      const isLast = index === segments.length - 1

      let existingNode = currentLevel.find(node => node.name === segment)

      if (!existingNode) {
        existingNode = {
          id: accumulatedPath,
          name: segment,
          fullBranchName: accumulatedPath,
          isFolder: !isLast,
          remoteBranch: isLast ? remoteBranch : undefined,
          children: [],
        }
        currentLevel.push(existingNode)
      } else if (!isLast) {
        existingNode.isFolder = true
      }

      currentLevel = existingNode.children
    }
  }

  // Ensure any node with children is marked as a folder
  function ensureRemoteFolderStatus(nodes: RemoteBranchTreeNode[]): void {
    for (const node of nodes) {
      if (node.children.length > 0) {
        node.isFolder = true
        ensureRemoteFolderStatus(node.children)
      }
    }
  }
  ensureRemoteFolderStatus(rootNodes)

  // Sort nodes: folders first, then alphabetical
  function sortNodes(nodes: RemoteBranchTreeNode[]): void {
    nodes.sort((nodeA, nodeB) => {
      if (nodeA.isFolder && !nodeB.isFolder) return -1
      if (!nodeA.isFolder && nodeB.isFolder) return 1
      return nodeA.name.localeCompare(nodeB.name)
    })
    for (const node of nodes) {
      if (node.isFolder) {
        sortNodes(node.children)
      }
    }
  }

  sortNodes(rootNodes)
  return rootNodes
}
