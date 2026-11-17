import type { FileStatusEntry } from '~/types/repository'

export interface FileTreeNode {
  id: string
  name: string
  fullPath: string
  isDirectory: boolean
  fileEntry?: FileStatusEntry
  children: FileTreeNode[]
}

/**
 * Builds a nested hierarchical tree structure from a flat array of FileStatusEntry.
 */
export function buildFileTree(files: FileStatusEntry[]): FileTreeNode[] {
  const rootNodes: FileTreeNode[] = []

  for (const file of files) {
    const segments = file.path.split('/').filter(Boolean)
    let currentLevel = rootNodes
    let accumulatedPath = ''

    for (let i = 0; i < segments.length; i++) {
      const segment = segments[i]
      if (!segment) continue
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${segment}` : segment
      const isLast = i === segments.length - 1

      let existingNode = currentLevel.find(node => node.name === segment)

      if (!existingNode) {
        existingNode = {
          id: accumulatedPath,
          name: segment,
          fullPath: accumulatedPath,
          isDirectory: !isLast,
          fileEntry: isLast ? file : undefined,
          children: [],
        }
        currentLevel.push(existingNode)
      }

      currentLevel = existingNode.children
    }
  }

  // Sort nodes: directories first, then alphabetically
  function sortNodes(nodes: FileTreeNode[]): void {
    nodes.sort((a, b) => {
      if (a.isDirectory && !b.isDirectory) return -1
      if (!a.isDirectory && b.isDirectory) return 1
      return a.name.localeCompare(b.name)
    })
    for (const node of nodes) {
      if (node.isDirectory) {
        sortNodes(node.children)
      }
    }
  }

  sortNodes(rootNodes)
  return rootNodes
}
