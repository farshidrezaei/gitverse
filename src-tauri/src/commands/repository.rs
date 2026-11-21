use std::path::PathBuf;
use tauri::{AppHandle, State};
use crate::application::ApplicationState;
use crate::domain::{
    ApplicationDiagnostics, ApplicationError, CommitComparison, CommitDetails, ConflictFileDetails, DetailedRepositoryStatus, DirectoryListing, FileDiff, GitBinaryInformation,
    GitWorktreeSummary, InteractiveRebaseRequest, MergeBranchOutcome, RebaseBranchOutcome, RebasePlanItem, RecentRepositoryRecord, RepositoryOverview, StashRecord,
};

/// Tauri IPC command to discover and validate the native Git binary.
#[tauri::command]
pub async fn discover_git_binary(
    state: State<'_, ApplicationState>,
    custom_path: Option<String>,
) -> Result<GitBinaryInformation, ApplicationError> {
    state.discover_git(custom_path.as_deref()).await
}

/// Tauri IPC command to open a Git repository from a directory path.
#[tauri::command]
pub async fn open_repository(
    app_handle: AppHandle,
    state: State<'_, ApplicationState>,
    directory_path: String,
) -> Result<RepositoryOverview, ApplicationError> {
    let path = PathBuf::from(&directory_path);
    state.open_repository(app_handle, &path).await
}

/// Tauri IPC command to get currently active repository.
#[tauri::command]
pub async fn get_current_repository(
    state: State<'_, ApplicationState>,
) -> Result<Option<RepositoryOverview>, ApplicationError> {
    Ok(state.get_current_repository().await)
}

/// Tauri IPC command to close the currently active repository.
#[tauri::command]
pub async fn close_repository(
    state: State<'_, ApplicationState>,
) -> Result<(), ApplicationError> {
    state.close_current_repository().await;
    Ok(())
}

/// Tauri IPC command to get detailed status (staged, unstaged, conflicted files).
#[tauri::command]
pub async fn get_repository_status(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<DetailedRepositoryStatus, ApplicationError> {
    state.get_repository_status(directory_path.as_deref()).await
}

/// Tauri IPC command to stage a file.
#[tauri::command]
pub async fn stage_file(
    state: State<'_, ApplicationState>,
    file_path: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.stage_file(directory_path.as_deref(), &file_path).await
}

/// Tauri IPC command to unstage a file.
#[tauri::command]
pub async fn unstage_file(
    state: State<'_, ApplicationState>,
    file_path: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.unstage_file(directory_path.as_deref(), &file_path).await
}

/// Tauri IPC command to stage all changes.
#[tauri::command]
pub async fn stage_all(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.stage_all(directory_path.as_deref()).await
}

/// Tauri IPC command to unstage all changes.
#[tauri::command]
pub async fn unstage_all(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.unstage_all(directory_path.as_deref()).await
}

/// Tauri IPC command to discard changes for a file.
#[tauri::command]
pub async fn discard_file_changes(
    state: State<'_, ApplicationState>,
    file_path: String,
    is_untracked: bool,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.discard_file_changes(directory_path.as_deref(), &file_path, is_untracked).await
}

/// Tauri IPC command to stage a unified diff patch into the Git index.
#[tauri::command]
pub async fn stage_patch(
    state: State<'_, ApplicationState>,
    patch_content: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.stage_patch(directory_path.as_deref(), &patch_content).await
}

/// Tauri IPC command to unstage a unified diff patch from the Git index.
#[tauri::command]
pub async fn unstage_patch(
    state: State<'_, ApplicationState>,
    patch_content: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.unstage_patch(directory_path.as_deref(), &patch_content).await
}

/// Tauri IPC command to discard changes matching a unified diff patch from working tree.
#[tauri::command]
pub async fn discard_patch(
    state: State<'_, ApplicationState>,
    patch_content: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.discard_patch(directory_path.as_deref(), &patch_content).await
}

/// Tauri IPC command to export repository changes or a commit as a patch.
#[tauri::command]
pub async fn export_patch(
    state: State<'_, ApplicationState>,
    scope: String,
    commit_hash: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state
        .export_patch(directory_path.as_deref(), &scope, commit_hash.as_deref())
        .await
}

/// Tauri IPC command to test if a patch can apply cleanly without conflicts.
#[tauri::command]
pub async fn check_patch(
    state: State<'_, ApplicationState>,
    patch_content: String,
    directory_path: Option<String>,
) -> Result<crate::domain::PatchCheckResult, ApplicationError> {
    state
        .check_patch(directory_path.as_deref(), &patch_content)
        .await
}

/// Tauri IPC command to apply a unified diff patch to working tree or staging index.
#[tauri::command]
pub async fn apply_patch_file(
    state: State<'_, ApplicationState>,
    patch_content: String,
    is_cached: bool,
    is_reverse: bool,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state
        .apply_patch_file(
            directory_path.as_deref(),
            &patch_content,
            is_cached,
            is_reverse,
        )
        .await
}

/// Tauri IPC command to fetch all Git submodules and their status.
#[tauri::command]
pub async fn get_submodules(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<Vec<crate::domain::GitSubmoduleSummary>, ApplicationError> {
    state.get_submodules(directory_path.as_deref()).await
}

/// Tauri IPC command to update one or all Git submodules.
#[tauri::command]
pub async fn update_submodules(
    state: State<'_, ApplicationState>,
    submodule_path: Option<String>,
    is_init: bool,
    is_recursive: bool,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state
        .update_submodules(
            directory_path.as_deref(),
            submodule_path.as_deref(),
            is_init,
            is_recursive,
        )
        .await
}

/// Tauri IPC command to synchronize Git submodules URL configuration.
#[tauri::command]
pub async fn sync_submodules(
    state: State<'_, ApplicationState>,
    submodule_path: Option<String>,
    is_recursive: bool,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state
        .sync_submodules(
            directory_path.as_deref(),
            submodule_path.as_deref(),
            is_recursive,
        )
        .await
}

/// Tauri IPC command to compute repository analytics, commit velocity, and contributors.
#[tauri::command]
pub async fn get_repository_insights(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<crate::domain::RepositoryInsights, ApplicationError> {
    state.get_repository_insights(directory_path.as_deref()).await
}

/// Tauri IPC command to fetch the chronological commit history of a file.
#[tauri::command]
pub async fn get_file_history(
    state: State<'_, ApplicationState>,
    file_path: String,
    limit: Option<usize>,
    directory_path: Option<String>,
) -> Result<crate::domain::FileHistoryTimeline, ApplicationError> {
    state
        .get_file_history(directory_path.as_deref(), &file_path, limit)
        .await
}

/// Tauri IPC command to fetch file content at a historical commit revision.
#[tauri::command]
pub async fn get_file_content_at_commit(
    state: State<'_, ApplicationState>,
    commit_hash: String,
    file_path: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state
        .get_file_content_at_commit(
            directory_path.as_deref(),
            &commit_hash,
            &file_path,
        )
        .await
}

/// Tauri IPC command to search repository files or history using git grep.
#[tauri::command]
pub async fn search_repository(
    state: State<'_, ApplicationState>,
    query: String,
    reference: Option<String>,
    path_pattern: Option<String>,
    is_case_sensitive: Option<bool>,
    is_word_match: Option<bool>,
    is_regex: Option<bool>,
    directory_path: Option<String>,
) -> Result<crate::domain::RepositorySearchResult, ApplicationError> {
    state
        .search_repository(
            directory_path.as_deref(),
            &query,
            reference.as_deref(),
            path_pattern.as_deref(),
            is_case_sensitive.unwrap_or(false),
            is_word_match.unwrap_or(false),
            is_regex.unwrap_or(false),
        )
        .await
}

/// Tauri IPC command to commit staged changes.
#[tauri::command]
pub async fn commit(
    state: State<'_, ApplicationState>,
    message: String,
    is_amend: bool,
    bypass_hooks: Option<bool>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state
        .commit(
            directory_path.as_deref(),
            &message,
            is_amend,
            bypass_hooks.unwrap_or(false),
        )
        .await
}

/// Tauri IPC command to get structured file diff.
#[tauri::command]
pub async fn get_file_diff(
    state: State<'_, ApplicationState>,
    file_path: String,
    is_staged: bool,
    directory_path: Option<String>,
) -> Result<crate::domain::FileDiff, ApplicationError> {
    state.get_file_diff(directory_path.as_deref(), &file_path, is_staged).await
}

/// Tauri IPC command to retrieve configured commit template from repository or git config.
#[tauri::command]
pub async fn get_commit_template(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<Option<String>, ApplicationError> {
    state.get_commit_template(directory_path.as_deref()).await
}

/// Tauri IPC command to extract Git reflog history for HEAD or a specific reference.
#[tauri::command]
pub async fn get_reflog(
    state: State<'_, ApplicationState>,
    reference: Option<String>,
    limit: Option<usize>,
    directory_path: Option<String>,
) -> Result<Vec<crate::domain::ReflogRecord>, ApplicationError> {
    state
        .get_reflog(directory_path.as_deref(), reference, limit)
        .await
}

/// Tauri IPC command to inspect the current Git bisect state and log.
#[tauri::command]
pub async fn get_bisect_status(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<crate::domain::BisectState, ApplicationError> {
    state.get_bisect_status(directory_path.as_deref()).await
}

/// Tauri IPC command to start a Git bisect session.
#[tauri::command]
pub async fn bisect_start(
    state: State<'_, ApplicationState>,
    bad_commit_hash: String,
    good_commit_hash: String,
    directory_path: Option<String>,
) -> Result<crate::domain::BisectState, ApplicationError> {
    state
        .bisect_start(
            directory_path.as_deref(),
            &bad_commit_hash,
            &good_commit_hash,
        )
        .await
}

/// Tauri IPC command to record a Git bisect decision (good, bad, skip).
#[tauri::command]
pub async fn bisect_step(
    state: State<'_, ApplicationState>,
    decision: String,
    directory_path: Option<String>,
) -> Result<crate::domain::BisectState, ApplicationError> {
    state
        .bisect_step(directory_path.as_deref(), &decision)
        .await
}

/// Tauri IPC command to reset a Git bisect session.
#[tauri::command]
pub async fn bisect_reset(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.bisect_reset(directory_path.as_deref()).await
}

/// Tauri IPC command to extract Git commit history and compute deterministic DAG layout.
#[tauri::command]
pub async fn get_commit_graph(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
    skip: Option<usize>,
    limit: Option<usize>,
    branch_or_reference: Option<String>,
) -> Result<crate::domain::CommitGraphHistoryResponse, ApplicationError> {
    state.get_commit_graph(directory_path.as_deref(), skip, limit, branch_or_reference).await
}

/// Tauri IPC command to fetch repository references (branches, remotes, tags).
#[tauri::command]
pub async fn get_repository_references(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<crate::domain::RepositoryReferences, ApplicationError> {
    state.get_repository_references(directory_path.as_deref()).await
}

/// Tauri IPC command to checkout or switch branches.
#[tauri::command]
pub async fn checkout_branch(
    state: State<'_, ApplicationState>,
    branch_name: String,
    create_if_missing: Option<bool>,
    start_point: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.checkout_branch(
        directory_path.as_deref(),
        &branch_name,
        create_if_missing.unwrap_or(false),
        start_point.as_deref(),
    ).await
}

/// Tauri IPC command to create a new branch.
#[tauri::command]
pub async fn create_branch(
    state: State<'_, ApplicationState>,
    branch_name: String,
    start_point: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.create_branch(directory_path.as_deref(), &branch_name, start_point.as_deref()).await
}

/// Tauri IPC command to delete a branch.
#[tauri::command]
pub async fn delete_branch(
    state: State<'_, ApplicationState>,
    branch_name: String,
    force: Option<bool>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.delete_branch(directory_path.as_deref(), &branch_name, force.unwrap_or(false)).await
}

/// Tauri IPC command to rename a branch.
#[tauri::command]
pub async fn rename_branch(
    state: State<'_, ApplicationState>,
    old_name: String,
    new_name: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.rename_branch(directory_path.as_deref(), &old_name, &new_name).await
}

/// Tauri IPC command to fetch changes from remote.
#[tauri::command]
pub async fn fetch_remote(
    state: State<'_, ApplicationState>,
    remote_name: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.fetch_remote(directory_path.as_deref(), remote_name.as_deref()).await
}

/// Tauri IPC command to pull changes.
#[tauri::command]
pub async fn pull_changes(
    state: State<'_, ApplicationState>,
    remote_name: Option<String>,
    branch_name: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.pull_changes(directory_path.as_deref(), remote_name.as_deref(), branch_name.as_deref()).await
}

/// Tauri IPC command to push commits.
#[tauri::command]
pub async fn push_changes(
    state: State<'_, ApplicationState>,
    remote_name: Option<String>,
    branch_name: Option<String>,
    set_upstream: Option<bool>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.push_changes(
        directory_path.as_deref(),
        remote_name.as_deref(),
        branch_name.as_deref(),
        set_upstream.unwrap_or(false),
    ).await
}

/// Tauri IPC command to add a new Git remote.
#[tauri::command]
pub async fn add_remote(
    state: State<'_, ApplicationState>,
    name: String,
    url: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.add_remote(directory_path.as_deref(), &name, &url).await
}

/// Tauri IPC command to remove a Git remote.
#[tauri::command]
pub async fn remove_remote(
    state: State<'_, ApplicationState>,
    name: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.remove_remote(directory_path.as_deref(), &name).await
}

/// Tauri IPC command to rename a Git remote.
#[tauri::command]
pub async fn rename_remote(
    state: State<'_, ApplicationState>,
    old_name: String,
    new_name: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.rename_remote(directory_path.as_deref(), &old_name, &new_name).await
}

/// Tauri IPC command to update a Git remote URL.
#[tauri::command]
pub async fn set_remote_url(
    state: State<'_, ApplicationState>,
    name: String,
    url: String,
    is_push_only: Option<bool>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.set_remote_url(
        directory_path.as_deref(),
        &name,
        &url,
        is_push_only.unwrap_or(false),
    ).await
}

/// Tauri IPC command to retrieve comprehensive commit details with changed files.
#[tauri::command]
pub async fn get_commit_details(
    state: State<'_, ApplicationState>,
    commit_hash: String,
    directory_path: Option<String>,
) -> Result<CommitDetails, ApplicationError> {
    state.get_commit_details(directory_path.as_deref(), &commit_hash).await
}

/// Tauri IPC command to retrieve the diff of a specific file or the entire commit.
#[tauri::command]
pub async fn get_commit_file_diff(
    state: State<'_, ApplicationState>,
    commit_hash: String,
    file_path: Option<String>,
    directory_path: Option<String>,
) -> Result<FileDiff, ApplicationError> {
    state.get_commit_file_diff(directory_path.as_deref(), &commit_hash, file_path.as_deref()).await
}

/// Tauri IPC command to compare two commits or references and return changed files and metrics.
#[tauri::command]
pub async fn compare_commits(
    state: State<'_, ApplicationState>,
    base_hash: String,
    target_hash: String,
    directory_path: Option<String>,
) -> Result<CommitComparison, ApplicationError> {
    state.compare_commits(directory_path.as_deref(), &base_hash, &target_hash).await
}

/// Tauri IPC command to retrieve the diff of a specific file or all files between two commits.
#[tauri::command]
pub async fn get_comparison_file_diff(
    state: State<'_, ApplicationState>,
    base_hash: String,
    target_hash: String,
    file_path: Option<String>,
    directory_path: Option<String>,
) -> Result<FileDiff, ApplicationError> {
    state.get_comparison_file_diff(directory_path.as_deref(), &base_hash, &target_hash, file_path.as_deref()).await
}

/// Tauri IPC command to fetch recent repositories from local SQLite database.
#[tauri::command]
pub async fn get_recent_repositories(
    state: State<'_, ApplicationState>,
    limit: Option<usize>,
) -> Result<Vec<RecentRepositoryRecord>, ApplicationError> {
    let limit_value = limit.unwrap_or(20);
    state.get_recent_repositories(limit_value)
}

/// Tauri IPC command to remove a repository from recent history.
#[tauri::command]
pub async fn remove_recent_repository(
    state: State<'_, ApplicationState>,
    directory_path: String,
) -> Result<(), ApplicationError> {
    state.remove_recent_repository(&directory_path)
}

/// Tauri IPC command to clear all repositories from recent history.
#[tauri::command]
pub async fn clear_recent_repositories(
    state: State<'_, ApplicationState>,
) -> Result<(), ApplicationError> {
    state.clear_recent_repositories()
}

/// Tauri IPC command to list all stash entries for the active repository.
#[tauri::command]
pub async fn get_stash_list(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<Vec<StashRecord>, ApplicationError> {
    state.get_stash_list(directory_path.as_deref()).await
}

/// Tauri IPC command to save uncommitted changes to a stash entry.
#[tauri::command]
pub async fn save_stash(
    state: State<'_, ApplicationState>,
    message: Option<String>,
    include_untracked: bool,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.save_stash(directory_path.as_deref(), message.as_deref(), include_untracked).await
}

/// Tauri IPC command to apply a stash entry by index without removing it.
#[tauri::command]
pub async fn apply_stash(
    state: State<'_, ApplicationState>,
    stash_index: usize,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.apply_stash(directory_path.as_deref(), stash_index).await
}

/// Tauri IPC command to pop a stash entry by index.
#[tauri::command]
pub async fn pop_stash(
    state: State<'_, ApplicationState>,
    stash_index: usize,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.pop_stash(directory_path.as_deref(), stash_index).await
}

/// Tauri IPC command to drop a stash entry by index.
#[tauri::command]
pub async fn drop_stash(
    state: State<'_, ApplicationState>,
    stash_index: usize,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.drop_stash(directory_path.as_deref(), stash_index).await
}

/// Tauri IPC command to get detailed changed files within a stash.
#[tauri::command]
pub async fn get_stash_details(
    state: State<'_, ApplicationState>,
    stash_index: usize,
    directory_path: Option<String>,
) -> Result<crate::domain::StashDetails, ApplicationError> {
    state.get_stash_details(directory_path.as_deref(), stash_index).await
}

/// Tauri IPC command to get unified diff for a single file within a stash.
#[tauri::command]
pub async fn get_stash_file_diff(
    state: State<'_, ApplicationState>,
    stash_index: usize,
    file_path: String,
    directory_path: Option<String>,
) -> Result<crate::domain::FileDiff, ApplicationError> {
    state.get_stash_file_diff(directory_path.as_deref(), stash_index, &file_path).await
}

/// Tauri IPC command to cherry-pick a commit.
#[tauri::command]
pub async fn cherry_pick_commit(
    state: State<'_, ApplicationState>,
    commit_hash: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.cherry_pick_commit(directory_path.as_deref(), &commit_hash).await
}

/// Tauri IPC command to revert a commit.
#[tauri::command]
pub async fn revert_commit(
    state: State<'_, ApplicationState>,
    commit_hash: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.revert_commit(directory_path.as_deref(), &commit_hash).await
}

/// Tauri IPC command to reset current branch to a commit.
#[tauri::command]
pub async fn reset_to_commit(
    state: State<'_, ApplicationState>,
    commit_hash: String,
    reset_mode: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.reset_to_commit(directory_path.as_deref(), &commit_hash, &reset_mode).await
}

/// Tauri IPC command to retrieve 3-way conflict stages and working file content.
#[tauri::command]
pub async fn get_conflict_file_details(
    state: State<'_, ApplicationState>,
    file_path: String,
    directory_path: Option<String>,
) -> Result<ConflictFileDetails, ApplicationError> {
    state.get_conflict_file_details(directory_path.as_deref(), &file_path).await
}

/// Tauri IPC command to resolve a conflict file with a preset strategy ('ours', 'theirs', 'mark_resolved').
#[tauri::command]
pub async fn resolve_conflict_file(
    state: State<'_, ApplicationState>,
    file_path: String,
    resolution_strategy: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.resolve_conflict_file(directory_path.as_deref(), &file_path, &resolution_strategy).await
}

/// Tauri IPC command to save custom resolved file content and stage it.
#[tauri::command]
pub async fn write_conflict_resolved_content(
    state: State<'_, ApplicationState>,
    file_path: String,
    resolved_content: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.write_conflict_resolved_content(directory_path.as_deref(), &file_path, &resolved_content).await
}

/// Tauri IPC command to merge a branch into current branch.
#[tauri::command]
pub async fn merge_branch(
    state: State<'_, ApplicationState>,
    branch_name: String,
    no_fast_forward: Option<bool>,
    directory_path: Option<String>,
) -> Result<MergeBranchOutcome, ApplicationError> {
    state
        .merge_branch(
            directory_path.as_deref(),
            &branch_name,
            no_fast_forward.unwrap_or(false),
        )
        .await
}

/// Tauri IPC command to rebase current branch onto an upstream branch.
#[tauri::command]
pub async fn rebase_branch(
    state: State<'_, ApplicationState>,
    upstream_branch: String,
    directory_path: Option<String>,
) -> Result<RebaseBranchOutcome, ApplicationError> {
    state
        .rebase_branch(directory_path.as_deref(), &upstream_branch)
        .await
}

/// Tauri IPC command to abort active ongoing repository operation.
#[tauri::command]
pub async fn abort_repository_operation(
    state: State<'_, ApplicationState>,
    create_backup: Option<bool>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.abort_repository_operation(directory_path.as_deref(), create_backup).await
}

/// Tauri IPC command to continue active ongoing repository operation after conflicts are resolved.
#[tauri::command]
pub async fn continue_repository_operation(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.continue_repository_operation(directory_path.as_deref()).await
}

/// Tauri IPC command to list subdirectories only for custom folder picker.
#[tauri::command]
pub async fn list_directory_folders(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<DirectoryListing, ApplicationError> {
    state.list_directory_folders(directory_path.as_deref()).await
}

/// Tauri IPC command to list worktrees linked to the repository.
#[tauri::command]
pub async fn list_worktrees(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<Vec<GitWorktreeSummary>, ApplicationError> {
    state.list_worktrees(directory_path.as_deref()).await
}

/// Tauri IPC command to add a new worktree to the repository.
#[tauri::command]
pub async fn add_worktree(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
    worktree_path: String,
    branch_name: Option<String>,
    is_new_branch: Option<bool>,
) -> Result<String, ApplicationError> {
    state.add_worktree(
        directory_path.as_deref(),
        &worktree_path,
        branch_name.as_deref(),
        is_new_branch.unwrap_or(false),
    ).await
}

/// Tauri IPC command to remove a worktree from the repository.
#[tauri::command]
pub async fn remove_worktree(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
    worktree_path: String,
    force: Option<bool>,
) -> Result<String, ApplicationError> {
    state.remove_worktree(
        directory_path.as_deref(),
        &worktree_path,
        force.unwrap_or(false),
    ).await
}

/// Tauri IPC command to get commits for interactive rebase plan.
#[tauri::command]
pub async fn get_rebase_commits(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
    base_commit_hash: String,
) -> Result<Vec<RebasePlanItem>, ApplicationError> {
    state.get_rebase_commits(directory_path.as_deref(), &base_commit_hash).await
}

/// Tauri IPC command to execute interactive rebase with plan items.
#[tauri::command]
pub async fn execute_interactive_rebase(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
    request: InteractiveRebaseRequest,
) -> Result<RebaseBranchOutcome, ApplicationError> {
    state.execute_interactive_rebase(directory_path.as_deref(), request).await
}

/// Tauri IPC command to extract file blame annotations.
#[tauri::command]
pub async fn get_file_blame(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
    file_path: String,
    revision: Option<String>,
) -> Result<crate::domain::FileBlame, ApplicationError> {
    state.get_file_blame(directory_path.as_deref(), &file_path, revision.as_deref()).await
}

/// Tauri IPC command to get system and application diagnostics.
#[tauri::command]
pub async fn get_application_diagnostics(
    state: State<'_, ApplicationState>,
) -> Result<ApplicationDiagnostics, ApplicationError> {
    state.get_application_diagnostics().await
}

/// Tauri IPC command to export a sanitized diagnostic bundle for troubleshooting.
#[tauri::command]
pub async fn export_diagnostic_bundle(
    state: State<'_, ApplicationState>,
) -> Result<crate::infrastructure::diagnostics::DiagnosticBundle, ApplicationError> {
    state.export_diagnostic_bundle().await
}

/// Tauri IPC command to open an external URL in the default system browser.
#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), ApplicationError> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(ApplicationError::InvalidInput("Only HTTP/HTTPS URLs are allowed".into()));
    }

    #[cfg(target_os = "linux")]
    {
        tokio::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|error| ApplicationError::Unexpected(format!("Failed to open browser: {}", error)))?;
    }

    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|error| ApplicationError::Unexpected(format!("Failed to open browser: {}", error)))?;
    }

    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|error| ApplicationError::Unexpected(format!("Failed to open browser: {}", error)))?;
    }

    Ok(())
}

/// Tauri IPC command to read plain text from the system clipboard.
#[tauri::command]
pub async fn read_clipboard_text() -> Result<String, ApplicationError> {
    #[cfg(target_os = "linux")]
    {
        // Try wl-paste (Wayland)
        if let Ok(output) = tokio::process::Command::new("wl-paste").output().await {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        // Try xclip (X11)
        if let Ok(output) = tokio::process::Command::new("xclip").args(["-selection", "clipboard", "-o"]).output().await {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        // Try xsel (X11)
        if let Ok(output) = tokio::process::Command::new("xsel").args(["--clipboard", "--output"]).output().await {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        Err(ApplicationError::Unexpected("Clipboard command failed on Linux".into()))
    }

    #[cfg(target_os = "macos")]
    {
        let output = tokio::process::Command::new("pbpaste")
            .output()
            .await
            .map_err(|error| ApplicationError::Unexpected(format!("Failed to execute pbpaste: {}", error)))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[cfg(target_os = "windows")]
    {
        let output = tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-Clipboard"])
            .output()
            .await
            .map_err(|error| ApplicationError::Unexpected(format!("Failed to read clipboard: {}", error)))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

/// Tauri IPC command to open a directory or file in a designated external editor.
#[tauri::command]
pub async fn open_in_external_editor(
    directory_path: String,
    file_path: Option<String>,
    editor_name: String,
) -> Result<(), ApplicationError> {
    let base_directory = std::path::Path::new(&directory_path);
    if !base_directory.exists() {
        return Err(ApplicationError::InvalidInput(format!(
            "Directory does not exist: {}",
            directory_path
        )));
    }

    let binary_name = match editor_name.to_lowercase().as_str() {
        "vscode" | "code" => "code",
        "cursor" => "cursor",
        "zed" => "zed",
        "sublime" | "subl" => "subl",
        "intellij" | "idea" => "idea",
        _ => {
            return Err(ApplicationError::InvalidInput(format!(
                "Unsupported external editor: {}",
                editor_name
            )))
        }
    };

    let target_path = if let Some(ref relative_or_absolute_file) = file_path {
        let candidate_path = std::path::Path::new(relative_or_absolute_file);
        if candidate_path.is_absolute() {
            candidate_path.to_path_buf()
        } else {
            base_directory.join(candidate_path)
        }
    } else {
        base_directory.to_path_buf()
    };

    tokio::process::Command::new(binary_name)
        .arg(&target_path)
        .current_dir(base_directory)
        .spawn()
        .map_err(|error| {
            ApplicationError::Unexpected(format!(
                "Failed to launch external editor '{}': {}",
                binary_name, error
            ))
        })?;

    Ok(())
}

/// Tauri IPC command to open the terminal in the given directory path.
#[tauri::command]
pub async fn open_in_terminal(directory_path: String) -> Result<(), ApplicationError> {
    let target_directory = std::path::Path::new(&directory_path);
    if !target_directory.exists() {
        return Err(ApplicationError::InvalidInput(format!(
            "Directory does not exist: {}",
            directory_path
        )));
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(terminal_from_environment) = std::env::var("TERMINAL") {
            if tokio::process::Command::new(&terminal_from_environment)
                .current_dir(target_directory)
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }

        let terminal_candidates = [
            "x-terminal-emulator",
            "gnome-terminal",
            "kitty",
            "alacritty",
            "konsole",
            "xfce4-terminal",
            "tilix",
            "xterm",
        ];

        for candidate_terminal in terminal_candidates {
            if tokio::process::Command::new(candidate_terminal)
                .current_dir(target_directory)
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }

        Err(ApplicationError::Unexpected(
            "No suitable terminal emulator found on Linux system".into(),
        ))
    }

    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("open")
            .args(["-a", "Terminal", &directory_path])
            .current_dir(target_directory)
            .spawn()
            .map_err(|error| {
                ApplicationError::Unexpected(format!("Failed to open Terminal: {}", error))
            })?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        if tokio::process::Command::new("cmd")
            .args(["/C", "start", "wt", "-d", &directory_path])
            .current_dir(target_directory)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        tokio::process::Command::new("cmd")
            .args(["/C", "start", "powershell"])
            .current_dir(target_directory)
            .spawn()
            .map_err(|error| {
                ApplicationError::Unexpected(format!("Failed to launch terminal: {}", error))
            })?;
        Ok(())
    }
}

/// Tauri IPC command to reveal a file or directory in the system file manager.
#[tauri::command]
pub async fn open_in_file_manager(
    directory_path: String,
    file_path: Option<String>,
) -> Result<(), ApplicationError> {
    let base_directory = std::path::Path::new(&directory_path);
    if !base_directory.exists() {
        return Err(ApplicationError::InvalidInput(format!(
            "Directory does not exist: {}",
            directory_path
        )));
    }

    let target_path = if let Some(ref relative_or_absolute_file) = file_path {
        let candidate_path = std::path::Path::new(relative_or_absolute_file);
        if candidate_path.is_absolute() {
            candidate_path.to_path_buf()
        } else {
            base_directory.join(candidate_path)
        }
    } else {
        base_directory.to_path_buf()
    };

    #[cfg(target_os = "linux")]
    {
        tokio::process::Command::new("xdg-open")
            .arg(&target_path)
            .spawn()
            .map_err(|error| {
                ApplicationError::Unexpected(format!("Failed to open file manager: {}", error))
            })?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        if file_path.is_some() {
            tokio::process::Command::new("open")
                .arg("-R")
                .arg(&target_path)
                .spawn()
                .map_err(|error| {
                    ApplicationError::Unexpected(format!("Failed to reveal file: {}", error))
                })?;
        } else {
            tokio::process::Command::new("open")
                .arg(&target_path)
                .spawn()
                .map_err(|error| {
                    ApplicationError::Unexpected(format!("Failed to open directory: {}", error))
                })?;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        if file_path.is_some() {
            tokio::process::Command::new("explorer")
                .arg(format!("/select,{}", target_path.to_string_lossy()))
                .spawn()
                .map_err(|error| {
                    ApplicationError::Unexpected(format!("Failed to reveal file: {}", error))
                })?;
        } else {
            tokio::process::Command::new("explorer")
                .arg(&target_path)
                .spawn()
                .map_err(|error| {
                    ApplicationError::Unexpected(format!("Failed to open directory: {}", error))
                })?;
        }
        Ok(())
    }
}

/// Tauri IPC command to execute multi-commit cherry-pick queue.
#[tauri::command]
pub async fn execute_cherry_pick_queue(
    state: State<'_, ApplicationState>,
    commit_hashes: Vec<String>,
    should_commit: bool,
    directory_path: Option<String>,
) -> Result<crate::domain::CommitQueueOperationResult, ApplicationError> {
    state
        .execute_cherry_pick_queue(directory_path.as_deref(), &commit_hashes, should_commit)
        .await
}

/// Tauri IPC command to execute multi-commit revert queue.
#[tauri::command]
pub async fn execute_revert_queue(
    state: State<'_, ApplicationState>,
    commit_hashes: Vec<String>,
    should_commit: bool,
    directory_path: Option<String>,
) -> Result<crate::domain::CommitQueueOperationResult, ApplicationError> {
    state
        .execute_revert_queue(directory_path.as_deref(), &commit_hashes, should_commit)
        .await
}

/// Tauri IPC command to create an offline Git bundle file.
#[tauri::command]
pub async fn create_git_bundle(
    state: State<'_, ApplicationState>,
    output_bundle_path: String,
    revision_range: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state
        .create_git_bundle(
            directory_path.as_deref(),
            &output_bundle_path,
            revision_range.as_deref(),
        )
        .await
}

/// Tauri IPC command to verify an offline Git bundle file.
#[tauri::command]
pub async fn verify_git_bundle(
    state: State<'_, ApplicationState>,
    bundle_path: String,
    directory_path: Option<String>,
) -> Result<crate::domain::GitBundleVerificationResult, ApplicationError> {
    state
        .verify_git_bundle(directory_path.as_deref(), &bundle_path)
        .await
}

/// Tauri IPC command to export a source code archive (.zip or .tar.gz).
#[tauri::command]
pub async fn export_source_archive(
    state: State<'_, ApplicationState>,
    output_archive_path: String,
    reference: String,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state
        .export_source_archive(directory_path.as_deref(), &output_archive_path, &reference)
        .await
}


