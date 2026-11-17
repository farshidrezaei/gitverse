use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;
use crate::domain::{
    ApplicationError, DetailedRepositoryStatus, GitBinaryInformation, RecentRepositoryRecord, RepositoryOverview,
};
use crate::infrastructure::git::{GitProcessExecutor, GitRepositoryInspector, RepositoryWatcherManager};
use crate::infrastructure::storage::SQLiteRepositoryStore;

/// Global application state shared across Tauri commands.
pub struct ApplicationState {
    pub current_repository: Arc<RwLock<Option<RepositoryOverview>>>,
    pub store: Arc<SQLiteRepositoryStore>,
    pub git_executor: Arc<GitProcessExecutor>,
    pub git_inspector: Arc<GitRepositoryInspector>,
    pub watcher_manager: Arc<RepositoryWatcherManager>,
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

