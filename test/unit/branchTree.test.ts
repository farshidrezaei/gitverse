import { describe, expect, it } from 'vitest'
import { buildBranchTree, buildRemoteBranchTree } from '~/utils/branchTree'
import type { BranchSummary, RemoteBranchSummary } from '~/types/repository'

describe('branchTree - buildBranchTree', () => {
  it('correctly handles flat root-level branches', () => {
    const branches: BranchSummary[] = [
      { name: 'main', is_head: true, upstream_name: 'origin/main', ahead_count: 0, behind_count: 0, head_commit_hash: '1234' },
      { name: 'develop', is_head: false, upstream_name: null, ahead_count: 2, behind_count: 0, head_commit_hash: '5678' },
    ]

    const tree = buildBranchTree(branches)
    expect(tree).toHaveLength(2)
    expect(tree[0]?.name).toBe('develop')
    expect(tree[0]?.isFolder).toBe(false)
    expect(tree[1]?.name).toBe('main')
    expect(tree[1]?.isFolder).toBe(false)
    expect(tree[1]?.hasHeadBranch).toBe(true)
  })

  it('correctly builds hierarchical folders and propagates hasHeadBranch status', () => {
    const branches: BranchSummary[] = [
      { name: 'main', is_head: false, upstream_name: null, ahead_count: 0, behind_count: 0, head_commit_hash: '111' },
      { name: 'feature/auth/login', is_head: true, upstream_name: null, ahead_count: 1, behind_count: 0, head_commit_hash: '222' },
      { name: 'feature/auth/logout', is_head: false, upstream_name: null, ahead_count: 0, behind_count: 0, head_commit_hash: '333' },
      { name: 'bugfix/issue-12', is_head: false, upstream_name: null, ahead_count: 0, behind_count: 1, head_commit_hash: '444' },
    ]

    const tree = buildBranchTree(branches)
    // Folders come first: 'bugfix', 'feature', then leaf 'main'
    expect(tree).toHaveLength(3)

    // Bugfix folder
    expect(tree[0]?.name).toBe('bugfix')
    expect(tree[0]?.isFolder).toBe(true)
    expect(tree[0]?.children).toHaveLength(1)
    expect(tree[0]?.children[0]?.name).toBe('issue-12')
    expect(tree[0]?.children[0]?.fullBranchName).toBe('bugfix/issue-12')

    // Feature folder (contains auth folder)
    expect(tree[1]?.name).toBe('feature')
    expect(tree[1]?.isFolder).toBe(true)
    expect(tree[1]?.hasHeadBranch).toBe(true)
    expect(tree[1]?.children).toHaveLength(1)

    const authFolder = tree[1]?.children[0]
    expect(authFolder?.name).toBe('auth')
    expect(authFolder?.isFolder).toBe(true)
    expect(authFolder?.hasHeadBranch).toBe(true)
    expect(authFolder?.children).toHaveLength(2)
    expect(authFolder?.children[0]?.name).toBe('login')
    expect(authFolder?.children[0]?.fullBranchName).toBe('feature/auth/login')
    expect(authFolder?.children[0]?.branch?.is_head).toBe(true)

    // Root leaf
    expect(tree[2]?.name).toBe('main')
    expect(tree[2]?.isFolder).toBe(false)
  })
})

describe('branchTree - buildRemoteBranchTree', () => {
  it('correctly nests remote branches under remote name folder', () => {
    const remoteBranches: RemoteBranchSummary[] = [
      { name: 'origin/main', remote_name: 'origin', head_commit_hash: 'aaa' },
      { name: 'origin/feature/login', remote_name: 'origin', head_commit_hash: 'bbb' },
    ]

    const tree = buildRemoteBranchTree(remoteBranches)
    expect(tree).toHaveLength(1)
    expect(tree[0]?.name).toBe('origin')
    expect(tree[0]?.isFolder).toBe(true)
    // Inside origin: feature folder first, then main
    expect(tree[0]?.children[0]?.name).toBe('feature')
    expect(tree[0]?.children[0]?.isFolder).toBe(true)
    expect(tree[0]?.children[1]?.name).toBe('main')
    expect(tree[0]?.children[1]?.isFolder).toBe(false)
  })

  it('correctly ensures folder status even if an origin entry exists before sub-branches', () => {
    const remoteBranches: RemoteBranchSummary[] = [
      { name: 'origin', remote_name: 'origin', head_commit_hash: '000' },
      { name: 'origin/main', remote_name: 'origin', head_commit_hash: 'aaa' },
      { name: 'origin/dev', remote_name: 'origin', head_commit_hash: 'bbb' },
    ]

    const tree = buildRemoteBranchTree(remoteBranches)
    expect(tree).toHaveLength(1)
    expect(tree[0]?.name).toBe('origin')
    expect(tree[0]?.isFolder).toBe(true)
    expect(tree[0]?.children.length).toBeGreaterThanOrEqual(2)
  })
})
