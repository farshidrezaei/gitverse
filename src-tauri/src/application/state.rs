use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::{
    ApplicationError, GitBinaryInformation, RecentRepositoryRecord, RepositoryOverview,
};
use crate::infrastructure::git::{GitProcessExecutor, GitRepositoryInspector};
use crate::infrastructure::storage::SQLiteRepositoryStore;

/// Global application state shared across Tauri commands.
pub struct ApplicationState {
    pub current_repository: Arc<RwLock<Option<RepositoryOverview>>>,
    pub store: Arc<SQLiteRepositoryStore>,
    pub git_executor: Arc<GitProcessExecutor>,
    pub git_inspector: Arc<GitRepositoryInspector>,
}

impl ApplicationState {
    pub fn new(database_path: &Path) -> Result<Self, ApplicationError> {
        let store = Arc::new(SQLiteRepositoryStore::new(database_path)?);
        let git_executor = Arc::new(GitProcessExecutor::new(PathBuf::from("git")));
        let git_inspector = Arc::new(GitRepositoryInspector::new((*git_executor).clone()));

        Ok(Self {
            current_repository: Arc::new(RwLock::new(None)),
            store,
            git_executor,
            git_inspector,
        })
    }

    /// Discovers system Git and returns binary metadata.
    pub async fn discover_git(&self, custom_path: Option<&str>) -> Result<GitBinaryInformation, ApplicationError> {
        GitProcessExecutor::discover_system_git(custom_path).await
    }

    /// Opens a repository from the filesystem, records it in history, and stores it as current.
    pub async fn open_repository(&self, directory_path: &Path) -> Result<RepositoryOverview, ApplicationError> {
        let overview = self.git_inspector.inspect_repository(directory_path).await?;

        // Persist to recent history
        let _ = self.store.record_opened_repository(&overview.path, &overview.name);

        // Update active repository in memory
        let mut active = self.current_repository.write().await;
        *active = Some(overview.clone());

        Ok(overview)
    }

    /// Retrieves current active repository overview.
    pub async fn get_current_repository(&self) -> Option<RepositoryOverview> {
        let active = self.current_repository.read().await;
        active.clone()
    }

    /// Closes current active repository.
    pub async fn close_current_repository(&self) {
        let mut active = self.current_repository.write().await;
        *active = None;
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
