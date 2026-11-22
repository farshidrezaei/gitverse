import { describe, expect, it } from 'vitest'
import { useActionGuard } from '../../app/composables/useActionGuard'
import type { GitPreflightAction } from '../../app/composables/useActionGuard'
import type { DetailedRepositoryStatus, GitWorktreeSummary } from '../../app/types/repository'

function buildStatus(options: {
  branch?: string | null
  isDetached?: boolean
  operationState?: DetailedRepositoryStatus['operation_state']
  staged?: number
  unstaged?: number
  conflicted?: number
  hasHeadCommit?: boolean
}): DetailedRepositoryStatus {
  return {
    branch_name: options.isDetached ? null : (options.branch ?? 'main'),
    head_commit_hash: options.hasHeadCommit === false ? null : 'a1b2c3d4e5f67890',
    upstream_branch_name: options.isDetached ? null : `origin/${options.branch ?? 'main'}`,
    ahead_count: 0,
    behind_count: 0,
    is_detached_head: Boolean(options.isDetached),
    staged_files: Array.from({ length: options.staged ?? 0 }, (_, i) => ({
      path: `staged_${i}.txt`,
      original_path: null,
      status: 'modified',
      is_staged: true,
    })),
    unstaged_files: Array.from({ length: options.unstaged ?? 0 }, (_, i) => ({
      path: `unstaged_${i}.txt`,
      original_path: null,
      status: 'modified',
      is_staged: false,
    })),
    conflicted_files: Array.from({ length: options.conflicted ?? 0 }, (_, i) => ({
      path: `conflicted_${i}.txt`,
      original_path: null,
      status: 'conflicted',
      is_staged: false,
    })),
    operation_state: options.operationState ?? 'none',
  }
}

describe('Table-Driven Action Guard Matrix', () => {
  const { evaluateAction } = useActionGuard()

  interface ActionMatrixTestCase {
    action: GitPreflightAction
    stateLabel: string
    status: DetailedRepositoryStatus
    targetBranch?: string
    commitMessage?: string
    isAmend?: boolean
    worktrees?: GitWorktreeSummary[]
    canExecute: boolean
    expectedReasonKey?: string
  }

  // Baseline states
  const cleanAttached = buildStatus({})
  const dirtyUnstaged = buildStatus({ unstaged: 2 })
  const dirtyStaged = buildStatus({ staged: 3 })
  const conflictedState = buildStatus({ conflicted: 1 })
  const midMergeState = buildStatus({ operationState: 'merge' })
  const midRebaseState = buildStatus({ operationState: 'rebase' })
  const midCherryPickState = buildStatus({ operationState: 'cherry_pick' })
  const midRevertState = buildStatus({ operationState: 'revert' })

  const matrixCases: ActionMatrixTestCase[] = [
    // --- COMMIT ACTION ---
    { action: 'commit', stateLabel: 'clean tree (no staged)', status: cleanAttached, canExecute: false, expectedReasonKey: 'preflight.commit.empty_staged_changes' },
    { action: 'commit', stateLabel: 'dirty unstaged only', status: dirtyUnstaged, canExecute: false, expectedReasonKey: 'preflight.commit.empty_staged_changes' },
    { action: 'commit', stateLabel: 'dirty staged available', status: dirtyStaged, canExecute: true },
    { action: 'commit', stateLabel: 'amend commit without staged', status: cleanAttached, isAmend: true, canExecute: true },
    { action: 'commit', stateLabel: 'conflicted active', status: conflictedState, canExecute: false, expectedReasonKey: 'preflight.commit.unresolved_conflicts' },
    { action: 'commit', stateLabel: 'empty commit message provided', status: dirtyStaged, commitMessage: '   ', canExecute: false, expectedReasonKey: 'preflight.commit.empty_commit_message' },

    // --- REBASE ACTION ---
    { action: 'rebase', stateLabel: 'clean tree onto other branch', status: cleanAttached, targetBranch: 'develop', canExecute: true },
    { action: 'rebase', stateLabel: 'clean tree onto self', status: cleanAttached, targetBranch: 'main', canExecute: false, expectedReasonKey: 'preflight.rebase.cannot_rebase_onto_self' },
    { action: 'rebase', stateLabel: 'dirty unstaged tree', status: dirtyUnstaged, targetBranch: 'develop', canExecute: false, expectedReasonKey: 'preflight.rebase.dirty_working_tree' },
    { action: 'rebase', stateLabel: 'dirty staged tree', status: dirtyStaged, targetBranch: 'develop', canExecute: false, expectedReasonKey: 'preflight.rebase.dirty_working_tree' },
    { action: 'rebase', stateLabel: 'mid-merge in progress', status: midMergeState, targetBranch: 'develop', canExecute: false, expectedReasonKey: 'preflight.rebase.active_operation_in_progress' },
    { action: 'rebase', stateLabel: 'mid-rebase in progress', status: midRebaseState, targetBranch: 'develop', canExecute: false, expectedReasonKey: 'preflight.rebase.active_operation_in_progress' },

    // --- MERGE ACTION ---
    { action: 'merge', stateLabel: 'clean tree into branch', status: cleanAttached, targetBranch: 'feature/login', canExecute: true },
    { action: 'merge', stateLabel: 'merge branch into self', status: cleanAttached, targetBranch: 'main', canExecute: false, expectedReasonKey: 'preflight.merge.cannot_merge_into_self' },
    { action: 'merge', stateLabel: 'dirty tree during merge', status: dirtyUnstaged, targetBranch: 'feature/login', canExecute: false, expectedReasonKey: 'preflight.merge.dirty_working_tree' },
    { action: 'merge', stateLabel: 'mid-merge operation in progress', status: midMergeState, targetBranch: 'feature/login', canExecute: false, expectedReasonKey: 'preflight.merge.active_operation_in_progress' },

    // --- CHECKOUT ACTION ---
    { action: 'checkout', stateLabel: 'clean tree to another branch', status: cleanAttached, targetBranch: 'feature', canExecute: true },
    { action: 'checkout', stateLabel: 'checkout current branch', status: cleanAttached, targetBranch: 'main', canExecute: false, expectedReasonKey: 'preflight.checkout.already_on_target_branch' },
    {
      action: 'checkout',
      stateLabel: 'branch in another worktree',
      status: cleanAttached,
      targetBranch: 'hotfix',
      worktrees: [{ path: '/tmp/wt', branch_name: 'hotfix', head_hash: '123', is_main: false, is_locked: false, is_prunable: false }],
      canExecute: false,
      expectedReasonKey: 'preflight.checkout.branch_in_another_worktree',
    },

    // --- DELETE BRANCH ACTION ---
    { action: 'delete_branch', stateLabel: 'delete other branch', status: cleanAttached, targetBranch: 'feature/obsolete', canExecute: true },
    { action: 'delete_branch', stateLabel: 'delete current active branch', status: cleanAttached, targetBranch: 'main', canExecute: false, expectedReasonKey: 'preflight.branch.cannot_delete_active_branch' },
    {
      action: 'delete_branch',
      stateLabel: 'delete branch locked in worktree',
      status: cleanAttached,
      targetBranch: 'feature/worktree',
      worktrees: [{ path: '/tmp/wt', branch_name: 'feature/worktree', head_hash: '123', is_main: false, is_locked: false, is_prunable: false }],
      canExecute: false,
      expectedReasonKey: 'preflight.branch.cannot_delete_worktree_branch',
    },

    // --- STASH SAVE ACTION ---
    { action: 'stash_save', stateLabel: 'dirty unstaged tree', status: dirtyUnstaged, canExecute: true },
    { action: 'stash_save', stateLabel: 'dirty staged tree', status: dirtyStaged, canExecute: true },
    { action: 'stash_save', stateLabel: 'completely clean tree', status: cleanAttached, canExecute: false, expectedReasonKey: 'preflight.stash.no_local_changes' },

    // --- ABORT & CONTINUE OPERATIONS ---
    { action: 'abort_operation', stateLabel: 'mid-merge abort', status: midMergeState, canExecute: true },
    { action: 'abort_operation', stateLabel: 'no operation active abort', status: cleanAttached, canExecute: false, expectedReasonKey: 'preflight.operation.no_active_operation' },
    { action: 'continue_operation', stateLabel: 'mid-rebase without conflicts', status: midRebaseState, canExecute: true },
    { action: 'continue_operation', stateLabel: 'mid-rebase with unresolved conflicts', status: buildStatus({ operationState: 'rebase', conflicted: 1 }), canExecute: false, expectedReasonKey: 'preflight.operation.unresolved_conflicts' },
    { action: 'continue_operation', stateLabel: 'no operation active continue', status: cleanAttached, canExecute: false, expectedReasonKey: 'preflight.operation.no_active_operation' },

    // --- CHERRY-PICK & REVERT ACTIONS ---
    { action: 'cherry_pick', stateLabel: 'clean tree cherry-pick', status: cleanAttached, canExecute: true },
    { action: 'cherry_pick', stateLabel: 'dirty tree cherry-pick', status: dirtyUnstaged, canExecute: false, expectedReasonKey: 'preflight.cherry_pick.dirty_working_tree' },
    { action: 'cherry_pick', stateLabel: 'mid-rebase cherry-pick', status: midRebaseState, canExecute: false, expectedReasonKey: 'preflight.cherry_pick.active_operation_in_progress' },
    { action: 'revert', stateLabel: 'clean tree revert', status: cleanAttached, canExecute: true },
    { action: 'revert', stateLabel: 'dirty tree revert', status: dirtyUnstaged, canExecute: false, expectedReasonKey: 'preflight.revert.dirty_working_tree' },
    { action: 'revert', stateLabel: 'mid-merge revert', status: midMergeState, canExecute: false, expectedReasonKey: 'preflight.revert.active_operation_in_progress' },
  ]

  it.each(matrixCases)(
    'Action "$action" on [$stateLabel] -> canExecute: $canExecute',
    ({ action, status, targetBranch, commitMessage, isAmend, worktrees, canExecute, expectedReasonKey }) => {
      const evaluation = evaluateAction(action, {
        overrideStatus: status,
        targetBranch,
        commitMessage,
        isAmend,
        worktrees,
      })
      expect(evaluation.canExecute).toBe(canExecute)
      if (expectedReasonKey) {
        expect(evaluation.reasonKey).toBe(expectedReasonKey)
      }
    }
  )
})
