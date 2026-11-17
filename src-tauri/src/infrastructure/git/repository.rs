use std::path::{Path, PathBuf};
use crate::domain::{
    ApplicationError, ConflictFileDetails, RepositoryOperationState, RepositoryOverview, StashRecord,
};
use super::process::GitProcessExecutor;
use super::stash::parse_stash_list;

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
                        behind_count = behind_str.parse::<u32>().unwrap_or(0);
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

        let mut status = super::status::parse_porcelain_v2_status(&output);
        status.operation_state = self.detect_repository_operation_state(directory_path).await.unwrap_or(RepositoryOperationState::None);
        Ok(status)
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
            "%(refname:short){del}%(objectname){del}%(symref)",
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

    /// Lists all stash records for the given repository.
    pub async fn list_stashes(&self, directory_path: &Path) -> Result<Vec<StashRecord>, ApplicationError> {
        let output = self.executor
            .execute_git_command(Some(directory_path), &["stash", "list", "--format=%gd\x1f%gs\x1f%H\x1f%ct"])
            .await?;
        Ok(parse_stash_list(&output))
    }

    /// Saves the current working directory changes to a new stash entry.
    pub async fn save_stash(
        &self,
        directory_path: &Path,
        message: Option<&str>,
        include_untracked: bool,
    ) -> Result<String, ApplicationError> {
        let mut arguments = vec!["stash", "push"];
        if include_untracked {
            arguments.push("--include-untracked");
        }
        if let Some(custom_message) = message {
            let trimmed_message = custom_message.trim();
            if !trimmed_message.is_empty() {
                arguments.push("-m");
                arguments.push(trimmed_message);
            }
        }
        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }

    /// Applies a stash entry by its numeric index without removing it from the stash stack.
    pub async fn apply_stash(&self, directory_path: &Path, stash_index: usize) -> Result<String, ApplicationError> {
        let stash_identifier = format!("stash@{{{}}}", stash_index);
        self.executor
            .execute_git_command(Some(directory_path), &["stash", "apply", &stash_identifier])
            .await
    }

    /// Pops a stash entry by its numeric index, applying it and removing it from the stack.
    pub async fn pop_stash(&self, directory_path: &Path, stash_index: usize) -> Result<String, ApplicationError> {
        let stash_identifier = format!("stash@{{{}}}", stash_index);
        self.executor
            .execute_git_command(Some(directory_path), &["stash", "pop", &stash_identifier])
            .await
    }

    /// Drops a stash entry by its numeric index, removing it from the stack.
    pub async fn drop_stash(&self, directory_path: &Path, stash_index: usize) -> Result<String, ApplicationError> {
        let stash_identifier = format!("stash@{{{}}}", stash_index);
        self.executor
            .execute_git_command(Some(directory_path), &["stash", "drop", &stash_identifier])
            .await
    }

    /// Cherry-picks a commit onto the current branch.
    pub async fn cherry_pick_commit(
        &self,
        directory_path: &Path,
        commit_hash: &str,
    ) -> Result<String, ApplicationError> {
        self.executor
            .execute_git_command(Some(directory_path), &["cherry-pick", commit_hash])
            .await
    }

    /// Reverts a commit by creating an inverse commit.
    pub async fn revert_commit(
        &self,
        directory_path: &Path,
        commit_hash: &str,
    ) -> Result<String, ApplicationError> {
        self.executor
            .execute_git_command(Some(directory_path), &["revert", "--no-edit", commit_hash])
            .await
    }

    /// Resets the current branch to a specific commit using soft, mixed, or hard mode.
    pub async fn reset_to_commit(
        &self,
        directory_path: &Path,
        commit_hash: &str,
        reset_mode: &str,
    ) -> Result<String, ApplicationError> {
        let mode_flag = match reset_mode {
            "soft" => "--soft",
            "hard" => "--hard",
            _ => "--mixed",
        };
        self.executor
            .execute_git_command(Some(directory_path), &["reset", mode_flag, commit_hash])
            .await
    }

    /// Discovers the absolute path of the `.git` directory for repository or worktree.
    pub async fn get_git_directory(&self, directory_path: &Path) -> Result<PathBuf, ApplicationError> {
        let output = self.executor
            .execute_git_command(Some(directory_path), &["rev-parse", "--git-dir"])
            .await?;
        let trimmed = output.trim();
        let path = PathBuf::from(trimmed);
        if path.is_absolute() {
            Ok(path)
        } else {
            Ok(directory_path.join(path))
        }
    }

    /// Detects if Git is currently in an ongoing operation such as merge, rebase, or cherry-pick.
    pub async fn detect_repository_operation_state(
        &self,
        directory_path: &Path,
    ) -> Result<RepositoryOperationState, ApplicationError> {
        let git_dir = match self.get_git_directory(directory_path).await {
            Ok(dir) => dir,
            Err(_) => return Ok(RepositoryOperationState::None),
        };

        if git_dir.join("MERGE_HEAD").exists() {
            Ok(RepositoryOperationState::Merging)
        } else if git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists() {
            Ok(RepositoryOperationState::Rebasing)
        } else if git_dir.join("CHERRY_PICK_HEAD").exists() {
            Ok(RepositoryOperationState::CherryPicking)
        } else if git_dir.join("REVERT_HEAD").exists() {
            Ok(RepositoryOperationState::Reverting)
        } else {
            Ok(RepositoryOperationState::None)
        }
    }

    /// Retrieves base (:1:), ours (:2:), theirs (:3:), and current working file content for a conflicted file.
    pub async fn get_conflict_file_details(
        &self,
        directory_path: &Path,
        file_path: &str,
    ) -> Result<ConflictFileDetails, ApplicationError> {
        let base_output = self.executor
            .execute_git_command(Some(directory_path), &["show", &format!(":1:{}", file_path)])
            .await
            .ok();

        let ours_output = self.executor
            .execute_git_command(Some(directory_path), &["show", &format!(":2:{}", file_path)])
            .await
            .ok();

        let theirs_output = self.executor
            .execute_git_command(Some(directory_path), &["show", &format!(":3:{}", file_path)])
            .await
            .ok();

        let full_path = directory_path.join(file_path);
        let working_file_content = tokio::fs::read_to_string(&full_path)
            .await
            .unwrap_or_default();

        Ok(ConflictFileDetails {
            file_path: file_path.to_string(),
            base_content: base_output,
            ours_content: ours_output,
            theirs_content: theirs_output,
            working_file_content,
        })
    }

    /// Resolves a conflict file using predefined strategies ('ours', 'theirs', or 'mark_resolved').
    pub async fn resolve_conflict_file(
        &self,
        directory_path: &Path,
        file_path: &str,
        resolution_strategy: &str,
    ) -> Result<(), ApplicationError> {
        match resolution_strategy {
            "ours" => {
                self.executor
                    .execute_git_command(Some(directory_path), &["checkout", "--ours", "--", file_path])
                    .await?;
                self.executor
                    .execute_git_command(Some(directory_path), &["add", "--", file_path])
                    .await?;
            }
            "theirs" => {
                self.executor
                    .execute_git_command(Some(directory_path), &["checkout", "--theirs", "--", file_path])
                    .await?;
                self.executor
                    .execute_git_command(Some(directory_path), &["add", "--", file_path])
                    .await?;
            }
            "mark_resolved" => {
                self.executor
                    .execute_git_command(Some(directory_path), &["add", "--", file_path])
                    .await?;
            }
            _ => {
                return Err(ApplicationError::Unexpected(format!(
                    "Unknown conflict resolution strategy: {}",
                    resolution_strategy
                )));
            }
        }
        Ok(())
    }

    /// Writes resolved content directly to the file on disk and stages it with `git add`.
    pub async fn write_conflict_resolved_content(
        &self,
        directory_path: &Path,
        file_path: &str,
        resolved_content: &str,
    ) -> Result<(), ApplicationError> {
        let full_path = directory_path.join(file_path);
        tokio::fs::write(&full_path, resolved_content)
            .await
            .map_err(|error| ApplicationError::FilesystemError(error.to_string()))?;

        self.executor
            .execute_git_command(Some(directory_path), &["add", "--", file_path])
            .await?;
        Ok(())
    }

    /// Aborts any active ongoing operation (merge, rebase, cherry-pick, revert).
    pub async fn abort_repository_operation(
        &self,
        directory_path: &Path,
    ) -> Result<String, ApplicationError> {
        let operation_state = self.detect_repository_operation_state(directory_path).await?;
        match operation_state {
            RepositoryOperationState::Merging => {
                self.executor
                    .execute_git_command(Some(directory_path), &["merge", "--abort"])
                    .await
            }
            RepositoryOperationState::Rebasing => {
                self.executor
                    .execute_git_command(Some(directory_path), &["rebase", "--abort"])
                    .await
            }
            RepositoryOperationState::CherryPicking => {
                self.executor
                    .execute_git_command(Some(directory_path), &["cherry-pick", "--abort"])
                    .await
            }
            RepositoryOperationState::Reverting => {
                self.executor
                    .execute_git_command(Some(directory_path), &["revert", "--abort"])
                    .await
            }
            RepositoryOperationState::None => {
                Err(ApplicationError::Unexpected("No active Git operation to abort".into()))
            }
        }
    }

    /// Continues an ongoing operation after conflicts are resolved.
    pub async fn continue_repository_operation(
        &self,
        directory_path: &Path,
    ) -> Result<String, ApplicationError> {
        let operation_state = self.detect_repository_operation_state(directory_path).await?;
        match operation_state {
            RepositoryOperationState::Merging => {
                self.executor
                    .execute_git_command(Some(directory_path), &["commit", "--no-edit"])
                    .await
            }
            RepositoryOperationState::Rebasing => {
                self.executor
                    .execute_git_command(Some(directory_path), &["rebase", "--continue"])
                    .await
            }
            RepositoryOperationState::CherryPicking => {
                self.executor
                    .execute_git_command(Some(directory_path), &["cherry-pick", "--continue"])
                    .await
            }
            RepositoryOperationState::Reverting => {
                self.executor
                    .execute_git_command(Some(directory_path), &["revert", "--continue"])
                    .await
            }
            RepositoryOperationState::None => {
                Err(ApplicationError::Unexpected("No active Git operation to continue".into()))
            }
        }
    }
}
