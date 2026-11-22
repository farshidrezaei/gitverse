import type { DetailedRepositoryStatus, GitWorktreeSummary } from '~/types/repository'

export type GitPreflightAction =
  | 'rebase'
  | 'merge'
  | 'checkout'
  | 'commit'
  | 'delete_branch'
  | 'stash_save'
  | 'abort_operation'
  | 'continue_operation'
  | 'cherry_pick'
  | 'revert'

export type PreflightSuggestedResolution =
  | 'stash_and_proceed'
  | 'stage_all_and_proceed'

export interface PreflightActionContext {
  targetBranch?: string
  commitMessage?: string
  isAmend?: boolean
  worktrees?: GitWorktreeSummary[]
  overrideStatus?: DetailedRepositoryStatus | null
}

export interface PreflightCheckOutcome {
  canExecute: boolean
  reasonKey?: string
  suggestedResolution?: PreflightSuggestedResolution
}

export interface PreflightModalConfiguration {
  isOpen: boolean
  action: GitPreflightAction | null
  titleKey: string
  descriptionKey: string
  actionButtonKey: string
  isExecutingResolution: boolean
  onProceedWithResolution: () => Promise<void>
}

function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useActionGuard() {
  const repositoryStatusState = safeState<DetailedRepositoryStatus | null>(
    'workspace-repository-status',
    () => null
  )
  const worktreeListState = safeState<GitWorktreeSummary[]>(
    'workspace-worktree-list',
    () => []
  )

  const preflightModal = safeState<PreflightModalConfiguration>(
    'workspace-preflight-modal-configuration',
    () => ({
      isOpen: false,
      action: null,
      titleKey: '',
      descriptionKey: '',
      actionButtonKey: '',
      isExecutingResolution: false,
      onProceedWithResolution: async () => {},
    })
  )

  function evaluateAction(
    action: GitPreflightAction,
    context?: PreflightActionContext
  ): PreflightCheckOutcome {
    const status = context?.overrideStatus ?? repositoryStatusState.value
    const worktrees = context?.worktrees ?? worktreeListState.value

    if (!status) {
      return { canExecute: true }
    }

    const hasDirtyWorkingTree =
      status.staged_files.length > 0 || status.unstaged_files.length > 0
    const hasActiveOperation = status.operation_state !== 'none'

    switch (action) {
      case 'rebase': {
        if (hasActiveOperation) {
          return {
            canExecute: false,
            reasonKey: 'preflight.rebase.active_operation_in_progress',
          }
        }
        if (hasDirtyWorkingTree) {
          return {
            canExecute: false,
            reasonKey: 'preflight.rebase.dirty_working_tree',
            suggestedResolution: 'stash_and_proceed',
          }
        }
        if (
          context?.targetBranch &&
          status.branch_name === context.targetBranch
        ) {
          return {
            canExecute: false,
            reasonKey: 'preflight.rebase.cannot_rebase_onto_self',
          }
        }
        return { canExecute: true }
      }

      case 'merge': {
        if (hasActiveOperation) {
          return {
            canExecute: false,
            reasonKey: 'preflight.merge.active_operation_in_progress',
          }
        }
        if (hasDirtyWorkingTree) {
          return {
            canExecute: false,
            reasonKey: 'preflight.merge.dirty_working_tree',
            suggestedResolution: 'stash_and_proceed',
          }
        }
        if (
          context?.targetBranch &&
          status.branch_name === context.targetBranch
        ) {
          return {
            canExecute: false,
            reasonKey: 'preflight.merge.cannot_merge_into_self',
          }
        }
        return { canExecute: true }
      }

      case 'checkout': {
        if (
          context?.targetBranch &&
          status.branch_name === context.targetBranch &&
          !status.is_detached_head
        ) {
          return {
            canExecute: false,
            reasonKey: 'preflight.checkout.already_on_target_branch',
          }
        }
        const isBranchInAnotherWorktree = worktrees.some(
          (worktree) =>
            !worktree.is_main && worktree.branch_name === context?.targetBranch
        )
        if (isBranchInAnotherWorktree) {
          return {
            canExecute: false,
            reasonKey: 'preflight.checkout.branch_in_another_worktree',
          }
        }
        return { canExecute: true }
      }

      case 'commit': {
        if (
          context?.commitMessage !== undefined &&
          context.commitMessage.trim().length === 0
        ) {
          return {
            canExecute: false,
            reasonKey: 'preflight.commit.empty_commit_message',
          }
        }
        if (status.conflicted_files.length > 0) {
          return {
            canExecute: false,
            reasonKey: 'preflight.commit.unresolved_conflicts',
          }
        }
        if (!context?.isAmend && status.staged_files.length === 0) {
          return {
            canExecute: false,
            reasonKey: 'preflight.commit.empty_staged_changes',
            suggestedResolution: 'stage_all_and_proceed',
          }
        }
        return { canExecute: true }
      }

      case 'delete_branch': {
        if (
          context?.targetBranch &&
          status.branch_name === context.targetBranch &&
          !status.is_detached_head
        ) {
          return {
            canExecute: false,
            reasonKey: 'preflight.branch.cannot_delete_active_branch',
          }
        }
        const isBranchInWorktree = worktrees.some(
          (worktree) => worktree.branch_name === context?.targetBranch
        )
        if (isBranchInWorktree) {
          return {
            canExecute: false,
            reasonKey: 'preflight.branch.cannot_delete_worktree_branch',
          }
        }
        return { canExecute: true }
      }

      case 'stash_save': {
        if (
          status.staged_files.length === 0 &&
          status.unstaged_files.length === 0
        ) {
          return {
            canExecute: false,
            reasonKey: 'preflight.stash.no_local_changes',
          }
        }
        return { canExecute: true }
      }

      case 'abort_operation': {
        if (!hasActiveOperation) {
          return {
            canExecute: false,
            reasonKey: 'preflight.operation.no_active_operation',
          }
        }
        return { canExecute: true }
      }

      case 'continue_operation': {
        if (!hasActiveOperation) {
          return {
            canExecute: false,
            reasonKey: 'preflight.operation.no_active_operation',
          }
        }
        if (status.conflicted_files.length > 0) {
          return {
            canExecute: false,
            reasonKey: 'preflight.operation.unresolved_conflicts',
          }
        }
        return { canExecute: true }
      }

      case 'cherry_pick': {
        if (hasActiveOperation) {
          return {
            canExecute: false,
            reasonKey: 'preflight.cherry_pick.active_operation_in_progress',
          }
        }
        if (hasDirtyWorkingTree) {
          return {
            canExecute: false,
            reasonKey: 'preflight.cherry_pick.dirty_working_tree',
            suggestedResolution: 'stash_and_proceed',
          }
        }
        return { canExecute: true }
      }

      case 'revert': {
        if (hasActiveOperation) {
          return {
            canExecute: false,
            reasonKey: 'preflight.revert.active_operation_in_progress',
          }
        }
        if (hasDirtyWorkingTree) {
          return {
            canExecute: false,
            reasonKey: 'preflight.revert.dirty_working_tree',
            suggestedResolution: 'stash_and_proceed',
          }
        }
        return { canExecute: true }
      }
    }
  }

  function closePreflightModal(): void {
    preflightModal.value.isOpen = false
    preflightModal.value.isExecutingResolution = false
    preflightModal.value.onProceedWithResolution = async () => {}
  }

  return {
    evaluateAction,
    preflightModal,
    closePreflightModal,
  }
}
