pub mod process;
pub mod repository;
pub mod status;
pub mod watcher;

#[cfg(test)]
mod status_test;

pub use process::GitProcessExecutor;
pub use repository::GitRepositoryInspector;
pub use status::parse_porcelain_v2_status;
pub use watcher::RepositoryWatcherManager;


