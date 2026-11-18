use crate::domain::GitWorktreeSummary;

/// Parses the output of `git worktree list --porcelain` into structured worktree summaries.
pub fn parse_worktree_list(raw_output: &str) -> Vec<GitWorktreeSummary> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_head: Option<String> = None;
    let mut current_branch: Option<String> = None;
    let mut is_bare = false;
    let mut is_locked = false;
    let mut lock_reason: Option<String> = None;
    let mut is_prunable = false;

    let flush_current = |worktrees: &mut Vec<GitWorktreeSummary>,
                         current_path: &mut Option<String>,
                         current_head: &mut Option<String>,
                         current_branch: &mut Option<String>,
                         is_bare: &mut bool,
                         is_locked: &mut bool,
                         lock_reason: &mut Option<String>,
                         is_prunable: &mut bool| {
        if let Some(path) = current_path.take() {
            let is_main = worktrees.is_empty();
            worktrees.push(GitWorktreeSummary {
                directory_path: path,
                head_commit_hash: current_head.take().unwrap_or_default(),
                branch_name: current_branch.take(),
                is_bare: *is_bare,
                is_locked: *is_locked,
                lock_reason: lock_reason.take(),
                is_prunable: *is_prunable,
                is_main,
            });
            *is_bare = false;
            *is_locked = false;
            *is_prunable = false;
        }
    };

    for line in raw_output.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            flush_current(
                &mut worktrees,
                &mut current_path,
                &mut current_head,
                &mut current_branch,
                &mut is_bare,
                &mut is_locked,
                &mut lock_reason,
                &mut is_prunable,
            );
            continue;
        }

        if let Some(path) = trimmed_line.strip_prefix("worktree ") {
            flush_current(
                &mut worktrees,
                &mut current_path,
                &mut current_head,
                &mut current_branch,
                &mut is_bare,
                &mut is_locked,
                &mut lock_reason,
                &mut is_prunable,
            );
            current_path = Some(path.trim().to_string());
        } else if let Some(head) = trimmed_line.strip_prefix("HEAD ") {
            current_head = Some(head.trim().to_string());
        } else if let Some(branch_reference) = trimmed_line.strip_prefix("branch ") {
            let clean_branch = branch_reference
                .trim()
                .strip_prefix("refs/heads/")
                .unwrap_or(branch_reference.trim());
            current_branch = Some(clean_branch.to_string());
        } else if trimmed_line == "bare" {
            is_bare = true;
        } else if trimmed_line == "detached" {
            current_branch = None;
        } else if let Some(reason) = trimmed_line.strip_prefix("locked") {
            is_locked = true;
            let reason_text = reason.trim();
            if !reason_text.is_empty() {
                lock_reason = Some(reason_text.to_string());
            }
        } else if trimmed_line.starts_with("prunable") {
            is_prunable = true;
        }
    }

    flush_current(
        &mut worktrees,
        &mut current_path,
        &mut current_head,
        &mut current_branch,
        &mut is_bare,
        &mut is_locked,
        &mut lock_reason,
        &mut is_prunable,
    );

    worktrees
}
