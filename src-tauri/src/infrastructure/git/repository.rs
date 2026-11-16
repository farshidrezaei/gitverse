use std::path::Path;
use crate::domain::{ApplicationError, RepositoryOverview};
use super::process::GitProcessExecutor;

/// Service responsible for querying Git repository state, references, and working tree status.
#[derive(Debug, Clone)]
pub struct GitRepositoryInspector {
    executor: GitProcessExecutor,
}

impl GitRepositoryInspector {
    pub fn new(executor: GitProcessExecutor) -> Self {
        Self { executor }
    }

    /// Verifies if a given path is a valid Git repository root or worktree.
    pub async fn verify_repository_path(&self, directory_path: &Path) -> Result<bool, ApplicationError> {
        let output = self.executor
            .execute_git_command(Some(directory_path), &["rev-parse", "--is-inside-work-tree"])
            .await;

        match output {
            Ok(result) => Ok(result.trim() == "true"),
            Err(_) => Ok(false),
        }
    }

    /// Reads full overview metadata for a given repository path.
    pub async fn inspect_repository(&self, directory_path: &Path) -> Result<RepositoryOverview, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let folder_name = directory_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("repository")
            .to_string();

        // Check if repository has any commits
        let head_commit_result = self.executor
            .execute_git_command(Some(directory_path), &["rev-parse", "HEAD"])
            .await;

        let (is_empty, head_commit_hash) = match head_commit_result {
            Ok(hash) => (false, Some(hash.trim().to_string())),
            Err(_) => (true, None),
        };

        // Determine current branch name or detached HEAD
        let branch_result = self.executor
            .execute_git_command(Some(directory_path), &["branch", "--show-current"])
            .await;

        let (is_detached_head, current_branch_name) = match branch_result {
            Ok(branch_output) => {
                let trimmed = branch_output.trim();
                if trimmed.is_empty() {
                    (true, None)
                } else {
                    (false, Some(trimmed.to_string()))
                }
            }
            Err(_) => (false, None),
        };

        // Check working tree status using machine-readable porcelain v2
        let status_result = self.executor
            .execute_git_command(Some(directory_path), &["status", "--porcelain=v2", "--branch"])
            .await?;

        let mut ahead_count: u32 = 0;
        let mut behind_count: u32 = 0;
        let mut working_tree_is_clean = true;

        for line in status_result.lines() {
            if let Some(rest) = line.strip_prefix("# branch.ab +") {
                // Format: +0 -0
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() == 2 {
                    if let Ok(ahead) = parts[0].parse::<u32>() {
                        ahead_count = ahead;
                    }
                    if let Some(behind_str) = parts[1].strip_prefix('-') {
                        if let Ok(behind) = behind_str.parse::<u32>() {
                            behind_count = behind;
                        }
                    }
                }
            } else if !line.starts_with('#') {
                // Any line not starting with # indicates modified, untracked, or staged changes
                working_tree_is_clean = false;
            }
        }

        Ok(RepositoryOverview {
            path: directory_path.to_string_lossy().to_string(),
            name: folder_name,
            current_branch_name,
            head_commit_hash,
            is_empty,
            is_detached_head,
            working_tree_is_clean,
            ahead_count,
            behind_count,
        })
    }
}
