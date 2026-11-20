use std::path::{Path, PathBuf};
use base64::Engine;
use crate::domain::{
    ApplicationError, BisectState, ConflictFileDetails, DirectoryListing, FolderEntry, GitSubmoduleSummary, GitWorktreeSummary, InteractiveRebaseRequest,
    MergeBranchOutcome, PatchCheckResult, RebaseBranchOutcome, RebasePlanItem, RepositoryOperationState, RepositoryOverview, StashRecord,
};
use crate::domain::validation::{
    validate_branch_name, validate_commit_message, validate_remote_name, validate_remote_url,
    validate_stash_message, validate_worktree_path,
};
use super::bisect::parse_bisect_step_output;
use super::process::GitProcessExecutor;
use super::stash::parse_stash_list;
use super::submodule::parse_submodule_status;
use super::worktree::parse_worktree_list;

/// Service responsible for querying Git repository state, references, and working tree status.
#[derive(Debug, Clone)]
pub struct GitRepositoryInspector {
    executor: GitProcessExecutor,
}

fn is_media_file(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".svg")
        || lower.ends_with(".webp")
        || lower.ends_with(".bmp")
        || lower.ends_with(".ico")
        || lower.ends_with(".mp3")
        || lower.ends_with(".wav")
        || lower.ends_with(".ogg")
        || lower.ends_with(".mp4")
        || lower.ends_with(".webm")
}

impl GitRepositoryInspector {
    pub fn new(executor: GitProcessExecutor) -> Self {
        Self { executor }
    }

    /// Verifies if a given path is a valid Git repository root or worktree.
    pub async fn verify_repository_path(&self, directory_path: &Path) -> Result<bool, ApplicationError> {
        let output = self.executor
            .execute_git_command(Some(directory_path), &["rev-parse", "--show-toplevel"])
            .await;

        match output {
            Ok(result) => {
                let toplevel_str = result.trim();
                let canonical_target = std::fs::canonicalize(directory_path).unwrap_or_else(|_| directory_path.to_path_buf());
                let canonical_toplevel = std::fs::canonicalize(Path::new(toplevel_str)).unwrap_or_else(|_| PathBuf::from(toplevel_str));
                Ok(canonical_target == canonical_toplevel)
            }
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

    /// Applies a unified diff patch to the Git staging index (--cached).
    pub async fn stage_patch(&self, directory_path: &Path, patch_content: &str) -> Result<(), ApplicationError> {
        self.executor
            .execute_git_command_with_stdin(
                Some(directory_path),
                &["apply", "--cached", "--unidiff-zero", "--recount", "--whitespace=nowarn", "-"],
                patch_content.as_bytes(),
            )
            .await?;
        Ok(())
    }

    /// Reverses a unified diff patch from the Git staging index (--cached --reverse).
    pub async fn unstage_patch(&self, directory_path: &Path, patch_content: &str) -> Result<(), ApplicationError> {
        self.executor
            .execute_git_command_with_stdin(
                Some(directory_path),
                &["apply", "--cached", "--reverse", "--unidiff-zero", "--recount", "--whitespace=nowarn", "-"],
                patch_content.as_bytes(),
            )
            .await?;
        Ok(())
    }

    /// Discards working tree changes represented by a unified diff patch (--reverse).
    pub async fn discard_patch(&self, directory_path: &Path, patch_content: &str) -> Result<(), ApplicationError> {
        self.executor
            .execute_git_command_with_stdin(
                Some(directory_path),
                &["apply", "--reverse", "--unidiff-zero", "--recount", "--whitespace=nowarn", "-"],
                patch_content.as_bytes(),
            )
            .await?;
        Ok(())
    }

    /// Commits staged changes with a commit message, optional amend, and optional hook bypass.
    pub async fn commit(
        &self,
        directory_path: &Path,
        message: &str,
        is_amend: bool,
        bypass_hooks: bool,
    ) -> Result<String, ApplicationError> {
        validate_commit_message(message)?;
        let mut arguments = vec!["commit", "-m", message];
        if is_amend {
            arguments.push("--amend");
        }
        if bypass_hooks {
            arguments.push("--no-verify");
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

        let mut parsed_diff = super::diff::parse_unified_diff(&raw_diff, file_path);

        // If file is binary or media, load base64 content for visual side-by-side comparison
        if parsed_diff.is_binary || is_media_file(file_path) {
            let (old_b64, new_b64) = self.load_diff_media_contents(directory_path, file_path, is_staged).await;
            parsed_diff.old_content_base64 = old_b64;
            parsed_diff.new_content_base64 = new_b64;
        }

        Ok(parsed_diff)
    }

    async fn load_diff_media_contents(
        &self,
        directory_path: &Path,
        file_path: &str,
        is_staged: bool,
    ) -> (Option<String>, Option<String>) {
        let engine = base64::engine::general_purpose::STANDARD;

        // Old content: from HEAD (or index if unstaged)
        let head_target = if is_staged {
            format!("HEAD:{}", file_path)
        } else {
            format!(":{}", file_path)
        };

        let old_content = self.executor
            .execute_git_command_raw(Some(directory_path), &["show", &head_target])
            .await
            .ok()
            .map(|bytes| engine.encode(&bytes));

        // New content: from file on disk or staging index
        let new_content = if is_staged {
            self.executor
                .execute_git_command_raw(Some(directory_path), &[":", file_path])
                .await
                .ok()
                .map(|bytes| engine.encode(&bytes))
        } else {
            let full_path = directory_path.join(file_path);
            std::fs::read(&full_path)
                .ok()
                .map(|bytes| engine.encode(&bytes))
        };

        (old_content, new_content)
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

        // Query cryptographic signature verification status (%G? = code, %GS = signer, %GK = key id)
        let signature_output = self
            .executor
            .execute_git_command(
                Some(directory_path),
                &["log", "-1", "--format=%G?%x1f%GS%x1f%GK", commit_hash],
            )
            .await
            .unwrap_or_default();
        let signature = super::history::parse_commit_signature(&signature_output);

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
            signature,
        })
    }

    /// Reads configured Git commit template (from git config commit.template or .gitmessage).
    pub async fn get_commit_template(
        &self,
        directory_path: &Path,
    ) -> Result<Option<String>, ApplicationError> {
        // 1. Check git config commit.template
        if let Ok(config_path) = self
            .executor
            .execute_git_command(Some(directory_path), &["config", "--get", "commit.template"])
            .await
        {
            let trimmed_path = config_path.trim();
            if !trimmed_path.is_empty() {
                let candidate_path = Path::new(trimmed_path);
                let full_path = if candidate_path.is_absolute() {
                    candidate_path.to_path_buf()
                } else {
                    directory_path.join(candidate_path)
                };

                if let Ok(content) = tokio::fs::read_to_string(&full_path).await {
                    return Ok(Some(content));
                }
            }
        }

        // 2. Check fallback .gitmessage in repository root
        let default_gitmessage_path = directory_path.join(".gitmessage");
        if default_gitmessage_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&default_gitmessage_path).await {
                return Ok(Some(content));
            }
        }

        Ok(None)
    }

    /// Extracts repository reflog records with optional reference target and limit.
    pub async fn get_reflog(
        &self,
        directory_path: &Path,
        reference: Option<&str>,
        limit: usize,
    ) -> Result<Vec<crate::domain::ReflogRecord>, ApplicationError> {
        let target_reference = reference.unwrap_or("HEAD");
        let limit_argument = format!("-n{}", limit.clamp(1, 500));
        let format_argument = "--format=%gd%x1f%h%x1f%H%x1f%gs%x1f%ct%x1f%an%x1e";

        let raw_output = self
            .executor
            .execute_git_command(
                Some(directory_path),
                &["reflog", "show", &limit_argument, format_argument, target_reference],
            )
            .await?;

        Ok(super::history::parse_reflog_output(&raw_output))
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
        let mut parsed_diff = super::diff::parse_unified_diff(&raw_diff, display_path);

        if let Some(target_file) = file_path {
            if parsed_diff.is_binary || is_media_file(target_file) {
                let engine = base64::engine::general_purpose::STANDARD;
                let old_target = format!("{}:{}", commit_hash, target_file);
                let old_content = self.executor
                    .execute_git_command_raw(Some(directory_path), &["show", &old_target])
                    .await
                    .ok()
                    .map(|bytes| engine.encode(&bytes));
                parsed_diff.new_content_base64 = old_content.clone(); // In a commit diff, new is the commit version
                parsed_diff.old_content_base64 = old_content;
            }
        }

        Ok(parsed_diff)
    }

    /// Compares two commits or references, returning the list of changed files and summary metrics.
    pub async fn compare_commits(
        &self,
        directory_path: &Path,
        base_hash: &str,
        target_hash: &str,
    ) -> Result<crate::domain::CommitComparison, ApplicationError> {
        let base_subject = self
            .executor
            .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", base_hash])
            .await
            .ok()
            .map(|subject| subject.trim().to_string());

        let target_subject = self
            .executor
            .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", target_hash])
            .await
            .ok()
            .map(|subject| subject.trim().to_string());

        let name_status_output = self
            .executor
            .execute_git_command(
                Some(directory_path),
                &["diff", "-M", "--name-status", base_hash, target_hash],
            )
            .await
            .unwrap_or_default();

        let numstat_output = self
            .executor
            .execute_git_command(
                Some(directory_path),
                &["diff", "-M", "--numstat", base_hash, target_hash],
            )
            .await
            .unwrap_or_default();

        let changed_files = super::history::parse_commit_changed_files(&name_status_output, &numstat_output);
        let total_additions = changed_files.iter().map(|file| file.additions).sum();
        let total_deletions = changed_files.iter().map(|file| file.deletions).sum();

        Ok(crate::domain::CommitComparison {
            base_hash: base_hash.to_string(),
            target_hash: target_hash.to_string(),
            base_subject,
            target_subject,
            total_additions,
            total_deletions,
            changed_files,
        })
    }

    /// Fetches unified diff for a single file or entire comparison between two commits.
    pub async fn get_comparison_file_diff(
        &self,
        directory_path: &Path,
        base_hash: &str,
        target_hash: &str,
        file_path: Option<&str>,
    ) -> Result<crate::domain::FileDiff, ApplicationError> {
        let mut arguments = vec!["diff", "-M", base_hash, target_hash];

        if let Some(target_file) = file_path {
            arguments.push("--");
            arguments.push(target_file);
        }

        let raw_diff = self
            .executor
            .execute_git_command(Some(directory_path), &arguments)
            .await
            .unwrap_or_default();

        let display_path = file_path.unwrap_or(target_hash);
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
        validate_branch_name(branch_name)?;

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

    /// Renames a local branch from old_name to new_name.
    pub async fn rename_branch(
        &self,
        directory_path: &Path,
        old_name: &str,
        new_name: &str,
    ) -> Result<String, ApplicationError> {
        validate_branch_name(new_name)?;

        self.executor
            .execute_git_command(Some(directory_path), &["branch", "-m", old_name, new_name])
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

    /// Adds a new Git remote to the repository.
    pub async fn add_remote(
        &self,
        directory_path: &Path,
        name: &str,
        url: &str,
    ) -> Result<String, ApplicationError> {
        let trimmed_name = name.trim();
        let trimmed_url = url.trim();
        validate_remote_name(trimmed_name)?;
        validate_remote_url(trimmed_url)?;
        self.executor
            .execute_git_command(Some(directory_path), &["remote", "add", trimmed_name, trimmed_url])
            .await
    }

    /// Removes an existing Git remote from the repository.
    pub async fn remove_remote(
        &self,
        directory_path: &Path,
        name: &str,
    ) -> Result<String, ApplicationError> {
        let trimmed_name = name.trim();
        validate_remote_name(trimmed_name)?;
        self.executor
            .execute_git_command(Some(directory_path), &["remote", "remove", trimmed_name])
            .await
    }

    /// Renames a Git remote.
    pub async fn rename_remote(
        &self,
        directory_path: &Path,
        old_name: &str,
        new_name: &str,
    ) -> Result<String, ApplicationError> {
        let trimmed_old = old_name.trim();
        let trimmed_new = new_name.trim();
        validate_remote_name(trimmed_old)?;
        validate_remote_name(trimmed_new)?;
        self.executor
            .execute_git_command(Some(directory_path), &["remote", "rename", trimmed_old, trimmed_new])
            .await
    }

    /// Updates the URL of an existing Git remote.
    pub async fn set_remote_url(
        &self,
        directory_path: &Path,
        name: &str,
        url: &str,
        is_push_only: bool,
    ) -> Result<String, ApplicationError> {
        let trimmed_name = name.trim();
        let trimmed_url = url.trim();
        validate_remote_name(trimmed_name)?;
        validate_remote_url(trimmed_url)?;
        let arguments = if is_push_only {
            vec!["remote", "set-url", "--push", trimmed_name, trimmed_url]
        } else {
            vec!["remote", "set-url", trimmed_name, trimmed_url]
        };
        self.executor
            .execute_git_command(Some(directory_path), &arguments)
            .await
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
        if let Some(msg) = message {
            validate_stash_message(msg)?;
        }
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

    /// Extracts detailed information and changed files for a specific stash entry.
    pub async fn get_stash_details(
        &self,
        directory_path: &Path,
        stash_index: usize,
    ) -> Result<crate::domain::StashDetails, ApplicationError> {
        let stash_identifier = format!("stash@{{{}}}", stash_index);
        let parent_identifier = format!("stash@{{{}}}^1", stash_index);

        let log_output = self.executor
            .execute_git_command(Some(directory_path), &["log", "-1", "--format=%H%x1f%s%x1f%ct", &stash_identifier])
            .await?;
        let parts: Vec<&str> = log_output.trim().split('\x1f').collect();
        let (commit_hash, message, timestamp) = if parts.len() >= 3 {
            (parts[0].to_string(), parts[1].to_string(), parts[2].parse::<i64>().unwrap_or(0))
        } else {
            (String::new(), format!("Stash {}", stash_index), 0)
        };

        let name_status_output = self.executor
            .execute_git_command(Some(directory_path), &["diff", "--name-status", &parent_identifier, &stash_identifier])
            .await
            .unwrap_or_default();

        let numstat_output = self.executor
            .execute_git_command(Some(directory_path), &["diff", "--numstat", &parent_identifier, &stash_identifier])
            .await
            .unwrap_or_default();

        let changed_files = super::history::parse_commit_changed_files(&name_status_output, &numstat_output);
        let total_additions = changed_files.iter().map(|file| file.additions).sum();
        let total_deletions = changed_files.iter().map(|file| file.deletions).sum();

        Ok(crate::domain::StashDetails {
            index: stash_index,
            stash_identifier,
            message,
            commit_hash,
            timestamp_seconds: timestamp,
            total_additions,
            total_deletions,
            changed_files,
        })
    }

    /// Extracts unified diff for a single file within a stash entry.
    pub async fn get_stash_file_diff(
        &self,
        directory_path: &Path,
        stash_index: usize,
        file_path: &str,
    ) -> Result<crate::domain::FileDiff, ApplicationError> {
        let stash_identifier = format!("stash@{{{}}}", stash_index);
        let parent_identifier = format!("stash@{{{}}}^1", stash_index);
        let raw_diff = self.executor
            .execute_git_command(Some(directory_path), &["diff", "-p", &parent_identifier, &stash_identifier, "--", file_path])
            .await?;

        Ok(super::diff::parse_unified_diff(&raw_diff, file_path))
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
    /// Merges the specified branch into the current HEAD branch.
    pub async fn merge_branch(
        &self,
        directory_path: &Path,
        branch_name: &str,
        no_fast_forward: bool,
    ) -> Result<MergeBranchOutcome, ApplicationError> {
        let mut arguments = vec!["merge"];
        if no_fast_forward {
            arguments.push("--no-ff");
        }
        arguments.push(branch_name);

        let (exit_code, stdout, stderr) = self
            .executor
            .execute_git_command_with_status(Some(directory_path), &arguments)
            .await?;

        if exit_code == 0 {
            Ok(MergeBranchOutcome {
                is_successful: true,
                has_conflicts: false,
                message: stdout.trim().to_string(),
            })
        } else {
            let operation_state = self.detect_repository_operation_state(directory_path).await?;
            let has_conflicts = operation_state == RepositoryOperationState::Merging
                || stdout.contains("CONFLICT")
                || stderr.contains("CONFLICT");

            if has_conflicts {
                Ok(MergeBranchOutcome {
                    is_successful: false,
                    has_conflicts: true,
                    message: format!("{}\n{}", stdout.trim(), stderr.trim()).trim().to_string(),
                })
            } else {
                let error_message = if !stderr.trim().is_empty() {
                    stderr.trim().to_string()
                } else {
                    stdout.trim().to_string()
                };
                Err(ApplicationError::GitExecutionFailed {
                    exit_code,
                    message: error_message,
                })
            }
        }
    }

    /// Rebases current HEAD branch onto the specified upstream branch.
    pub async fn rebase_branch(
        &self,
        directory_path: &Path,
        upstream_branch: &str,
    ) -> Result<RebaseBranchOutcome, ApplicationError> {
        let (exit_code, stdout, stderr) = self
            .executor
            .execute_git_command_with_status(Some(directory_path), &["rebase", upstream_branch])
            .await?;

        if exit_code == 0 {
            Ok(RebaseBranchOutcome {
                is_successful: true,
                has_conflicts: false,
                message: stdout.trim().to_string(),
            })
        } else {
            let operation_state = self.detect_repository_operation_state(directory_path).await?;
            let has_conflicts = operation_state == RepositoryOperationState::Rebasing
                || stdout.contains("CONFLICT")
                || stderr.contains("CONFLICT");

            if has_conflicts {
                Ok(RebaseBranchOutcome {
                    is_successful: false,
                    has_conflicts: true,
                    message: format!("{}\n{}", stdout.trim(), stderr.trim()).trim().to_string(),
                })
            } else {
                let error_message = if !stderr.trim().is_empty() {
                    stderr.trim().to_string()
                } else {
                    stdout.trim().to_string()
                };
                Err(ApplicationError::GitExecutionFailed {
                    exit_code,
                    message: error_message,
                })
            }
        }
    }

    /// Aborts an active ongoing operation (merge, rebase, cherry-pick, revert).
    /// If create_backup is true, creates a timestamped backup directory in `.git/gitverse_backups/`
    /// containing all currently conflicted or modified files before executing the abort.
    pub async fn abort_repository_operation(
        &self,
        directory_path: &Path,
        create_backup: bool,
    ) -> Result<String, ApplicationError> {
        if create_backup {
            let _ = self.create_pre_abort_backup(directory_path).await;
        }

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

    /// Creates a timestamped backup of conflicted/modified files to preserve manual work before aborting.
    async fn create_pre_abort_backup(&self, directory_path: &Path) -> Result<PathBuf, ApplicationError> {
        let git_dir = self.get_git_directory(directory_path).await?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        let backup_directory = git_dir.join("gitverse_backups").join(format!("backup_{}", timestamp));
        let _ = tokio::fs::create_dir_all(&backup_directory).await;

        let status = self.get_detailed_status(directory_path).await?;
        for file in status.conflicted_files.iter().chain(status.unstaged_files.iter()) {
            let source = directory_path.join(&file.path);
            let destination = backup_directory.join(&file.path);
            if let Some(parent) = destination.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            let _ = tokio::fs::copy(&source, &destination).await;
        }

        Ok(backup_directory)
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

    /// Lists only directory folders within a specified path and checks if each is a Git repository.
    pub async fn list_directory_folders(&self, directory_path: &Path) -> Result<DirectoryListing, ApplicationError> {
        let canonical_path = match std::fs::canonicalize(directory_path) {
            Ok(p) => p,
            Err(_) => directory_path.to_path_buf(),
        };

        let current_path_str = canonical_path.to_string_lossy().to_string();
        let parent_path = canonical_path.parent().map(|p| p.to_string_lossy().to_string());
        let is_current_git_repository = self.verify_repository_path(&canonical_path).await.unwrap_or(false);

        let mut folders = Vec::new();

        let read_dir_result = std::fs::read_dir(&canonical_path).map_err(|error| {
            ApplicationError::FilesystemError(format!("Failed to read directory {}: {}", current_path_str, error))
        })?;

        for entry_result in read_dir_result {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                let folder_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden folders unless it's a specific visible query
                if folder_name.starts_with('.') {
                    continue;
                }

                let subfolder_path = entry.path();
                let has_git_folder = subfolder_path.join(".git").exists();
                let is_git_repo = if has_git_folder {
                    true
                } else {
                    self.verify_repository_path(&subfolder_path).await.unwrap_or(false)
                };

                folders.push(FolderEntry {
                    name: folder_name,
                    path: subfolder_path.to_string_lossy().to_string(),
                    is_git_repository: is_git_repo,
                });
            }
        }

        // Sort folders alphabetically (case-insensitive)
        folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(DirectoryListing {
            current_path: current_path_str,
            parent_path,
            folders,
            is_current_git_repository,
        })
    }

    /// Lists all worktrees linked to the repository.
    pub async fn list_worktrees(&self, directory_path: &Path) -> Result<Vec<GitWorktreeSummary>, ApplicationError> {
        let output = self.executor
            .execute_git_command(Some(directory_path), &["worktree", "list", "--porcelain"])
            .await?;
        Ok(parse_worktree_list(&output))
    }

    /// Adds a new worktree to the repository.
    pub async fn add_worktree(
        &self,
        directory_path: &Path,
        worktree_path: &str,
        branch_name: Option<&str>,
        is_new_branch: bool,
    ) -> Result<String, ApplicationError> {
        validate_worktree_path(worktree_path)?;
        if is_new_branch {
            if let Some(branch) = branch_name {
                validate_branch_name(branch)?;
            }
        }
        let mut arguments = vec!["worktree", "add"];
        if is_new_branch {
            if let Some(branch) = branch_name {
                arguments.push("-b");
                arguments.push(branch);
            }
        }
        arguments.push(worktree_path);
        if !is_new_branch {
            if let Some(branch) = branch_name {
                arguments.push(branch);
            }
        }

        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }

    /// Removes a worktree from the repository.
    pub async fn remove_worktree(
        &self,
        directory_path: &Path,
        worktree_path: &str,
        force: bool,
    ) -> Result<String, ApplicationError> {
        let mut arguments = vec!["worktree", "remove"];
        if force {
            arguments.push("--force");
        }
        arguments.push(worktree_path);

        self.executor.execute_git_command(Some(directory_path), &arguments).await
    }

    /// Fetches all commits between base commit and HEAD for interactive rebase planning.
    pub async fn get_rebase_commits(
        &self,
        directory_path: &Path,
        base_commit_hash: &str,
    ) -> Result<Vec<RebasePlanItem>, ApplicationError> {
        let range = format!("{}..HEAD", base_commit_hash);
        let output = self
            .executor
            .execute_git_command(
                Some(directory_path),
                &["log", &range, "--reverse", "--format=%H\x1f%h\x1f%s\x1f%an"],
            )
            .await?;

        let mut plan_items = Vec::new();
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let fields: Vec<&str> = trimmed.split('\x1f').collect();
            if fields.len() >= 4 {
                plan_items.push(RebasePlanItem {
                    action: "pick".to_string(),
                    commit_hash: fields[0].to_string(),
                    short_hash: fields[1].to_string(),
                    original_subject: fields[2].to_string(),
                    new_message: None,
                    author_name: fields[3].to_string(),
                });
            }
        }

        Ok(plan_items)
    }

    /// Executes an interactive rebase with the specified ordered plan items.
    pub async fn execute_interactive_rebase(
        &self,
        directory_path: &Path,
        request: &InteractiveRebaseRequest,
    ) -> Result<RebaseBranchOutcome, ApplicationError> {
        let mut todo_content = String::new();
        for item in &request.plan_items {
            let valid_action = match item.action.as_str() {
                "reword" | "edit" | "squash" | "fixup" | "drop" => item.action.as_str(),
                _ => "pick",
            };
            todo_content.push_str(&format!("{} {} {}\n", valid_action, item.commit_hash, item.original_subject));
        }

        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_todo_path = temp_dir.join(format!("gitverse-rebase-todo-{}.txt", timestamp));

        tokio::fs::write(&temp_todo_path, &todo_content)
            .await
            .map_err(|error| ApplicationError::FilesystemError(format!("Failed to write rebase plan file: {}", error)))?;

        let current_exe = std::env::current_exe().map_err(|error| {
            ApplicationError::Unexpected(format!("Failed to determine current application path: {}", error))
        })?;

        let editor_argument = format!(
            "sequence.editor=\"{}\" --sequence-editor \"{}\"",
            current_exe.to_string_lossy(),
            temp_todo_path.to_string_lossy()
        );

        let (exit_code, stdout, stderr) = self
            .executor
            .execute_git_command_with_status(
                Some(directory_path),
                &["-c", &editor_argument, "rebase", "-i", &request.base_commit_hash],
            )
            .await?;

        let _ = tokio::fs::remove_file(&temp_todo_path).await;

        if exit_code == 0 {
            Ok(RebaseBranchOutcome {
                is_successful: true,
                has_conflicts: false,
                message: stdout.trim().to_string(),
            })
        } else {
            let operation_state = self.detect_repository_operation_state(directory_path).await?;
            let has_conflicts = operation_state == RepositoryOperationState::Rebasing
                || stdout.contains("CONFLICT")
                || stderr.contains("CONFLICT");

            if has_conflicts {
                Ok(RebaseBranchOutcome {
                    is_successful: false,
                    has_conflicts: true,
                    message: "Interactive rebase paused due to merge conflicts. Resolve conflicts and continue.".to_string(),
                })
            } else {
                Err(ApplicationError::GitExecutionFailed {
                    exit_code,
                    message: if stderr.is_empty() { stdout } else { stderr },
                })
            }
        }
    }

    /// Extracts line-by-line Git blame history for a file at an optional revision.
    pub async fn get_file_blame(
        &self,
        directory_path: &Path,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<crate::domain::FileBlame, ApplicationError> {
        let mut arguments = vec!["blame", "--porcelain"];
        if let Some(rev) = revision {
            if !rev.trim().is_empty() {
                arguments.push(rev);
            }
        }
        arguments.push("--");
        arguments.push(file_path);

        let output = self.executor.execute_git_command(Some(directory_path), &arguments).await?;

        let mut lines = Vec::new();
        let mut commit_metadata: std::collections::HashMap<String, (String, String, i64, String)> = std::collections::HashMap::new();

        let mut current_hash = String::new();
        let mut current_orig_line: u32 = 0;
        let mut current_final_line: u32 = 0;

        let mut temp_author = String::new();
        let mut temp_email = String::new();
        let mut temp_time: i64 = 0;
        let mut temp_summary = String::new();

        for line in output.lines() {
            if let Some(content) = line.strip_prefix('\t') {
                let (author, email, time, summary) = if let Some(meta) = commit_metadata.get(&current_hash) {
                    meta.clone()
                } else {
                    let meta = (
                        if temp_author.is_empty() { "Unknown".to_string() } else { temp_author.clone() },
                        temp_email.clone(),
                        temp_time,
                        temp_summary.clone(),
                    );
                    commit_metadata.insert(current_hash.clone(), meta.clone());
                    meta
                };

                lines.push(crate::domain::BlameHunk {
                    commit_hash: current_hash.clone(),
                    author_name: author,
                    author_email: email,
                    author_timestamp: time,
                    commit_summary: summary,
                    original_line_number: current_orig_line,
                    final_line_number: current_final_line,
                    line_content: content.to_string(),
                });
            } else if let Some(author) = line.strip_prefix("author ") {
                temp_author = author.trim().to_string();
            } else if let Some(mail) = line.strip_prefix("author-mail ") {
                temp_email = mail.trim().trim_matches(['<', '>']).to_string();
            } else if let Some(time_str) = line.strip_prefix("author-time ") {
                temp_time = time_str.trim().parse::<i64>().unwrap_or(0);
            } else if let Some(summary) = line.strip_prefix("summary ") {
                temp_summary = summary.trim().to_string();
            } else {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if (parts.len() == 3 || parts.len() == 4) && parts[0].len() == 40 {
                    current_hash = parts[0].to_string();
                    current_orig_line = parts[1].parse::<u32>().unwrap_or(0);
                    current_final_line = parts[2].parse::<u32>().unwrap_or(0);
                    temp_author.clear();
                    temp_email.clear();
                    temp_time = 0;
                    temp_summary.clear();
                }
            }
        }

        Ok(crate::domain::FileBlame {
            file_path: file_path.to_string(),
            lines,
        })
    }

    /// Queries the current Git bisect state and log for the given repository.
    pub async fn get_bisect_status(
        &self,
        directory_path: &Path,
    ) -> Result<BisectState, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let log_result = self
            .executor
            .execute_git_command(Some(directory_path), &["bisect", "log"])
            .await;

        match log_result {
            Err(_) => Ok(BisectState {
                is_active: false,
                current_commit_hash: None,
                current_commit_summary: None,
                revisions_remaining: None,
                estimated_steps_remaining: None,
                culprit_commit_hash: None,
                culprit_commit_summary: None,
                log_output: String::new(),
            }),
            Ok(log_output) => {
                let current_commit_hash = self
                    .executor
                    .execute_git_command(Some(directory_path), &["rev-parse", "HEAD"])
                    .await
                    .map(|hash| hash.trim().to_string())
                    .ok();

                let current_commit_summary = if let Some(ref hash) = current_commit_hash {
                    self.executor
                        .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", hash])
                        .await
                        .map(|summary| summary.trim().to_string())
                        .ok()
                } else {
                    None
                };

                let mut state = parse_bisect_step_output(
                    &log_output,
                    true,
                    current_commit_hash,
                    current_commit_summary,
                    log_output.clone(),
                );

                if let Some(ref culprit_hash) = state.culprit_commit_hash {
                    state.culprit_commit_summary = self
                        .executor
                        .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", culprit_hash])
                        .await
                        .map(|summary| summary.trim().to_string())
                        .ok();
                }

                Ok(state)
            }
        }
    }

    /// Starts a Git bisect session between a known bad and good commit.
    pub async fn bisect_start(
        &self,
        directory_path: &Path,
        bad_commit_hash: &str,
        good_commit_hash: &str,
    ) -> Result<BisectState, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let start_output = self
            .executor
            .execute_git_command(
                Some(directory_path),
                &["bisect", "start", bad_commit_hash.trim(), good_commit_hash.trim()],
            )
            .await?;

        let log_output = self
            .executor
            .execute_git_command(Some(directory_path), &["bisect", "log"])
            .await
            .unwrap_or_default();

        let current_commit_hash = self
            .executor
            .execute_git_command(Some(directory_path), &["rev-parse", "HEAD"])
            .await
            .map(|hash| hash.trim().to_string())
            .ok();

        let current_commit_summary = if let Some(ref hash) = current_commit_hash {
            self.executor
                .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", hash])
                .await
                .map(|summary| summary.trim().to_string())
                .ok()
        } else {
            None
        };

        let mut state = parse_bisect_step_output(
            &start_output,
            true,
            current_commit_hash,
            current_commit_summary,
            log_output,
        );

        if let Some(ref culprit_hash) = state.culprit_commit_hash {
            state.culprit_commit_summary = self
                .executor
                .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", culprit_hash])
                .await
                .map(|summary| summary.trim().to_string())
                .ok();
        }

        Ok(state)
    }

    /// Records a bisect step decision (good, bad, or skip) and advances the bisect session.
    pub async fn bisect_step(
        &self,
        directory_path: &Path,
        decision: &str,
    ) -> Result<BisectState, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let normalized_decision = decision.trim().to_lowercase();
        if normalized_decision != "good" && normalized_decision != "bad" && normalized_decision != "skip" {
            return Err(ApplicationError::InvalidInput(format!(
                "Invalid bisect decision: {decision}. Must be 'good', 'bad', or 'skip'."
            )));
        }

        let step_output = self
            .executor
            .execute_git_command(Some(directory_path), &["bisect", &normalized_decision])
            .await?;

        let log_output = self
            .executor
            .execute_git_command(Some(directory_path), &["bisect", "log"])
            .await
            .unwrap_or_default();

        let current_commit_hash = self
            .executor
            .execute_git_command(Some(directory_path), &["rev-parse", "HEAD"])
            .await
            .map(|hash| hash.trim().to_string())
            .ok();

        let current_commit_summary = if let Some(ref hash) = current_commit_hash {
            self.executor
                .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", hash])
                .await
                .map(|summary| summary.trim().to_string())
                .ok()
        } else {
            None
        };

        let mut state = parse_bisect_step_output(
            &step_output,
            true,
            current_commit_hash,
            current_commit_summary,
            log_output,
        );

        if let Some(ref culprit_hash) = state.culprit_commit_hash {
            state.culprit_commit_summary = self
                .executor
                .execute_git_command(Some(directory_path), &["log", "-1", "--format=%s", culprit_hash])
                .await
                .map(|summary| summary.trim().to_string())
                .ok();
        }

        Ok(state)
    }

    /// Resets the current Git bisect session, checking out the original branch HEAD.
    pub async fn bisect_reset(&self, directory_path: &Path) -> Result<(), ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        self.executor
            .execute_git_command(Some(directory_path), &["bisect", "reset"])
            .await?;

        Ok(())
    }

    /// Exports repository changes or a specific commit as a unified patch string.
    pub async fn export_patch(
        &self,
        directory_path: &Path,
        scope: &str,
        commit_hash: Option<&str>,
    ) -> Result<String, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        match scope.trim().to_lowercase().as_str() {
            "staged" => {
                self.executor
                    .execute_git_command(Some(directory_path), &["diff", "--cached"])
                    .await
            }
            "unstaged" => {
                self.executor
                    .execute_git_command(Some(directory_path), &["diff"])
                    .await
            }
            "working_tree" => {
                self.executor
                    .execute_git_command(Some(directory_path), &["diff", "HEAD"])
                    .await
            }
            "commit" => {
                let hash = commit_hash.ok_or_else(|| {
                    ApplicationError::InvalidInput("Commit hash is required when exporting commit patch".to_string())
                })?;
                self.executor
                    .execute_git_command(Some(directory_path), &["format-patch", "-1", hash.trim(), "--stdout"])
                    .await
            }
            other => Err(ApplicationError::InvalidInput(format!(
                "Invalid patch export scope: {other}. Expected 'staged', 'unstaged', 'working_tree', or 'commit'."
            ))),
        }
    }

    /// Validates if a unified diff patch can be applied cleanly without conflicts.
    pub async fn check_patch(
        &self,
        directory_path: &Path,
        patch_content: &str,
    ) -> Result<PatchCheckResult, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let mut affected_files = Vec::new();
        for line in patch_content.lines() {
            if let Some(rest) = line.strip_prefix("diff --git a/") {
                if let Some(space_index) = rest.find(" b/") {
                    let file_path = rest[..space_index].to_string();
                    if !affected_files.contains(&file_path) {
                        affected_files.push(file_path);
                    }
                }
            }
        }

        let check_result = self
            .executor
            .execute_git_command_with_stdin(
                Some(directory_path),
                &["apply", "--check", "--whitespace=nowarn", "-"],
                patch_content.as_bytes(),
            )
            .await;

        match check_result {
            Ok(_) => Ok(PatchCheckResult {
                can_apply: true,
                error_message: None,
                affected_files,
            }),
            Err(error) => {
                let error_message = match error {
                    ApplicationError::GitExecutionFailed { message, .. } => message,
                    other => other.to_string(),
                };
                Ok(PatchCheckResult {
                    can_apply: false,
                    error_message: Some(error_message),
                    affected_files,
                })
            }
        }
    }

    /// Applies a unified diff patch to the repository (either working tree or staging index).
    pub async fn apply_patch_file(
        &self,
        directory_path: &Path,
        patch_content: &str,
        is_cached: bool,
        is_reverse: bool,
    ) -> Result<(), ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let mut arguments: Vec<&str> = vec!["apply", "--recount", "--whitespace=nowarn"];
        if is_cached {
            arguments.push("--cached");
        }
        if is_reverse {
            arguments.push("--reverse");
        }
        arguments.push("-");

        self.executor
            .execute_git_command_with_stdin(
                Some(directory_path),
                &arguments,
                patch_content.as_bytes(),
            )
            .await?;

        Ok(())
    }

    /// Fetches all registered Git submodules and their current status.
    pub async fn get_submodules(
        &self,
        directory_path: &Path,
    ) -> Result<Vec<GitSubmoduleSummary>, ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let status_output = self
            .executor
            .execute_git_command(Some(directory_path), &["submodule", "status", "--recursive"])
            .await
            .unwrap_or_default();

        let mut submodules = parse_submodule_status(&status_output);

        // Enrich with URLs and configured branches from .gitmodules if present
        let url_output = self
            .executor
            .execute_git_command(
                Some(directory_path),
                &["config", "--file", ".gitmodules", "--get-regexp", "url"],
            )
            .await
            .unwrap_or_default();

        for line in url_output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let key = parts[0];
                let url = parts[1].to_string();
                if let Some(submodule_identifier) = key.strip_prefix("submodule.").and_then(|k| k.strip_suffix(".url")) {
                    for submodule in &mut submodules {
                        if submodule.name == submodule_identifier || submodule.path == submodule_identifier {
                            submodule.url = Some(url.clone());
                        }
                    }
                }
            }
        }

        Ok(submodules)
    }

    /// Updates one or all Git submodules.
    pub async fn update_submodules(
        &self,
        directory_path: &Path,
        submodule_path: Option<&str>,
        is_init: bool,
        is_recursive: bool,
    ) -> Result<(), ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let mut arguments = vec!["submodule", "update"];
        if is_init {
            arguments.push("--init");
        }
        if is_recursive {
            arguments.push("--recursive");
        }
        if let Some(path) = submodule_path {
            arguments.push(path);
        }

        self.executor
            .execute_git_command(Some(directory_path), &arguments)
            .await?;

        Ok(())
    }

    /// Synchronizes submodules remote URL configuration setting to .gitmodules values.
    pub async fn sync_submodules(
        &self,
        directory_path: &Path,
        submodule_path: Option<&str>,
        is_recursive: bool,
    ) -> Result<(), ApplicationError> {
        let is_valid = self.verify_repository_path(directory_path).await?;
        if !is_valid {
            return Err(ApplicationError::InvalidRepository(
                directory_path.to_string_lossy().to_string(),
            ));
        }

        let mut arguments = vec!["submodule", "sync"];
        if is_recursive {
            arguments.push("--recursive");
        }
        if let Some(path) = submodule_path {
            arguments.push(path);
        }

        self.executor
            .execute_git_command(Some(directory_path), &arguments)
            .await?;

        Ok(())
    }
}
