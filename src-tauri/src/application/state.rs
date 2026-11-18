use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;
use crate::domain::{
    ApplicationError, CommitComparison, CommitDetails, ConflictFileDetails, DetailedRepositoryStatus, FileDiff, GitBinaryInformation,
    MergeBranchOutcome, RebaseBranchOutcome, RecentRepositoryRecord, RepositoryOverview, StashRecord,
};
use crate::infrastructure::git::{GitProcessExecutor, GitRepositoryInspector, RepositoryWatcherManager};
use crate::infrastructure::storage::SQLiteRepositoryStore;

const MAX_CACHE_ENTRIES: usize = 250;

/// Global application state shared across Tauri commands.
pub struct ApplicationState {
    pub current_repository: Arc<RwLock<Option<RepositoryOverview>>>,
    pub store: Arc<SQLiteRepositoryStore>,
    pub git_executor: Arc<GitProcessExecutor>,
    pub git_inspector: Arc<GitRepositoryInspector>,
    pub watcher_manager: Arc<RepositoryWatcherManager>,
    commit_details_cache: Arc<RwLock<HashMap<String, CommitDetails>>>,
    commit_file_diff_cache: Arc<RwLock<HashMap<String, FileDiff>>>,
}

impl ApplicationState {
    pub fn new(database_path: &Path) -> Result<Self, ApplicationError> {
        let store = Arc::new(SQLiteRepositoryStore::new(database_path)?);
        let git_executor = Arc::new(GitProcessExecutor::new(PathBuf::from("git")));
        let git_inspector = Arc::new(GitRepositoryInspector::new((*git_executor).clone()));
        let watcher_manager = Arc::new(RepositoryWatcherManager::new());

        Ok(Self {
            current_repository: Arc::new(RwLock::new(None)),
            store,
            git_executor,
            git_inspector,
            watcher_manager,
            commit_details_cache: Arc::new(RwLock::new(HashMap::new())),
            commit_file_diff_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Discovers system Git and returns binary metadata.
    pub async fn discover_git(&self, custom_path: Option<&str>) -> Result<GitBinaryInformation, ApplicationError> {
        GitProcessExecutor::discover_system_git(custom_path).await
    }

    /// Opens a repository from the filesystem, records it in history, and starts file watcher.
    pub async fn open_repository(&self, app_handle: AppHandle, directory_path: &Path) -> Result<RepositoryOverview, ApplicationError> {
        let overview = self.git_inspector.inspect_repository(directory_path).await?;

        // Persist to recent history
        let _ = self.store.record_opened_repository(&overview.path, &overview.name);

        // Update active repository in memory
        let mut active = self.current_repository.write().await;
        *active = Some(overview.clone());

        // Start real-time background file watching
        self.watcher_manager.watch_repository(app_handle, directory_path);

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
    pub async fn get_repository_status(&self, directory_path: Option<&str>) -> Result<DetailedRepositoryStatus, ApplicationError> {
        let target_path = if let Some(path_str) = directory_path {
            PathBuf::from(path_str)
        } else {
            let active = self.current_repository.read().await;
            if let Some(ref current) = *active {
                PathBuf::from(&current.path)
            } else {
                return Err(ApplicationError::InvalidRepository("No active repository open".into()));
            }
        };

        self.git_inspector.get_detailed_status(&target_path).await
    }

    /// Stage a file.
    pub async fn stage_file(&self, directory_path: Option<&str>, file_path: &str) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.stage_file(&target_path, file_path).await
    }

    /// Unstage a file.
    pub async fn unstage_file(&self, directory_path: Option<&str>, file_path: &str) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.unstage_file(&target_path, file_path).await
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
    pub async fn discard_file_changes(&self, directory_path: Option<&str>, file_path: &str, is_untracked: bool) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.discard_file_changes(&target_path, file_path, is_untracked).await
    }

    /// Stages a unified diff patch into the index.
    pub async fn stage_patch(&self, directory_path: Option<&str>, patch_content: &str) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.stage_patch(&target_path, patch_content).await
    }

    /// Unstages a unified diff patch from the index.
    pub async fn unstage_patch(&self, directory_path: Option<&str>, patch_content: &str) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.unstage_patch(&target_path, patch_content).await
    }

    /// Discards changes in a unified diff patch from the working tree.
    pub async fn discard_patch(&self, directory_path: Option<&str>, patch_content: &str) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.discard_patch(&target_path, patch_content).await
    }

    /// Commit staged changes.
    pub async fn commit(&self, directory_path: Option<&str>, message: &str, is_amend: bool) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.commit(&target_path, message, is_amend).await
    }

    /// Retrieves structured diff for a file.
    pub async fn get_file_diff(&self, directory_path: Option<&str>, file_path: &str, is_staged: bool) -> Result<crate::domain::FileDiff, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.get_file_diff(&target_path, file_path, is_staged).await
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

        let commits = self.git_inspector
            .get_commit_history(&target_path, skip_count, limit_count, branch_or_reference.as_deref())
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
        self.git_inspector.get_repository_references(&target_path).await
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
        self.git_inspector.checkout_branch(&target_path, branch_name, create_if_missing, start_point).await
    }

    /// Creates a branch without switching to it.
    pub async fn create_branch(
        &self,
        directory_path: Option<&str>,
        branch_name: &str,
        start_point: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.create_branch(&target_path, branch_name, start_point).await
    }

    /// Deletes a local branch.
    pub async fn delete_branch(
        &self,
        directory_path: Option<&str>,
        branch_name: &str,
        force: bool,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.delete_branch(&target_path, branch_name, force).await
    }

    /// Renames a local branch.
    pub async fn rename_branch(
        &self,
        directory_path: Option<&str>,
        old_name: &str,
        new_name: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.rename_branch(&target_path, old_name, new_name).await
    }

    /// Fetches changes from remote.
    pub async fn fetch_remote(
        &self,
        directory_path: Option<&str>,
        remote_name: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.fetch_remote(&target_path, remote_name).await
    }

    /// Pulls changes from upstream.
    pub async fn pull_changes(
        &self,
        directory_path: Option<&str>,
        remote_name: Option<&str>,
        branch_name: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.pull_changes(&target_path, remote_name, branch_name).await
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
        self.git_inspector.push_changes(&target_path, remote_name, branch_name, set_upstream).await
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

        let details = self.git_inspector.get_commit_details(&target_path, commit_hash).await?;

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

        let diff = self.git_inspector.get_commit_file_diff(&target_path, commit_hash, file_path).await?;

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
        self.git_inspector.compare_commits(&target_path, base_hash, target_hash).await
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
        self.git_inspector.get_comparison_file_diff(&target_path, base_hash, target_hash, file_path).await
    }

    /// Lists stash records for repository.
    pub async fn get_stash_list(&self, directory_path: Option<&str>) -> Result<Vec<StashRecord>, ApplicationError> {
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
        self.git_inspector.save_stash(&target_path, message, include_untracked).await
    }

    /// Applies a stash entry without removing it.
    pub async fn apply_stash(&self, directory_path: Option<&str>, stash_index: usize) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.apply_stash(&target_path, stash_index).await
    }

    /// Pops a stash entry, applying and dropping it.
    pub async fn pop_stash(&self, directory_path: Option<&str>, stash_index: usize) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.pop_stash(&target_path, stash_index).await
    }

    /// Drops a stash entry from the stack.
    pub async fn drop_stash(&self, directory_path: Option<&str>, stash_index: usize) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.drop_stash(&target_path, stash_index).await
    }

    /// Cherry-picks a commit onto current branch.
    pub async fn cherry_pick_commit(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.cherry_pick_commit(&target_path, commit_hash).await
    }

    /// Reverts a commit by creating an inverse commit.
    pub async fn revert_commit(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.revert_commit(&target_path, commit_hash).await
    }

    /// Resets the current branch to a specific commit.
    pub async fn reset_to_commit(
        &self,
        directory_path: Option<&str>,
        commit_hash: &str,
        reset_mode: &str,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.reset_to_commit(&target_path, commit_hash, reset_mode).await
    }

    /// Retrieves 3-way conflict stages and working file content for a conflicted file.
    pub async fn get_conflict_file_details(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
    ) -> Result<ConflictFileDetails, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.get_conflict_file_details(&target_path, file_path).await
    }

    /// Resolves a conflicted file using a preset strategy ('ours', 'theirs', 'mark_resolved').
    pub async fn resolve_conflict_file(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        resolution_strategy: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.resolve_conflict_file(&target_path, file_path, resolution_strategy).await
    }

    /// Saves custom resolved content directly to disk and marks the file as resolved.
    pub async fn write_conflict_resolved_content(
        &self,
        directory_path: Option<&str>,
        file_path: &str,
        resolved_content: &str,
    ) -> Result<(), ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.write_conflict_resolved_content(&target_path, file_path, resolved_content).await
    }

    /// Merges branch into current HEAD branch.
    pub async fn merge_branch(
        &self,
        directory_path: Option<&str>,
        branch_name: &str,
        no_fast_forward: bool,
    ) -> Result<MergeBranchOutcome, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.merge_branch(&target_path, branch_name, no_fast_forward).await
    }

    /// Rebases current HEAD branch onto upstream branch.
    pub async fn rebase_branch(
        &self,
        directory_path: Option<&str>,
        upstream_branch: &str,
    ) -> Result<RebaseBranchOutcome, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.rebase_branch(&target_path, upstream_branch).await
    }

    /// Aborts active ongoing operation (merge, rebase, cherry-pick, revert).
    pub async fn abort_repository_operation(
        &self,
        directory_path: Option<&str>,
        create_backup: Option<bool>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.abort_repository_operation(&target_path, create_backup.unwrap_or(false)).await
    }

    /// Continues active ongoing operation after all conflicts are resolved.
    pub async fn continue_repository_operation(
        &self,
        directory_path: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let target_path = self.resolve_path(directory_path).await?;
        self.git_inspector.continue_repository_operation(&target_path).await
    }

    async fn resolve_path(&self, directory_path: Option<&str>) -> Result<PathBuf, ApplicationError> {
        if let Some(p) = directory_path {
            Ok(PathBuf::from(p))
        } else {
            let active = self.current_repository.read().await;
            if let Some(ref current) = *active {
                Ok(PathBuf::from(&current.path))
            } else {
                Err(ApplicationError::InvalidRepository("No active repository open".into()))
            }
        }
    }

    /// Retrieves recent repositories list.
    pub fn get_recent_repositories(&self, limit: usize) -> Result<Vec<RecentRepositoryRecord>, ApplicationError> {
        self.store.get_recent_repositories(limit)
    }

    /// Removes a repository from the recents list.
    pub fn remove_recent_repository(&self, path: &str) -> Result<(), ApplicationError> {
        self.store.remove_recent_repository(path)
    }
}

