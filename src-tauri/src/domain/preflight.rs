use crate::domain::{
    ApplicationError, DetailedRepositoryStatus, GitWorktreeSummary, RepositoryOperationState,
};

/// Verifies whether the repository is in an eligible state to perform a rebase.
pub fn verify_can_rebase(
    status: &DetailedRepositoryStatus,
    target_branch: &str,
) -> Result<(), ApplicationError> {
    if status.operation_state != RepositoryOperationState::None {
        return Err(ApplicationError::PreflightFailed(
            "preflight.rebase.active_operation_in_progress".to_string(),
        ));
    }

    if !status.staged_files.is_empty() || !status.unstaged_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.rebase.dirty_working_tree".to_string(),
        ));
    }

    if let Some(ref current_branch) = status.branch_name
        && current_branch == target_branch
    {
        return Err(ApplicationError::PreflightFailed(
            "preflight.rebase.cannot_rebase_onto_self".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether the repository is in an eligible state to perform a merge.
pub fn verify_can_merge(
    status: &DetailedRepositoryStatus,
    target_branch: &str,
) -> Result<(), ApplicationError> {
    if status.operation_state != RepositoryOperationState::None {
        return Err(ApplicationError::PreflightFailed(
            "preflight.merge.active_operation_in_progress".to_string(),
        ));
    }

    if !status.staged_files.is_empty() || !status.unstaged_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.merge.dirty_working_tree".to_string(),
        ));
    }

    if let Some(ref current_branch) = status.branch_name
        && current_branch == target_branch
    {
        return Err(ApplicationError::PreflightFailed(
            "preflight.merge.cannot_merge_into_self".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether a branch checkout can safely proceed without destructive collisions.
pub fn verify_can_checkout(
    status: &DetailedRepositoryStatus,
    target_branch: &str,
    worktrees: &[GitWorktreeSummary],
) -> Result<(), ApplicationError> {
    if let Some(ref current_branch) = status.branch_name
        && current_branch == target_branch
        && !status.is_detached_head
    {
        return Err(ApplicationError::PreflightFailed(
            "preflight.checkout.already_on_target_branch".to_string(),
        ));
    }

    // Git prevents checking out a branch currently checked out in another linked worktree
    let is_branch_in_another_worktree = worktrees.iter().any(|worktree| {
        !worktree.is_main
            && worktree
                .branch_name
                .as_deref()
                .is_some_and(|branch| branch == target_branch)
    });

    if is_branch_in_another_worktree {
        return Err(ApplicationError::PreflightFailed(
            "preflight.checkout.branch_in_another_worktree".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether a commit operation can be executed.
pub fn verify_can_commit(
    status: &DetailedRepositoryStatus,
    message: &str,
    is_amend: bool,
) -> Result<(), ApplicationError> {
    if message.trim().is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.commit.empty_commit_message".to_string(),
        ));
    }

    if !status.conflicted_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.commit.unresolved_conflicts".to_string(),
        ));
    }

    if !is_amend && status.staged_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.commit.empty_staged_changes".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether a branch can be safely deleted.
pub fn verify_can_delete_branch(
    status: &DetailedRepositoryStatus,
    branch_name: &str,
    worktrees: &[GitWorktreeSummary],
) -> Result<(), ApplicationError> {
    if let Some(ref current_branch) = status.branch_name
        && current_branch == branch_name
        && !status.is_detached_head
    {
        return Err(ApplicationError::PreflightFailed(
            "preflight.branch.cannot_delete_active_branch".to_string(),
        ));
    }

    let is_branch_in_worktree = worktrees.iter().any(|worktree| {
        worktree
            .branch_name
            .as_deref()
            .is_some_and(|branch| branch == branch_name)
    });

    if is_branch_in_worktree {
        return Err(ApplicationError::PreflightFailed(
            "preflight.branch.cannot_delete_worktree_branch".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether a stash save operation has modified content to preserve.
pub fn verify_can_stash_save(status: &DetailedRepositoryStatus) -> Result<(), ApplicationError> {
    if status.staged_files.is_empty() && status.unstaged_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.stash.no_local_changes".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether an active operation exists to abort.
pub fn verify_can_abort_operation(
    status: &DetailedRepositoryStatus,
) -> Result<(), ApplicationError> {
    if status.operation_state == RepositoryOperationState::None {
        return Err(ApplicationError::PreflightFailed(
            "preflight.operation.no_active_operation".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether an active operation can be continued after conflicts are resolved.
pub fn verify_can_continue_operation(
    status: &DetailedRepositoryStatus,
) -> Result<(), ApplicationError> {
    if status.operation_state == RepositoryOperationState::None {
        return Err(ApplicationError::PreflightFailed(
            "preflight.operation.no_active_operation".to_string(),
        ));
    }

    if !status.conflicted_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.operation.unresolved_conflicts".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether a cherry-pick operation can proceed safely.
pub fn verify_can_cherry_pick(status: &DetailedRepositoryStatus) -> Result<(), ApplicationError> {
    if status.operation_state != RepositoryOperationState::None {
        return Err(ApplicationError::PreflightFailed(
            "preflight.cherry_pick.active_operation_in_progress".to_string(),
        ));
    }

    if !status.staged_files.is_empty() || !status.unstaged_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.cherry_pick.dirty_working_tree".to_string(),
        ));
    }

    Ok(())
}

/// Verifies whether a revert operation can proceed safely.
pub fn verify_can_revert(status: &DetailedRepositoryStatus) -> Result<(), ApplicationError> {
    if status.operation_state != RepositoryOperationState::None {
        return Err(ApplicationError::PreflightFailed(
            "preflight.revert.active_operation_in_progress".to_string(),
        ));
    }

    if !status.staged_files.is_empty() || !status.unstaged_files.is_empty() {
        return Err(ApplicationError::PreflightFailed(
            "preflight.revert.dirty_working_tree".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::FileStatusEntry;

    fn create_dummy_status(
        branch: &str,
        operation: RepositoryOperationState,
        staged_count: usize,
        unstaged_count: usize,
        conflicted_count: usize,
    ) -> DetailedRepositoryStatus {
        DetailedRepositoryStatus {
            branch_name: Some(branch.to_string()),
            head_commit_hash: Some("abcdef123456".to_string()),
            upstream_branch_name: Some(format!("origin/{branch}")),
            ahead_count: 0,
            behind_count: 0,
            is_detached_head: false,
            staged_files: (0..staged_count)
                .map(|index| FileStatusEntry {
                    path: format!("file_{index}.txt"),
                    original_path: None,
                    status: crate::domain::FileChangeStatus::Modified,
                    is_staged: true,
                })
                .collect(),
            unstaged_files: (0..unstaged_count)
                .map(|index| FileStatusEntry {
                    path: format!("unstaged_{index}.txt"),
                    original_path: None,
                    status: crate::domain::FileChangeStatus::Modified,
                    is_staged: false,
                })
                .collect(),
            conflicted_files: (0..conflicted_count)
                .map(|index| FileStatusEntry {
                    path: format!("conflicted_{index}.txt"),
                    original_path: None,
                    status: crate::domain::FileChangeStatus::Conflicted,
                    is_staged: false,
                })
                .collect(),
            operation_state: operation,
        }
    }

    #[test]
    fn test_verify_can_rebase() {
        // Clean repository - succeeds
        let clean_status = create_dummy_status("feature", RepositoryOperationState::None, 0, 0, 0);
        assert!(verify_can_rebase(&clean_status, "main").is_ok());

        // Dirty repository - fails with dirty_working_tree
        let dirty_status = create_dummy_status("feature", RepositoryOperationState::None, 1, 0, 0);
        match verify_can_rebase(&dirty_status, "main") {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.rebase.dirty_working_tree");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        // Active operation - fails with active_operation_in_progress
        let rebasing_status =
            create_dummy_status("feature", RepositoryOperationState::Rebasing, 0, 0, 0);
        match verify_can_rebase(&rebasing_status, "main") {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.rebase.active_operation_in_progress");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        // Rebase onto self - fails
        match verify_can_rebase(&clean_status, "feature") {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.rebase.cannot_rebase_onto_self");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }
    }

    #[test]
    fn test_verify_can_merge() {
        let clean_status = create_dummy_status("main", RepositoryOperationState::None, 0, 0, 0);
        assert!(verify_can_merge(&clean_status, "feature").is_ok());

        // Cannot merge self
        match verify_can_merge(&clean_status, "main") {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.merge.cannot_merge_into_self");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }
    }

    #[test]
    fn test_verify_can_commit() {
        let clean_status = create_dummy_status("main", RepositoryOperationState::None, 0, 0, 0);

        // Empty commit message
        match verify_can_commit(&clean_status, "   ", false) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.commit.empty_commit_message");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        // Empty staged files for normal commit
        match verify_can_commit(&clean_status, "feat: implement test", false) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.commit.empty_staged_changes");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        // Amend commit with empty staging is allowed
        assert!(verify_can_commit(&clean_status, "feat: implement test", true).is_ok());

        // Unresolved conflicts
        let conflict_status =
            create_dummy_status("main", RepositoryOperationState::Merging, 1, 0, 1);
        match verify_can_commit(&conflict_status, "feat: conflict resolution", false) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.commit.unresolved_conflicts");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }
    }

    #[test]
    fn test_verify_can_checkout() {
        let status = create_dummy_status("feature-a", RepositoryOperationState::None, 0, 0, 0);
        let worktrees = vec![
            GitWorktreeSummary {
                directory_path: "/repo".to_string(),
                head_commit_hash: "123".to_string(),
                branch_name: Some("feature-a".to_string()),
                is_bare: false,
                is_locked: false,
                lock_reason: None,
                is_prunable: false,
                is_main: true,
            },
            GitWorktreeSummary {
                directory_path: "/repo-wt".to_string(),
                head_commit_hash: "456".to_string(),
                branch_name: Some("feature-b".to_string()),
                is_bare: false,
                is_locked: false,
                lock_reason: None,
                is_prunable: false,
                is_main: false,
            },
        ];

        // Already on target branch
        match verify_can_checkout(&status, "feature-a", &worktrees) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.checkout.already_on_target_branch");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        // Target branch in another worktree
        match verify_can_checkout(&status, "feature-b", &worktrees) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.checkout.branch_in_another_worktree");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        // Free branch
        assert!(verify_can_checkout(&status, "feature-c", &worktrees).is_ok());
    }

    #[test]
    fn test_verify_can_delete_branch() {
        let status = create_dummy_status("feature-a", RepositoryOperationState::None, 0, 0, 0);
        let worktrees = vec![GitWorktreeSummary {
            directory_path: "/repo".to_string(),
            head_commit_hash: "123".to_string(),
            branch_name: Some("feature-b".to_string()),
            is_bare: false,
            is_locked: false,
            lock_reason: None,
            is_prunable: false,
            is_main: false,
        }];

        // Cannot delete active branch
        match verify_can_delete_branch(&status, "feature-a", &worktrees) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.branch.cannot_delete_active_branch");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        // Cannot delete worktree branch
        match verify_can_delete_branch(&status, "feature-b", &worktrees) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.branch.cannot_delete_worktree_branch");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        assert!(verify_can_delete_branch(&status, "feature-c", &worktrees).is_ok());
    }

    #[test]
    fn test_verify_can_stash_save() {
        let clean = create_dummy_status("main", RepositoryOperationState::None, 0, 0, 0);
        match verify_can_stash_save(&clean) {
            Err(ApplicationError::PreflightFailed(reason)) => {
                assert_eq!(reason, "preflight.stash.no_local_changes");
            }
            other => panic!("Expected PreflightFailed, got {other:?}"),
        }

        let dirty = create_dummy_status("main", RepositoryOperationState::None, 1, 0, 0);
        assert!(verify_can_stash_save(&dirty).is_ok());
    }

    #[test]
    fn test_verify_can_abort_and_continue() {
        let clean = create_dummy_status("main", RepositoryOperationState::None, 0, 0, 0);
        assert!(verify_can_abort_operation(&clean).is_err());
        assert!(verify_can_continue_operation(&clean).is_err());

        let rebasing = create_dummy_status("main", RepositoryOperationState::Rebasing, 0, 0, 0);
        assert!(verify_can_abort_operation(&rebasing).is_ok());
        assert!(verify_can_continue_operation(&rebasing).is_ok());

        let conflicted = create_dummy_status("main", RepositoryOperationState::Rebasing, 0, 0, 1);
        assert!(verify_can_abort_operation(&conflicted).is_ok());
        assert!(verify_can_continue_operation(&conflicted).is_err());
    }
}
