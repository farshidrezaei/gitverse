import { describe, expect, it } from 'vitest'
import { useActionGuard } from '~/composables/useActionGuard'
import type { DetailedRepositoryStatus, GitWorktreeSummary } from '~/types/repository'

function createStatus(
  branch: string,
  operationState: DetailedRepositoryStatus['operation_state'] = 'none',
  stagedCount = 0,
  unstagedCount = 0,
  conflictedCount = 0
): DetailedRepositoryStatus {
  return {
    branch_name: branch,
    head_commit_hash: '1234567890abcdef',
    upstream_branch_name: `origin/${branch}`,
    ahead_count: 0,
    behind_count: 0,
    is_detached_head: false,
    staged_files: Array.from({ length: stagedCount }, (_, index) => ({
      path: `staged_${index}.txt`,
      original_path: null,
      status: 'modified',
      is_staged: true,
    })),
    unstaged_files: Array.from({ length: unstagedCount }, (_, index) => ({
      path: `unstaged_${index}.txt`,
      original_path: null,
      status: 'modified',
      is_staged: false,
    })),
    conflicted_files: Array.from({ length: conflictedCount }, (_, index) => ({
      path: `conflicted_${index}.txt`,
      original_path: null,
      status: 'conflicted',
      is_staged: false,
    })),
    operation_state: operationState,
  }
}

describe('useActionGuard', () => {
  const { evaluateAction } = useActionGuard()

  it('evaluates rebase feasibility', () => {
    const cleanStatus = createStatus('feature')
    expect(
      evaluateAction('rebase', {
        overrideStatus: cleanStatus,
        targetBranch: 'main',
      })
    ).toEqual({ canExecute: true })

    // Cannot rebase onto self
    expect(
      evaluateAction('rebase', {
        overrideStatus: cleanStatus,
        targetBranch: 'feature',
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.rebase.cannot_rebase_onto_self',
    })

    // Cannot rebase with dirty working tree
    const dirtyStatus = createStatus('feature', 'none', 0, 1)
    expect(
      evaluateAction('rebase', {
        overrideStatus: dirtyStatus,
        targetBranch: 'main',
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.rebase.dirty_working_tree',
      suggestedResolution: 'stash_and_proceed',
    })

    // Cannot rebase while already rebasing
    const rebasingStatus = createStatus('feature', 'rebasing')
    expect(
      evaluateAction('rebase', {
        overrideStatus: rebasingStatus,
        targetBranch: 'main',
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.rebase.active_operation_in_progress',
    })
  })

  it('evaluates merge feasibility', () => {
    const cleanStatus = createStatus('main')
    expect(
      evaluateAction('merge', {
        overrideStatus: cleanStatus,
        targetBranch: 'feature',
      })
    ).toEqual({ canExecute: true })

    // Cannot merge into self
    expect(
      evaluateAction('merge', {
        overrideStatus: cleanStatus,
        targetBranch: 'main',
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.merge.cannot_merge_into_self',
    })
  })

  it('evaluates checkout feasibility', () => {
    const status = createStatus('feature-a')
    const worktrees: GitWorktreeSummary[] = [
      {
        directory_path: '/repo',
        head_commit_hash: '123',
        branch_name: 'feature-a',
        is_bare: false,
        is_locked: false,
        lock_reason: null,
        is_prunable: false,
        is_main: true,
      },
      {
        directory_path: '/repo-wt',
        head_commit_hash: '456',
        branch_name: 'feature-b',
        is_bare: false,
        is_locked: false,
        lock_reason: null,
        is_prunable: false,
        is_main: false,
      },
    ]

    // Already on target branch
    expect(
      evaluateAction('checkout', {
        overrideStatus: status,
        targetBranch: 'feature-a',
        worktrees,
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.checkout.already_on_target_branch',
    })

    // In another worktree
    expect(
      evaluateAction('checkout', {
        overrideStatus: status,
        targetBranch: 'feature-b',
        worktrees,
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.checkout.branch_in_another_worktree',
    })

    // Free branch
    expect(
      evaluateAction('checkout', {
        overrideStatus: status,
        targetBranch: 'feature-c',
        worktrees,
      })
    ).toEqual({ canExecute: true })
  })

  it('evaluates commit feasibility', () => {
    const cleanStatus = createStatus('main')

    // Empty message
    expect(
      evaluateAction('commit', {
        overrideStatus: cleanStatus,
        commitMessage: '   ',
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.commit.empty_commit_message',
    })

    // Empty staged files
    expect(
      evaluateAction('commit', {
        overrideStatus: cleanStatus,
        commitMessage: 'feat: add test',
        isAmend: false,
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.commit.empty_staged_changes',
      suggestedResolution: 'stage_all_and_proceed',
    })

    // Amend with empty staged files allowed
    expect(
      evaluateAction('commit', {
        overrideStatus: cleanStatus,
        commitMessage: 'feat: add test',
        isAmend: true,
      })
    ).toEqual({ canExecute: true })

    // Unresolved conflicts
    const conflictStatus = createStatus('main', 'merging', 1, 0, 1)
    expect(
      evaluateAction('commit', {
        overrideStatus: conflictStatus,
        commitMessage: 'feat: conflict fix',
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.commit.unresolved_conflicts',
    })
  })

  it('evaluates stash save feasibility', () => {
    const cleanStatus = createStatus('main')
    expect(
      evaluateAction('stash_save', {
        overrideStatus: cleanStatus,
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.stash.no_local_changes',
    })

    const dirtyStatus = createStatus('main', 'none', 1, 0)
    expect(
      evaluateAction('stash_save', {
        overrideStatus: dirtyStatus,
      })
    ).toEqual({ canExecute: true })
  })

  it('evaluates abort and continue operations feasibility', () => {
    const cleanStatus = createStatus('main')
    expect(
      evaluateAction('abort_operation', {
        overrideStatus: cleanStatus,
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.operation.no_active_operation',
    })

    const rebasingStatus = createStatus('main', 'rebasing')
    expect(
      evaluateAction('abort_operation', {
        overrideStatus: rebasingStatus,
      })
    ).toEqual({ canExecute: true })
    expect(
      evaluateAction('continue_operation', {
        overrideStatus: rebasingStatus,
      })
    ).toEqual({ canExecute: true })

    const conflictedStatus = createStatus('main', 'rebasing', 0, 0, 1)
    expect(
      evaluateAction('continue_operation', {
        overrideStatus: conflictedStatus,
      })
    ).toEqual({
      canExecute: false,
      reasonKey: 'preflight.operation.unresolved_conflicts',
    })
  })
})
