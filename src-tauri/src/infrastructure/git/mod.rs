pub mod diff;
pub mod process;
pub mod repository;
pub mod status;
pub mod watcher;

#[cfg(test)]
mod status_test;
#[cfg(test)]
mod diff_test;

pub use diff::parse_unified_diff;
pub use process::GitProcessExecutor;
pub use repository::GitRepositoryInspector;
pub use status::parse_porcelain_v2_status;
pub use watcher::RepositoryWatcherManager;



