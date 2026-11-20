pub mod bisect;
pub mod branch;
pub mod diff;
pub mod history;
pub mod process;
pub mod repository;
pub mod stash;
pub mod status;
pub mod submodule;
pub mod watcher;
pub mod worktree;

#[cfg(test)]
mod bisect_test;
#[cfg(test)]
mod branch_test;
#[cfg(test)]
mod diff_test;
#[cfg(test)]
mod history_test;
#[cfg(test)]
mod stash_test;
#[cfg(test)]
mod status_test;
#[cfg(test)]
mod submodule_test;
#[cfg(test)]
mod worktree_test;

pub use bisect::parse_bisect_step_output;
pub use branch::{parse_local_branches, parse_remote_branches, parse_remotes, parse_tags, validate_branch_name};
pub use diff::parse_unified_diff;
pub use history::parse_delimited_git_log;
pub use process::GitProcessExecutor;
pub use repository::GitRepositoryInspector;
pub use stash::parse_stash_list;
pub use status::parse_porcelain_v2_status;
pub use submodule::parse_submodule_status;
pub use watcher::RepositoryWatcherManager;
pub use worktree::parse_worktree_list;



