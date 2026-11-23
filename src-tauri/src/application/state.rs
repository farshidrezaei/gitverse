use crate::domain::{
    ApplicationDiagnostics, ApplicationError, CommitComparison, CommitDetails, ConflictFileDetails,
    DetailedRepositoryStatus, DirectoryListing, FileDiff, GitBinaryInformation, GitWorktreeSummary,
    InteractiveRebaseRequest, MergeBranchOutcome, ProviderAccount, ProviderKind,
    PullRequestChangedFile, PullRequestMergeResult, RebaseBranchOutcome, RebasePlanItem,
    RecentRepositoryRecord, RemoteProjectRepository, RemotePullRequest, RemoteReviewComment,
    RepositoryOverview, StashRecord, WorkflowJob, WorkflowRun,
};
use crate::infrastructure::git::{
    GitProcessExecutor, GitRepositoryInspector, RepositoryWatcherManager,
};
use crate::infrastructure::provider::{GitHubClient, GitLabClient};
use crate::infrastructure::security::CredentialStore;
use crate::infrastructure::storage::SQLiteRepositoryStore;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;

const MAX_CACHE_ENTRIES: usize = 250;

/// Global application state shared across Tauri commands.
pub struct ApplicationState {
    pub current_repository: Arc<RwLock<Option<RepositoryOverview>>>,
    pub store: Arc<SQLiteRepositoryStore>,
    pub credential_store: Arc<CredentialStore>,
    pub git_executor: Arc<GitProcessExecutor>,
    pub git_inspector: Arc<GitRepositoryInspector>,
    pub watcher_manager: Arc<RepositoryWatcherManager>,
    commit_details_cache: Arc<RwLock<HashMap<String, CommitDetails>>>,
    commit_file_diff_cache: Arc<RwLock<HashMap<String, FileDiff>>>,
}

impl ApplicationState {
    pub fn new(database_path: &Path) -> Result<Self, ApplicationError> {
        let store = Arc::new(SQLiteRepositoryStore::new(database_path)?);
        let credential_store = Arc::new(CredentialStore::new());
        let git_executor = Arc::new(GitProcessExecutor::new(PathBuf::from("git")));
        let git_inspector = Arc::new(GitRepositoryInspector::new((*git_executor).clone()));
        let watcher_manager = Arc::new(RepositoryWatcherManager::new());

        Ok(Self {
            current_repository: Arc::new(RwLock::new(None)),
            store,
            credential_store,
            git_executor,
            git_inspector,
            watcher_manager,
            commit_details_cache: Arc::new(RwLock::new(HashMap::new())),
            commit_file_diff_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Discovers system Git, returns binary metadata, and updates active executor if custom path is valid.
    pub async fn discover_git(
        &self,
        custom_path: Option<&str>,
    ) -> Result<GitBinaryInformation, ApplicationError> {
        let binary_info = GitProcessExecutor::discover_system_git(custom_path).await?;
        if custom_path.is_some() && binary_info.is_valid {
            self.set_git_executable_path(PathBuf::from(&binary_info.executable_path));
        }
        Ok(binary_info)
    }

    /// Updates the Git executable path used by the active executor and inspector.
    pub fn set_git_executable_path(&self, executable_path: PathBuf) {
        self.git_executor.set_executable_path(executable_path);
    }

    /// Opens a repository from the filesystem, records it in history, and starts file watcher.
    pub async fn open_repository(
        &self,
        app_handle: AppHandle,
        directory_path: &Path,
    ) -> Result<RepositoryOverview, ApplicationError> {
        let overview = self
            .git_inspector
            .inspect_repository(directory_path)
            .await?;

        // Persist to recent history
        let _ = self
            .store
            .record_opened_repository(&overview.path, &overview.name);

        // Update active repository in memory
        let mut active = self.current_repository.write().await;
        *active = Some(overview.clone());

        // Start real-time background file watching
        self.watcher_manager
            .watch_repository(app_handle, directory_path);

        Ok(overview)
    }

    /// Retrieves current active repository overview.
    pub async fn get_current_repository(&self) -> Option<RepositoryOverview> {
        let active = self.current_repository.read().await;
        active.clone()
    }

    /// Closes current active repository and stops file watcher.
    pub async fn close_current_repository(&self) {
        self.watcher_manager.stop_watching();
        let mut active = self.current_repository.write().await;
        *active = None;
        let mut details_cache = self.commit_details_cache.write().await;
        details_cache.clear();
        let mut diff_cache = self.commit_file_diff_cache.write().await;
        diff_cache.clear();
    }

    /// Gets detailed repository status (staged, unstaged, conflicted files).
    pub async fn get_repository_status(
        &self,
        directory_path: Option<&str>,
    ) -> Result<DetailedRepositoryStatus, ApplicationError> {
        let target_path = if let Some(path_str) = directory_path {
            PathBuf::from(path_str)
        } else {
            let active = self.current_repository.read().await;
            if let Some(ref current) = *active {
                PathBuf::from(&current.path)
            } else {
                return Err(ApplicationError::InvalidRepository(
                    "No active repository open".into(),
                ));
            }
        };

        self.git_inspector.get_detailed_status(&target_path).await
    }

    /// Stage a file.
    pub async fn stage_file(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.stage_file(&target_path, file_path).await
    }

    /// Unstage a file.
    pub async fn unstage_file(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .unstage_file(&target_path, file_path)
            .await
    }

    /// Stage all.
    pub async fn stage_all(&self, directory_path: Option<&str>) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.stage_all(&target_path).await
    }

    /// Unstage all.
    pub async fn unstage_all(&self, directory_path: Option<&str>) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.unstage_all(&target_path).await
    }

    /// Discard file changes.
    pub async fn discard_file_changes(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        is_untracked: bool,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .discard_file_changes(&target_path, file_path, is_untracked)
            .await
    }

    /// Stages a unified diff patch into the index.
    pub async fn stage_patch(
        &self,
        directory_path: Option<&str>,
        patch_content: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .stage_patch(&target_path, patch_content)
            .await
    }

    /// Unstages a unified diff patch from the index.
    pub async fn unstage_patch(
        &self,
        directory_path: Option<&str>,
        patch_content: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .unstage_patch(&target_path, patch_content)
            .await
    }

    /// Discards changes in a unified diff patch from the working tree.
    pub async fn discard_patch(
        &self,
        directory_path: Option<&str>,
        patch_content: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .discard_patch(&target_path, patch_content)
            .await
    }

    /// Commit staged changes with optional amend and hook bypass.
    pub async fn commit(
        &self,
        directory_path: Option<&str>,
        message: &str,
        is_amend: bool,
        bypass_hooks: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_commit(&status, message, is_amend)?;
        self.git_inspector
            .commit(&target_path, message, is_amend, bypass_hooks)
            .await
    }

    /// Retrieves structured diff for a file.
    pub async fn get_file_diff(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        is_staged: bool,
    ) -> Result<crate::domain::FileDiff, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_file_diff(&target_path, file_path, is_staged)
            .await
    }

    /// Retrieves configured commit template from repository or git config.
    pub async fn get_commit_template(
        &self,
        directory_path: Option<&str>,
    ) -> Result<Option<String>, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.get_commit_template(&target_path).await
    }

    /// Extracts repository reflog records with optional reference target and limit.
    pub async fn get_reflog(
        &self,
        directory_path: Option<&str>,
        reference: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<crate::domain::ReflogRecord>, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_reflog(&target_path, reference.as_deref(), limit.unwrap_or(100))
            .await
    }

    /// Extracts commit history and builds deterministic commit graph DAG.
    pub async fn get_commit_graph(
        &self,
        directory_path: Option<&str>,
        skip: Option<usize>,
        limit: Option<usize>,
        branch_or_reference: Option<String>,
    ) -> Result<crate::domain::CommitGraphHistoryResponse, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let skip_count = skip.unwrap_or(0);
        let limit_count = limit.unwrap_or(100);

        let commits = self
            .git_inspector
            .get_commit_history(
                &target_path,
                skip_count,
                limit_count,
                branch_or_reference.as_deref(),
            )
            .await?;

        let graph = crate::services::CommitGraphEngine::build_graph(commits);
        Ok(graph)
    }

    /// Fetches all repository references: branches, remotes, and tags.
    pub async fn get_repository_references(
        &self,
        directory_path: Option<&str>,
    ) -> Result<crate::domain::RepositoryReferences, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_repository_references(&target_path)
            .await
    }

    /// Checks out or creates and checkouts a branch.
    pub async fn checkout_branch(
        &self,
        directory_path: Option<&str>,
        branch_name: &str,
        create_if_missing: bool,
        start_point: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        if !create_if_missing {
            let status = self.git_inspector.get_detailed_status(&target_path).await?;
            let worktrees = self
                .git_inspector
                .list_worktrees(&target_path)
                .await
                .unwrap_or_default();
            crate::domain::preflight::verify_can_checkout(&status, branch_name, &worktrees)?;
        }
        self.git_inspector
            .checkout_branch(&target_path, branch_name, create_if_missing, start_point)
            .await
    }

    /// Creates a branch without switching to it.
    pub async fn create_branch(
        &self,
        directory_path: Option<&str>,
        branch_name: &str,
        start_point: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .create_branch(&target_path, branch_name, start_point)
            .await
    }

    /// Deletes a local branch.
    pub async fn delete_branch(
        &self,
        directory_path: Option<&str>,
        branch_name: &str,
        force: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        let worktrees = self
            .git_inspector
            .list_worktrees(&target_path)
            .await
            .unwrap_or_default();
        crate::domain::preflight::verify_can_delete_branch(&status, branch_name, &worktrees)?;
        self.git_inspector
            .delete_branch(&target_path, branch_name, force)
            .await
    }

    /// Renames a local branch.
    pub async fn rename_branch(
        &self,
        directory_path: Option<&str>,
        old_name: &str,
        new_name: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .rename_branch(&target_path, old_name, new_name)
            .await
    }

    /// Fetches changes from remote.
    pub async fn fetch_remote(
        &self,
        directory_path: Option<&str>,
        remote_name: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .fetch_remote(&target_path, remote_name)
            .await
    }

    /// Pulls changes from upstream.
    pub async fn pull_changes(
        &self,
        directory_path: Option<&str>,
        remote_name: Option<&str>,
        branch_name: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .pull_changes(&target_path, remote_name, branch_name)
            .await
    }

    /// Pushes commits to remote.
    pub async fn push_changes(
        &self,
        directory_path: Option<&str>,
        remote_name: Option<&str>,
        branch_name: Option<&str>,
        set_upstream: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .push_changes(&target_path, remote_name, branch_name, set_upstream)
            .await
    }

    /// Adds a new Git remote.
    pub async fn add_remote(
        &self,
        directory_path: Option<&str>,
        name: &str,
        url: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.add_remote(&target_path, name, url).await
    }

    /// Removes a Git remote.
    pub async fn remove_remote(
        &self,
        directory_path: Option<&str>,
        name: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.remove_remote(&target_path, name).await
    }

    /// Renames a Git remote.
    pub async fn rename_remote(
        &self,
        directory_path: Option<&str>,
        old_name: &str,
        new_name: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .rename_remote(&target_path, old_name, new_name)
            .await
    }

    /// Sets the URL of a Git remote.
    pub async fn set_remote_url(
        &self,
        directory_path: Option<&str>,
        name: &str,
        url: &str,
        is_push_only: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .set_remote_url(&target_path, name, url, is_push_only)
            .await
    }

    /// Fetches full commit details including changed files. Uses in-memory cache to eliminate duplicate Git invocations.
    pub async fn get_commit_details(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
    ) -> Result<CommitDetails, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let cache_key = format!("{}:{}", target_path.to_string_lossy(), commit_hash);

        {
            let cache = self.commit_details_cache.read().await;
            if let Some(details) = cache.get(&cache_key) {
                return Ok(details.clone());
            }
        }

        let details = self
            .git_inspector
            .get_commit_details(&target_path, commit_hash)
            .await?;

        {
            let mut cache = self.commit_details_cache.write().await;
            if cache.len() >= MAX_CACHE_ENTRIES {
                cache.clear();
            }
            cache.insert(cache_key, details.clone());
        }

        Ok(details)
    }

    /// Fetches unified diff for a single file in a commit or the whole commit with caching.
    pub async fn get_commit_file_diff(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
        file_path: Option<&str>,
    ) -> Result<FileDiff, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let cache_key = format!(
            "{}:{}:{}",
            target_path.to_string_lossy(),
            commit_hash,
            file_path.unwrap_or("")
        );

        {
            let cache = self.commit_file_diff_cache.read().await;
            if let Some(diff) = cache.get(&cache_key) {
                return Ok(diff.clone());
            }
        }

        let diff = self
            .git_inspector
            .get_commit_file_diff(&target_path, commit_hash, file_path)
            .await?;

        {
            let mut cache = self.commit_file_diff_cache.write().await;
            if cache.len() >= MAX_CACHE_ENTRIES {
                cache.clear();
            }
            cache.insert(cache_key, diff.clone());
        }

        Ok(diff)
    }

    /// Compares two commits or references and returns changed files list and metadata.
    pub async fn compare_commits(
        &self,
        directory_path: Option<&str>,
        base_hash: &str,
        target_hash: &str,
    ) -> Result<CommitComparison, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .compare_commits(&target_path, base_hash, target_hash)
            .await
    }

    /// Fetches unified diff for a single file or entire comparison between two commits.
    pub async fn get_comparison_file_diff(
        &self,
        directory_path: Option<&str>,
        base_hash: &str,
        target_hash: &str,
        file_path: Option<&str>,
    ) -> Result<FileDiff, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_comparison_file_diff(&target_path, base_hash, target_hash, file_path)
            .await
    }

    /// Lists stash records for repository.
    pub async fn get_stash_list(
        &self,
        directory_path: Option<&str>,
    ) -> Result<Vec<StashRecord>, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.list_stashes(&target_path).await
    }

    /// Saves working directory changes to a new stash entry.
    pub async fn save_stash(
        &self,
        directory_path: Option<&str>,
        message: Option<&str>,
        include_untracked: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_stash_save(&status)?;
        self.git_inspector
            .save_stash(&target_path, message, include_untracked)
            .await
    }

    /// Applies a stash entry without removing it.
    pub async fn apply_stash(
        &self,
        directory_path: Option<&str>,
        stash_index: usize,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .apply_stash(&target_path, stash_index)
            .await
    }

    /// Pops a stash entry, applying and dropping it.
    pub async fn pop_stash(
        &self,
        directory_path: Option<&str>,
        stash_index: usize,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .pop_stash(&target_path, stash_index)
            .await
    }

    /// Drops a stash entry from the stack.
    pub async fn drop_stash(
        &self,
        directory_path: Option<&str>,
        stash_index: usize,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .drop_stash(&target_path, stash_index)
            .await
    }

    /// Retrieves detailed stash inspection including changed files list.
    pub async fn get_stash_details(
        &self,
        directory_path: Option<&str>,
        stash_index: usize,
    ) -> Result<crate::domain::StashDetails, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_stash_details(&target_path, stash_index)
            .await
    }

    /// Retrieves file diff within a stash entry.
    pub async fn get_stash_file_diff(
        &self,
        directory_path: Option<&str>,
        stash_index: usize,
        file_path: &str,
    ) -> Result<FileDiff, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_stash_file_diff(&target_path, stash_index, file_path)
            .await
    }

    /// Cherry-picks a commit onto current branch.
    pub async fn cherry_pick_commit(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_cherry_pick(&status)?;
        self.git_inspector
            .cherry_pick_commit(&target_path, commit_hash)
            .await
    }

    /// Reverts a commit by creating an inverse commit.
    pub async fn revert_commit(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_revert(&status)?;
        self.git_inspector
            .revert_commit(&target_path, commit_hash)
            .await
    }

    /// Resets the current branch to a specific commit.
    pub async fn reset_to_commit(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
        reset_mode: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .reset_to_commit(&target_path, commit_hash, reset_mode)
            .await
    }

    /// Retrieves 3-way conflict stages and working file content for a conflicted file.
    pub async fn get_conflict_file_details(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
    ) -> Result<ConflictFileDetails, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_conflict_file_details(&target_path, file_path)
            .await
    }

    /// Resolves a conflicted file using a preset strategy ('ours', 'theirs', 'mark_resolved').
    pub async fn resolve_conflict_file(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        resolution_strategy: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .resolve_conflict_file(&target_path, file_path, resolution_strategy)
            .await
    }

    /// Saves custom resolved content directly to disk and marks the file as resolved.
    pub async fn write_conflict_resolved_content(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        resolved_content: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .write_conflict_resolved_content(&target_path, file_path, resolved_content)
            .await
    }

    /// Merges branch into current HEAD branch.
    pub async fn merge_branch(
        &self,
        directory_path: Option<&str>,
        branch_name: &str,
        no_fast_forward: bool,
    ) -> Result<MergeBranchOutcome, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_merge(&status, branch_name)?;
        self.git_inspector
            .merge_branch(&target_path, branch_name, no_fast_forward)
            .await
    }

    /// Rebases current HEAD branch onto upstream branch.
    pub async fn rebase_branch(
        &self,
        directory_path: Option<&str>,
        upstream_branch: &str,
    ) -> Result<RebaseBranchOutcome, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_rebase(&status, upstream_branch)?;
        self.git_inspector
            .rebase_branch(&target_path, upstream_branch)
            .await
    }

    /// Aborts active ongoing operation (merge, rebase, cherry-pick, revert).
    pub async fn abort_repository_operation(
        &self,
        directory_path: Option<&str>,
        create_backup: Option<bool>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_abort_operation(&status)?;
        self.git_inspector
            .abort_repository_operation(&target_path, create_backup.unwrap_or(false))
            .await
    }

    /// Continues active ongoing operation after all conflicts are resolved.
    pub async fn continue_repository_operation(
        &self,
        directory_path: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let status = self.git_inspector.get_detailed_status(&target_path).await?;
        crate::domain::preflight::verify_can_continue_operation(&status)?;
        self.git_inspector
            .continue_repository_operation(&target_path)
            .await
    }

    async fn resolve_path(
        &self,
        directory_path: Option<&str>,
    ) -> Result<PathBuf, ApplicationError> {
        if let Some(p) = directory_path {
            Ok(PathBuf::from(p))
        } else {
            let active = self.current_repository.read().await;
            if let Some(ref current) = *active {
                Ok(PathBuf::from(&current.path))
            } else {
                Err(ApplicationError::InvalidRepository(
                    "No active repository open".into(),
                ))
            }
        }
    }

    /// Retrieves recent repositories list.
    pub fn get_recent_repositories(
        &self,
        limit: usize,
    ) -> Result<Vec<RecentRepositoryRecord>, ApplicationError> {
        self.store.get_recent_repositories(limit)
    }

    /// Removes a repository from the recents list.
    pub fn remove_recent_repository(&self, path: &str) -> Result<(), ApplicationError> {
        self.store.remove_recent_repository(path)
    }

    /// Clears all repositories from the recents list.
    pub fn clear_recent_repositories(&self) -> Result<(), ApplicationError> {
        self.store.clear_recent_repositories()
    }

    /// Lists subfolders in a given directory path for custom folder browsing.
    pub async fn list_directory_folders(
        &self,
        directory_path: Option<&str>,
    ) -> Result<DirectoryListing, ApplicationError> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let initial_path = if let Some(p) = directory_path {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                home_dir.clone()
            } else if trimmed.starts_with("~/") || trimmed == "~" {
                if trimmed == "~" {
                    home_dir.clone()
                } else {
                    home_dir.join(&trimmed[2..])
                }
            } else {
                PathBuf::from(trimmed)
            }
        } else {
            home_dir.clone()
        };

        let mut target_dir = initial_path;

        // If target path does not exist on disk, traverse ancestors until an existing folder is reached
        while !target_dir.exists() {
            if let Some(parent) = target_dir.parent() {
                if parent.as_os_str().is_empty() || parent == target_dir {
                    target_dir = home_dir.clone();
                    break;
                }
                target_dir = parent.to_path_buf();
            } else {
                target_dir = home_dir.clone();
                break;
            }
        }

        // If target path is a file, use its parent directory
        if target_dir.is_file() {
            if let Some(parent) = target_dir.parent() {
                target_dir = parent.to_path_buf();
            } else {
                target_dir = home_dir;
            }
        }

        self.git_inspector.list_directory_folders(&target_dir).await
    }

    /// Lists worktrees linked to the repository.
    pub async fn list_worktrees(
        &self,
        directory_path: Option<&str>,
    ) -> Result<Vec<GitWorktreeSummary>, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.list_worktrees(&target_path).await
    }

    /// Adds a new worktree to the repository.
    pub async fn add_worktree(
        &self,
        directory_path: Option<&str>,
        worktree_path: &str,
        branch_name: Option<&str>,
        is_new_branch: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .add_worktree(&target_path, worktree_path, branch_name, is_new_branch)
            .await
    }

    /// Removes a worktree from the repository.
    pub async fn remove_worktree(
        &self,
        directory_path: Option<&str>,
        worktree_path: &str,
        force: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .remove_worktree(&target_path, worktree_path, force)
            .await
    }

    /// Fetches commits between base commit and HEAD for interactive rebase plan.
    pub async fn get_rebase_commits(
        &self,
        directory_path: Option<&str>,
        base_commit_hash: &str,
    ) -> Result<Vec<RebasePlanItem>, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_rebase_commits(&target_path, base_commit_hash)
            .await
    }

    /// Executes interactive rebase according to plan items.
    pub async fn execute_interactive_rebase(
        &self,
        directory_path: Option<&str>,
        request: InteractiveRebaseRequest,
    ) -> Result<RebaseBranchOutcome, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .execute_interactive_rebase(&target_path, &request)
            .await
    }

    /// Retrieves file blame annotations for a file path.
    pub async fn get_file_blame(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<crate::domain::FileBlame, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_file_blame(&target_path, file_path, revision)
            .await
    }

    /// Queries the current bisect status for a repository.
    pub async fn get_bisect_status(
        &self,
        directory_path: Option<&str>,
    ) -> Result<crate::domain::BisectState, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.get_bisect_status(&target_path).await
    }

    /// Starts a Git bisect session.
    pub async fn bisect_start(
        &self,
        directory_path: Option<&str>,
        bad_commit_hash: &str,
        good_commit_hash: &str,
    ) -> Result<crate::domain::BisectState, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .bisect_start(&target_path, bad_commit_hash, good_commit_hash)
            .await
    }

    /// Advances a Git bisect session with a decision (good, bad, skip).
    pub async fn bisect_step(
        &self,
        directory_path: Option<&str>,
        decision: &str,
    ) -> Result<crate::domain::BisectState, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.bisect_step(&target_path, decision).await
    }

    /// Resets the Git bisect session.
    pub async fn bisect_reset(&self, directory_path: Option<&str>) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.bisect_reset(&target_path).await
    }

    /// Exports repository changes or a commit as a patch string.
    pub async fn export_patch(
        &self,
        directory_path: Option<&str>,
        scope: &str,
        commit_hash: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .export_patch(&target_path, scope, commit_hash)
            .await
    }

    /// Validates if a unified diff patch can be cleanly applied.
    pub async fn check_patch(
        &self,
        directory_path: Option<&str>,
        patch_content: &str,
    ) -> Result<crate::domain::PatchCheckResult, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .check_patch(&target_path, patch_content)
            .await
    }

    /// Applies a patch to working tree or staging index.
    pub async fn apply_patch_file(
        &self,
        directory_path: Option<&str>,
        patch_content: &str,
        is_cached: bool,
        is_reverse: bool,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .apply_patch_file(&target_path, patch_content, is_cached, is_reverse)
            .await
    }

    /// Fetches all registered Git submodules and their status.
    pub async fn get_submodules(
        &self,
        directory_path: Option<&str>,
    ) -> Result<Vec<crate::domain::GitSubmoduleSummary>, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.get_submodules(&target_path).await
    }

    /// Updates one or all Git submodules.
    pub async fn update_submodules(
        &self,
        directory_path: Option<&str>,
        submodule_path: Option<&str>,
        is_init: bool,
        is_recursive: bool,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .update_submodules(&target_path, submodule_path, is_init, is_recursive)
            .await
    }

    /// Synchronizes submodules' remote URL configuration setting.
    pub async fn sync_submodules(
        &self,
        directory_path: Option<&str>,
        submodule_path: Option<&str>,
        is_recursive: bool,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .sync_submodules(&target_path, submodule_path, is_recursive)
            .await
    }

    /// Computes repository analytics and contributor velocity metrics.
    pub async fn get_repository_insights(
        &self,
        directory_path: Option<&str>,
    ) -> Result<crate::domain::RepositoryInsights, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_repository_insights(&target_path)
            .await
    }

    /// Fetches historical commit revisions affecting a specific file.
    pub async fn get_file_history(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        limit: Option<usize>,
    ) -> Result<crate::domain::FileHistoryTimeline, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_file_history(&target_path, file_path, limit)
            .await
    }

    /// Fetches file content at a specific historical commit revision.
    pub async fn get_file_content_at_commit(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
        file_path: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .get_file_content_at_commit(&target_path, commit_hash, file_path)
            .await
    }

    /// Searches repository files or historical revisions using git grep.
    pub async fn search_repository(
        &self,
        directory_path: Option<&str>,
        query: &str,
        reference: Option<&str>,
        path_pattern: Option<&str>,
        is_case_sensitive: bool,
        is_word_match: bool,
        is_regex: bool,
    ) -> Result<crate::domain::RepositorySearchResult, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .search_repository(
                &target_path,
                query,
                reference,
                path_pattern,
                is_case_sensitive,
                is_word_match,
                is_regex,
            )
            .await
    }

    /// Collects system diagnostics and environment details for voluntary bug reporting.
    pub async fn get_application_diagnostics(
        &self,
    ) -> Result<ApplicationDiagnostics, ApplicationError> {
        let git_info = self.discover_git(None).await?;
        let current_repo = self.current_repository.read().await;
        let repo_path = current_repo.as_ref().map(|r| r.path.clone());

        Ok(ApplicationDiagnostics {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_name: std::env::consts::OS.to_string(),
            os_arch: std::env::consts::ARCH.to_string(),
            git_binary_path: git_info.executable_path,
            git_version: git_info.version_string,
            current_repository_path: repo_path,
        })
    }

    /// Exports a complete, sanitized diagnostic bundle with environment info and recent logs.
    pub async fn export_diagnostic_bundle(
        &self,
    ) -> Result<crate::infrastructure::diagnostics::DiagnosticBundle, ApplicationError> {
        let git_info = self.discover_git(None).await?;
        let current_repo = self.current_repository.read().await;
        let repo_path = current_repo.as_ref().map(|r| r.path.clone());
        let accounts = self.store.list_provider_accounts().unwrap_or_default();
        let connected_providers = accounts
            .into_iter()
            .map(|account| format!("{}: {}", account.provider, account.username))
            .collect();
        let recent_logs = crate::infrastructure::diagnostics::get_recent_diagnostic_logs(100);

        Ok(crate::infrastructure::diagnostics::DiagnosticBundle {
            application_version: env!("CARGO_PKG_VERSION").to_string(),
            operating_system: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            git_binary_path: git_info.executable_path,
            git_version: git_info.version_string,
            current_repository_path: repo_path,
            generated_at_iso: chrono::Utc::now().to_rfc3339(),
            connected_providers,
            recent_log_entries: recent_logs,
        })
    }

    /// Connects and validates a GitHub account using personal access token.
    pub async fn connect_github_account(
        &self,
        personal_access_token: &str,
        instance_url: Option<&str>,
    ) -> Result<ProviderAccount, ApplicationError> {
        let instance = instance_url.unwrap_or("https://github.com");
        let client = GitHubClient::new(Some(instance))?;
        let account = client
            .get_authenticated_user(personal_access_token, instance)
            .await?;

        // Securely store token
        self.credential_store
            .store_token(&account.id, personal_access_token)?;

        // Persist account profile
        self.store.save_provider_account(&account)?;

        Ok(account)
    }

    /// Connects and validates a GitLab account using personal access token.
    pub async fn connect_gitlab_account(
        &self,
        personal_access_token: &str,
        instance_url: Option<&str>,
    ) -> Result<ProviderAccount, ApplicationError> {
        let instance = instance_url.unwrap_or("https://gitlab.com");
        let client = GitLabClient::new(Some(instance))?;
        let account = client
            .get_authenticated_user(personal_access_token, instance)
            .await?;

        // Securely store token
        self.credential_store
            .store_token(&account.id, personal_access_token)?;

        // Persist account profile
        self.store.save_provider_account(&account)?;

        Ok(account)
    }

    /// Lists all connected remote provider accounts.
    pub async fn list_provider_accounts(&self) -> Result<Vec<ProviderAccount>, ApplicationError> {
        self.store.list_provider_accounts()
    }

    /// Disconnects a remote provider account and wipes secure credentials.
    pub async fn disconnect_provider_account(
        &self,
        account_identifier: &str,
    ) -> Result<bool, ApplicationError> {
        self.credential_store.delete_token(account_identifier)?;
        self.store.delete_provider_account(account_identifier)
    }

    /// Retrieves remote repositories/projects for a connected account.
    pub async fn get_provider_repositories(
        &self,
        account_identifier: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<Vec<RemoteProjectRepository>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        let page_number = page.unwrap_or(1);
        let items_per_page = per_page.unwrap_or(30);

        match account.provider {
            ProviderKind::GitHub => {
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .list_repositories(&token, page_number, items_per_page)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .list_projects(&token, page_number, items_per_page)
                    .await
            }
        }
    }

    /// Retrieves pull requests or merge requests for a repository.
    pub async fn get_provider_pull_requests(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        state_filter: Option<&str>,
    ) -> Result<Vec<RemotePullRequest>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .list_pull_requests(&token, parts[0], parts[1], state_filter)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .list_merge_requests(&token, repository_full_name, state_filter)
                    .await
            }
        }
    }

    /// Retrieves commit pipeline status summary for a commit or branch.
    pub async fn get_commit_pipeline_summary(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        reference_or_sha: &str,
    ) -> Result<Option<crate::domain::PipelineSummary>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .get_commit_pipeline_summary(&token, parts[0], parts[1], reference_or_sha)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .get_commit_pipeline_summary(&token, repository_full_name, reference_or_sha)
                    .await
            }
        }
    }

    /// Retrieves detailed check runs or CI jobs for a commit or pipeline.
    pub async fn list_check_runs(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        reference_or_pipeline_id: &str,
    ) -> Result<Vec<crate::domain::CiCheckRun>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .list_check_runs(&token, parts[0], parts[1], reference_or_pipeline_id)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                let mut pipeline_id = reference_or_pipeline_id.parse::<u64>().unwrap_or(0);
                if pipeline_id == 0
                    && let Ok(Some(summary)) = client
                        .get_commit_pipeline_summary(
                            &token,
                            repository_full_name,
                            reference_or_pipeline_id,
                        )
                        .await
                    && let Some(ref web_url) = summary.web_url
                    && let Some(last_segment) = web_url.split('/').next_back()
                {
                    pipeline_id = last_segment.parse::<u64>().unwrap_or(0);
                }
                client
                    .list_pipeline_jobs(&token, repository_full_name, pipeline_id)
                    .await
            }
        }
    }

    /// Fetches all comments and review notes for a pull or merge request.
    pub async fn get_pull_request_comments(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        pull_request_number: u64,
    ) -> Result<Vec<RemoteReviewComment>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .get_pull_request_comments(&token, parts[0], parts[1], pull_request_number)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .get_merge_request_comments(&token, repository_full_name, pull_request_number)
                    .await
            }
        }
    }

    /// Posts a new comment to a pull or merge request.
    pub async fn create_pull_request_comment(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        pull_request_number: u64,
        body: &str,
    ) -> Result<RemoteReviewComment, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .create_pull_request_comment(
                        &token,
                        parts[0],
                        parts[1],
                        pull_request_number,
                        body,
                    )
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .create_merge_request_comment(
                        &token,
                        repository_full_name,
                        pull_request_number,
                        body,
                    )
                    .await
            }
        }
    }

    /// Retrieves list of changed files and diff patches for a pull or merge request.
    pub async fn get_pull_request_files(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        pull_request_number: u64,
    ) -> Result<Vec<PullRequestChangedFile>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .get_pull_request_files(&token, parts[0], parts[1], pull_request_number)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .get_merge_request_files(&token, repository_full_name, pull_request_number)
                    .await
            }
        }
    }

    /// Submits a review (Approve, Request Changes, Comment) on a pull or merge request.
    pub async fn submit_pull_request_review(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        pull_request_number: u64,
        action: &str,
        body: Option<&str>,
    ) -> Result<bool, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .submit_pull_request_review(
                        &token,
                        parts[0],
                        parts[1],
                        pull_request_number,
                        action,
                        body,
                    )
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .submit_merge_request_review(
                        &token,
                        repository_full_name,
                        pull_request_number,
                        action,
                        body,
                    )
                    .await
            }
        }
    }

    /// Merges a pull request or merge request with chosen method.
    pub async fn merge_provider_pull_request(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        pull_request_number: u64,
        merge_method: Option<&str>,
        commit_title: Option<&str>,
        commit_message: Option<&str>,
        delete_source_branch: Option<bool>,
    ) -> Result<PullRequestMergeResult, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .merge_pull_request(
                        &token,
                        parts[0],
                        parts[1],
                        pull_request_number,
                        merge_method,
                        commit_title,
                        commit_message,
                    )
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                let is_squash = merge_method == Some("squash");
                client
                    .merge_merge_request(
                        &token,
                        repository_full_name,
                        pull_request_number,
                        commit_message,
                        Some(is_squash),
                        delete_source_branch,
                    )
                    .await
            }
        }
    }

    /// Updates pull request lifecycle state (closed or open).
    pub async fn update_pull_request_state(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        pull_request_number: u64,
        new_state: &str,
    ) -> Result<bool, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .update_pull_request_state(
                        &token,
                        parts[0],
                        parts[1],
                        pull_request_number,
                        new_state,
                    )
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .update_merge_request_state(
                        &token,
                        repository_full_name,
                        pull_request_number,
                        new_state,
                    )
                    .await
            }
        }
    }

    /// Executes sequential cherry-pick queue across multiple commit hashes.
    pub async fn execute_cherry_pick_queue(
        &self,
        directory_path: Option<&str>,
        commit_hashes: &[String],
        should_commit: bool,
    ) -> Result<crate::domain::CommitQueueOperationResult, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .execute_cherry_pick_queue(&target_path, commit_hashes, should_commit)
            .await
    }

    /// Executes sequential revert queue across multiple commit hashes.
    pub async fn execute_revert_queue(
        &self,
        directory_path: Option<&str>,
        commit_hashes: &[String],
        should_commit: bool,
    ) -> Result<crate::domain::CommitQueueOperationResult, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .execute_revert_queue(&target_path, commit_hashes, should_commit)
            .await
    }

    /// Creates an offline Git bundle file packaging branches or revisions.
    pub async fn create_git_bundle(
        &self,
        directory_path: Option<&str>,
        output_bundle_path: &str,
        revision_range: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .create_git_bundle(
                &target_path,
                std::path::Path::new(output_bundle_path),
                revision_range,
            )
            .await
    }

    /// Verifies integrity and references contained within an offline Git bundle file.
    pub async fn verify_git_bundle(
        &self,
        directory_path: Option<&str>,
        bundle_path: &str,
    ) -> Result<crate::domain::GitBundleVerificationResult, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .verify_git_bundle(&target_path, std::path::Path::new(bundle_path))
            .await
    }

    /// Exports repository snapshot at a reference into a compressed archive (.zip or .tar.gz).
    pub async fn export_source_archive(
        &self,
        directory_path: Option<&str>,
        output_archive_path: &str,
        reference: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .export_source_archive(
                &target_path,
                std::path::Path::new(output_archive_path),
                reference,
            )
            .await
    }

    /// Retrieves Git LFS overview, tracked patterns, tracked files, and locks.
    pub async fn get_lfs_overview(
        &self,
        directory_path: Option<&str>,
    ) -> Result<crate::domain::GitLfsOverview, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.get_lfs_overview(&target_path).await
    }

    /// Tracks a file pattern in Git LFS.
    pub async fn lfs_track_pattern(
        &self,
        directory_path: Option<&str>,
        pattern: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .lfs_track_pattern(&target_path, pattern)
            .await
    }

    /// Untracks a file pattern from Git LFS.
    pub async fn lfs_untrack_pattern(
        &self,
        directory_path: Option<&str>,
        pattern: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .lfs_untrack_pattern(&target_path, pattern)
            .await
    }

    /// Locks an LFS file to prevent concurrent remote edits.
    pub async fn lfs_lock_file(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .lfs_lock_file(&target_path, file_path)
            .await
    }

    /// Unlocks an LFS file.
    pub async fn lfs_unlock_file(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        force: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector
            .lfs_unlock_file(&target_path, file_path, force)
            .await
    }

    /// Lists configured custom actions.
    pub fn list_custom_actions(
        &self,
    ) -> Result<Vec<crate::domain::CustomActionDefinition>, ApplicationError> {
        self.store.list_custom_actions()
    }

    /// Saves or updates a custom action.
    pub fn save_custom_action(
        &self,
        action: crate::domain::CustomActionDefinition,
    ) -> Result<(), ApplicationError> {
        crate::domain::validation::validate_custom_action_definition(&action)?;
        self.store.save_custom_action(&action)
    }

    /// Deletes a custom action.
    pub fn delete_custom_action(&self, action_identifier: &str) -> Result<bool, ApplicationError> {
        self.store.delete_custom_action(action_identifier)
    }

    /// Executes a custom action command in the repository workspace.
    pub async fn execute_custom_action(
        &self,
        directory_path: Option<&str>,
        action_identifier: &str,
    ) -> Result<crate::domain::CustomActionExecutionResult, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        let actions = self.store.list_custom_actions()?;
        let action = actions
            .into_iter()
            .find(|item| item.action_identifier == action_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Custom action not found".into()))?;

        // Parse command line into executable and arguments
        let parts: Vec<&str> = action.command_string.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "Command string is empty".into(),
            ));
        }

        let executable = parts[0];
        let arguments = &parts[1..];

        let execution_outcome = tokio::process::Command::new(executable)
            .args(arguments)
            .current_dir(&target_path)
            .output()
            .await;

        match execution_outcome {
            Ok(output) => {
                let standard_output = String::from_utf8_lossy(&output.stdout).to_string();
                let standard_error = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);
                Ok(crate::domain::CustomActionExecutionResult {
                    action_identifier: action.action_identifier,
                    is_successful: output.status.success(),
                    exit_code,
                    standard_output,
                    standard_error,
                })
            }
            Err(error) => Ok(crate::domain::CustomActionExecutionResult {
                action_identifier: action.action_identifier,
                is_successful: false,
                exit_code: -1,
                standard_output: String::new(),
                standard_error: format!("Failed to spawn process: {}", error),
            }),
        }
    }

    /// Lists workflow runs / pipelines for a provider repository.
    pub async fn list_workflow_runs(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        status_filter: Option<&str>,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<WorkflowRun>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .list_workflow_runs(&token, parts[0], parts[1], status_filter)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .list_pipelines(&token, repository_full_name, status_filter, page, per_page)
                    .await
            }
        }
    }

    /// Gets jobs in a workflow run / pipeline.
    pub async fn get_workflow_run_jobs(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        run_id: u64,
    ) -> Result<Vec<WorkflowJob>, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .get_workflow_run_jobs(&token, parts[0], parts[1], run_id)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .get_pipeline_jobs(&token, repository_full_name, run_id)
                    .await
            }
        }
    }

    /// Re-runs a workflow run or pipeline.
    pub async fn rerun_workflow_run(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        run_id: u64,
        failed_jobs_only: bool,
    ) -> Result<bool, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .rerun_workflow_run(&token, parts[0], parts[1], run_id, failed_jobs_only)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .retry_pipeline(&token, repository_full_name, run_id)
                    .await
            }
        }
    }

    /// Cancels an in-progress workflow run or pipeline.
    pub async fn cancel_workflow_run(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        run_id: u64,
    ) -> Result<bool, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .cancel_workflow_run(&token, parts[0], parts[1], run_id)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .cancel_pipeline(&token, repository_full_name, run_id)
                    .await
            }
        }
    }

    /// Fetches execution logs for a workflow job.
    pub async fn get_job_logs(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        job_id: u64,
    ) -> Result<String, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .get_job_logs(&token, parts[0], parts[1], job_id)
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .get_job_trace(&token, repository_full_name, job_id)
                    .await
            }
        }
    }

    /// Creates a new pull request or merge request on the remote provider.
    pub async fn create_pull_request(
        &self,
        account_identifier: &str,
        repository_full_name: &str,
        title: &str,
        description: Option<&str>,
        source_branch: &str,
        target_branch: &str,
        is_draft: bool,
    ) -> Result<RemotePullRequest, ApplicationError> {
        let accounts = self.store.list_provider_accounts()?;
        let account = accounts
            .into_iter()
            .find(|item| item.id == account_identifier)
            .ok_or_else(|| ApplicationError::InvalidInput("Account not found".into()))?;

        let token = self
            .credential_store
            .get_token(account_identifier)?
            .ok_or_else(|| {
                ApplicationError::AuthenticationError("Stored credential not found".into())
            })?;

        match account.provider {
            ProviderKind::GitHub => {
                let parts: Vec<&str> = repository_full_name.split('/').collect();
                if parts.len() != 2 {
                    return Err(ApplicationError::InvalidInput(
                        "Repository must be in 'owner/repo' format".into(),
                    ));
                }
                let client = GitHubClient::new(Some(&account.instance_url))?;
                client
                    .create_pull_request(
                        &token,
                        parts[0],
                        parts[1],
                        title,
                        description,
                        source_branch,
                        target_branch,
                        is_draft,
                    )
                    .await
            }
            ProviderKind::GitLab => {
                let client = GitLabClient::new(Some(&account.instance_url))?;
                client
                    .create_merge_request(
                        &token,
                        repository_full_name,
                        title,
                        description,
                        source_branch,
                        target_branch,
                        is_draft,
                    )
                    .await
            }
        }
    }
}
