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

    /// Fetches detailed status (staged, unstaged, conflicted files) using porcelain v2.
    pub async fn get_detailed_status(&self, directory_path: &Path) -> Result<crate::domain::DetailedRepositoryStatus, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let output = self.executor
            .execute_git_command(Some(directory_path), &["status", "--porcelain=v2", "--branch", "-z"])
            .await?;

        Ok(super::status::parse_porcelain_v2_status(&output))
    }

    /// Stages a specific file.
    pub async fn stage_file(&self, directory_path: &Path, file_path: &str) -> Result<(), ApplicationError> {
        self.executor
            .execute_git_command(Some(directory_path), &["add", "--", file_path])
            .await?;
        Ok(())
    }

    /// Unstages a specific file.
    pub async fn unstage_file(&self, directory_path: &Path, file_path: &str) -> Result<(), ApplicationError> {
        self.executor
            .execute_git_command(Some(directory_path), &["restore", "--staged", "--", file_path])
            .await?;
        Ok(())
    }

    /// Stages all changes (tracked and untracked).
    pub async fn stage_all(&self, directory_path: &Path) -> Result<(), ApplicationError> {
        self.executor
            .execute_git_command(Some(directory_path), &["add", "-A"])
            .await?;
        Ok(())
    }

    /// Unstages all staged changes.
    pub async fn unstage_all(&self, directory_path: &Path) -> Result<(), ApplicationError> {
        self.executor
            .execute_git_command(Some(directory_path), &["restore", "--staged", "."])
            .await?;
        Ok(())
    }

    /// Discards working tree changes for a specific file.
    pub async fn discard_file_changes(&self, directory_path: &Path, file_path: &str, is_untracked: bool) -> Result<(), ApplicationError> {
        if is_untracked {
            self.executor
                .execute_git_command(Some(directory_path), &["clean", "-f", "--", file_path])
                .await?;
        } else {
            self.executor
                .execute_git_command(Some(directory_path), &["restore", "--", file_path])
                .await?;
        }
        Ok(())
    }

    /// Commits staged changes with a commit message and optional amend flag.
    pub async fn commit(&self, directory_path: &Path, message: &str, is_amend: bool) -> Result<String, ApplicationError> {
        let mut arguments = vec!["commit", "-m", message];
        if is_amend {
            arguments.push("--amend");
        }

        self.executor
            .execute_git_command(Some(directory_path), &arguments)
            .await
    }

    /// Retrieves structured diff for a specific file (staged or unstaged).
    pub async fn get_file_diff(&self, directory_path: &Path, file_path: &str, is_staged: bool) -> Result<crate::domain::FileDiff, ApplicationError> {
        let raw_diff = if is_staged {
            self.executor
                .execute_git_command(Some(directory_path), &["diff", "--cached", "--", file_path])
                .await?
        } else {
            // Check if file is untracked to generate diff against /dev/null
            let untracked_diff = self.executor
                .execute_git_command(Some(directory_path), &["diff", "--", file_path])
                .await?;

            if untracked_diff.trim().is_empty() {
                // If it's untracked, use git diff --no-index /dev/null <file_path> or read file
                let no_index_result = self.executor
                    .execute_git_command(Some(directory_path), &["diff", "--no-index", "--", "/dev/null", file_path])
                    .await;
                match no_index_result {
                    Ok(out) => out,
                    Err(ApplicationError::GitExecutionFailed { exit_code: 1, message }) => {
                        // git diff returns 1 when differences exist
                        message
                    }
                    Err(e) => return Err(e),
                }
            } else {
                untracked_diff
            }
        };

        Ok(super::diff::parse_unified_diff(&raw_diff, file_path))
    }

    /// Extracts commit history using machine-readable delimited output.
    pub async fn get_commit_history(
        &self,
        directory_path: &Path,
        skip: usize,
        limit: usize,
        branch_or_reference: Option<&str>,
    ) -> Result<Vec<crate::domain::GitCommit>, ApplicationError> {
        let skip_argument = format!("--skip={}", skip);
        let limit_argument = format!("-n{}", limit);
        let format_argument = format!("--format={}", super::history::COMMIT_LOG_FORMAT);

        let mut arguments: Vec<&str> = vec![
            "log",
            "--topo-order",
            &skip_argument,
            &limit_argument,
            &format_argument,
        ];

        let reference_target = branch_or_reference.unwrap_or("--all");
        arguments.push(reference_target);

        let raw_output = match self.executor.execute_git_command(Some(directory_path), &arguments).await {
            Ok(output) => output,
            Err(ApplicationError::GitExecutionFailed { exit_code: 128, .. }) => {
                // Repository might be empty (zero commits)
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };

        Ok(super::history::parse_delimited_git_log(&raw_output))
    }

    /// Fetches full commit details including metadata, changed files, and stats.
    pub async fn get_commit_details(
        &self,
        directory_path: &Path,
        commit_hash: &str,
    ) -> Result<crate::domain::CommitDetails, ApplicationError> {
        let format_argument = format!("--format={}", super::history::COMMIT_LOG_FORMAT);
        let log_output = self.executor
            .execute_git_command(
                Some(directory_path),
                &["log", "-1", &format_argument, commit_hash],
            )
            .await?;

        let commits = super::history::parse_delimited_git_log(&log_output);
        let commit = commits.into_iter().next().ok_or_else(|| {
            ApplicationError::GitExecutionFailed {
                exit_code: 1,
                message: format!("Commit {} not found", commit_hash),
            }
        })?;

        // Extract changed files list
        let name_status_output = self.executor
            .execute_git_command(
                Some(directory_path),
                &["diff-tree", "--root", "--no-commit-id", "-r", "-m", "-M", "--name-status", commit_hash],
            )
            .await
            .unwrap_or_default();

        let numstat_output = self.executor
            .execute_git_command(
                Some(directory_path),
                &["diff-tree", "--root", "--no-commit-id", "-r", "-m", "-M", "--numstat", commit_hash],
            )
            .await
            .unwrap_or_default();

        let changed_files = super::history::parse_commit_changed_files(&name_status_output, &numstat_output);
        let total_additions = changed_files.iter().map(|file| file.additions).sum();
        let total_deletions = changed_files.iter().map(|file| file.deletions).sum();

        Ok(crate::domain::CommitDetails {
            hash: commit.hash,
            parent_hashes: commit.parent_hashes,
            author_name: commit.author_name,
            author_email: commit.author_email,
            author_timestamp: commit.author_timestamp,
            committer_name: commit.committer_name,
            committer_email: commit.committer_email,
            committer_timestamp: commit.committer_timestamp,
            subject: commit.subject,
            body: commit.body,
            decorations: commit.decorations,
            total_additions,
            total_deletions,
            changed_files,
        })
    }

    /// Fetches unified diff for a single file in a commit or the entire commit.
    pub async fn get_commit_file_diff(
        &self,
        directory_path: &Path,
        commit_hash: &str,
        file_path: Option<&str>,
    ) -> Result<crate::domain::FileDiff, ApplicationError> {
        let mut arguments = vec!["show", "-m", "--first-parent", "--format="];
        arguments.push(commit_hash);

        if let Some(target_file) = file_path {
            arguments.push("--");
            arguments.push(target_file);
        }

        let raw_diff = self.executor
            .execute_git_command(Some(directory_path), &arguments)
            .await
            .unwrap_or_default();

        let display_path = file_path.unwrap_or(commit_hash);
        Ok(super::diff::parse_unified_diff(&raw_diff, display_path))
    }

    /// Fetches all repository references: local branches, remote branches, tags, and remotes.
    pub async fn get_repository_references(&self, directory_path: &Path) -> Result<crate::domain::RepositoryReferences, ApplicationError> {
        // 1. Local branches
        let branch_format = format!(
            "%(refname:short){del}%(HEAD){del}%(upstream:short){del}%(upstream:track,nobracket){del}%(objectname)",
            del = super::branch::REF_FIELD_DELIMITER
        );
        let raw_branches = self.executor
            .execute_git_command(Some(directory_path), &["for-each-ref", &format!("--format={}", branch_format), "refs/heads/"])
            .await
            .unwrap_or_default();
        let branches = super::branch::parse_local_branches(&raw_branches);

        // 2. Remote tracking branches
        let remote_branch_format = format!(
            "%(refname:short){del}%(objectname)",
            del = super::branch::REF_FIELD_DELIMITER
        );
        let raw_remote_branches = self.executor
            .execute_git_command(Some(directory_path), &["for-each-ref", &format!("--format={}", remote_branch_format), "refs/remotes/"])
            .await
            .unwrap_or_default();
        let remote_branches = super::branch::parse_remote_branches(&raw_remote_branches);

        // 3. Tags
        let raw_tags = self.executor
            .execute_git_command(Some(directory_path), &["tag", "-l", &format!("--format=%(refname:short){}%(objectname)", super::branch::REF_FIELD_DELIMITER)])
            .await
            .unwrap_or_default();
        let tags = super::branch::parse_tags(&raw_tags);

        // 4. Remotes
        let raw_remotes = self.executor
            .execute_git_command(Some(directory_path), &["remote", "-v"])
            .await
            .unwrap_or_default();
        let remotes = super::branch::parse_remotes(&raw_remotes);

        Ok(crate::domain::RepositoryReferences {
            branches,
            remote_branches,
            tags,
            remotes,
        })
    }

    /// Switches/checkouts to a specified branch.
    pub async fn checkout_branch(
        &self,
        directory_path: &Path,
        branch_name: &str,
        create_if_missing: bool,
        start_point: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let mut arguments = vec!["checkout"];
        if create_if_missing {
            arguments.push("-b");
        }
        arguments.push(branch_name);
        if let Some(sp) = start_point {
            arguments.push(sp);
        }

        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }

    /// Creates a new branch from current HEAD or specified commit.
    pub async fn create_branch(
        &self,
        directory_path: &Path,
        branch_name: &str,
        start_point: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let mut arguments = vec!["branch", branch_name];
        if let Some(sp) = start_point {
            arguments.push(sp);
        }

        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }

    /// Deletes a local branch (safe by default with -d, or forced with -D).
    pub async fn delete_branch(
        &self,
        directory_path: &Path,
        branch_name: &str,
        force: bool,
    ) -> Result<String, ApplicationError> {
        let flag = if force { "-D" } else { "-d" };
        self.executor
            .execute_git_command(Some(directory_path), &["branch", flag, branch_name])
            .await
    }

    /// Executes `git fetch`.
    pub async fn fetch_remote(
        &self,
        directory_path: &Path,
        remote_name: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let mut arguments = vec!["fetch", "--prune"];
        if let Some(name) = remote_name {
            arguments.push(name);
        }
        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }

    /// Executes `git pull`.
    pub async fn pull_changes(
        &self,
        directory_path: &Path,
        remote_name: Option<&str>,
        branch_name: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let mut arguments = vec!["pull"];
        if let Some(remote) = remote_name {
            arguments.push(remote);
            if let Some(branch) = branch_name {
                arguments.push(branch);
            }
        }
        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }

    /// Executes `git push`.
    pub async fn push_changes(
        &self,
        directory_path: &Path,
        remote_name: Option<&str>,
        branch_name: Option<&str>,
        set_upstream: bool,
    ) -> Result<String, ApplicationError> {
        let mut arguments = vec!["push"];
        if set_upstream {
            arguments.push("-u");
        }
        if let Some(remote) = remote_name {
            arguments.push(remote);
            if let Some(branch) = branch_name {
                arguments.push(branch);
            }
        }
        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }
}
