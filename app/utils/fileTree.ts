import type { FileStatusEntry } from '~/types/repository'

export interface FileTreeNode<T = FileStatusEntry> {
  id: string
  name: string
  fullPath: string
  isDirectory: boolean
  fileEntry?: T
  children: FileTreeNode<T>[]
}

/**
 * Builds a nested hierarchical tree structure from a flat array of items containing a path.
 */
export function buildFileTree<T extends { path: string }>(files: T[]): FileTreeNode<T>[] {
  const rootNodes: FileTreeNode<T>[] = []

  for (const file of files) {
    const segments = file.path.split('/').filter(Boolean)
    let currentLevel = rootNodes
    let accumulatedPath = ''

    for (let index = 0; index < segments.length; index++) {
      const segment = segments[index]
      if (!segment) continue
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${segment}` : segment
      const isLast = index === segments.length - 1

      let existingNode = currentLevel.find((node) => node.name === segment)

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
  function sortNodes(nodes: FileTreeNode<T>[]): void {
    nodes.sort((firstNode, secondNode) => {
      if (firstNode.isDirectory && !secondNode.isDirectory) return -1
      if (!firstNode.isDirectory && secondNode.isDirectory) return 1
      return firstNode.name.localeCompare(secondNode.name)
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
