import { describe, expect, it } from 'vitest'
import { buildFileTree } from '~/utils/fileTree'
import type { CommitChangedFile, FileStatusEntry } from '~/types/repository'

describe('fileTree - buildFileTree', () => {
  it('correctly handles flat root-level files', () => {
    const files: FileStatusEntry[] = [
      { path: 'package.json', original_path: null, status: 'modified', is_staged: false },
      { path: 'README.md', original_path: null, status: 'added', is_staged: false },
    ]

    const tree = buildFileTree(files)
    expect(tree).toHaveLength(2)
    expect(tree[0]?.name).toBe('package.json')
    expect(tree[0]?.isDirectory).toBe(false)
    expect(tree[0]?.fullPath).toBe('package.json')
    expect(tree[1]?.name).toBe('README.md')
    expect(tree[1]?.isDirectory).toBe(false)
  })

  it('correctly builds hierarchical directories and sorts folders before files', () => {
    const files: CommitChangedFile[] = [
      { path: 'apps/desktop/src/main.ts', original_path: null, status: 'modified', additions: 10, deletions: 2 },
      { path: 'apps/desktop/package.json', original_path: null, status: 'modified', additions: 1, deletions: 0 },
      { path: 'docs/README.md', original_path: null, status: 'added', additions: 50, deletions: 0 },
      { path: 'root.txt', original_path: null, status: 'added', additions: 5, deletions: 0 },
    ]

    const tree = buildFileTree(files)

    // Top-level should have 2 directories ('apps', 'docs') and 1 file ('root.txt')
    expect(tree).toHaveLength(3)

    // Directories first
    expect(tree[0]?.name).toBe('apps')
    expect(tree[0]?.isDirectory).toBe(true)
    expect(tree[1]?.name).toBe('docs')
    expect(tree[1]?.isDirectory).toBe(true)
    expect(tree[2]?.name).toBe('root.txt')
    expect(tree[2]?.isDirectory).toBe(false)

    // Nested structure for 'apps'
    const appsNode = tree[0]
    expect(appsNode?.children).toHaveLength(1)
    const desktopNode = appsNode?.children[0]
    expect(desktopNode?.name).toBe('desktop')
    expect(desktopNode?.isDirectory).toBe(true)

    // Inside 'apps/desktop': 1 folder ('src') and 1 file ('package.json')
    expect(desktopNode?.children).toHaveLength(2)
    expect(desktopNode?.children[0]?.name).toBe('src')
    expect(desktopNode?.children[0]?.isDirectory).toBe(true)
    expect(desktopNode?.children[1]?.name).toBe('package.json')
    expect(desktopNode?.children[1]?.isDirectory).toBe(false)
    expect(desktopNode?.children[1]?.fileEntry?.additions).toBe(1)

    // Inside 'apps/desktop/src'
    const srcNode = desktopNode?.children[0]
    expect(srcNode?.children).toHaveLength(1)
    expect(srcNode?.children[0]?.name).toBe('main.ts')
    expect(srcNode?.children[0]?.isDirectory).toBe(false)
    expect(srcNode?.children[0]?.fileEntry?.additions).toBe(10)
    expect(srcNode?.children[0]?.fileEntry?.deletions).toBe(2)
  })

  it('handles empty input gracefully', () => {
    const tree = buildFileTree([])
    expect(tree).toEqual([])
  })
})
