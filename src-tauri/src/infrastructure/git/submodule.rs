use crate::domain::GitSubmoduleSummary;

/// Parses output from `git submodule status --recursive`.
pub fn parse_submodule_status(status_output: &str) -> Vec<GitSubmoduleSummary> {
    let mut submodules = Vec::new();

    for line in status_output.lines() {
        let trimmed_line = line.trim_end();
        if trimmed_line.len() < 42 {
            continue;
        }

        let prefix_character = trimmed_line.chars().next().unwrap_or(' ');
        let is_initialized = prefix_character != '-';
        let has_revision_divergence = prefix_character == '+';
        let has_uncommitted_changes = prefix_character == 'U';

        let after_prefix = trimmed_line[1..].trim_start();
        let parts: Vec<&str> = after_prefix.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let commit_hash = parts[0].to_string();
        let path = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            String::new()
        };

        let branch = if let Some(open_parenthesis) = trimmed_line.find('(') {
            if let Some(close_parenthesis) = trimmed_line.rfind(')') {
                if close_parenthesis > open_parenthesis {
                    Some(trimmed_line[open_parenthesis + 1..close_parenthesis].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let name = path.rsplit('/').next().unwrap_or(&path).to_string();

        submodules.push(GitSubmoduleSummary {
            path,
            name,
            url: None,
            branch,
            current_commit_hash: commit_hash,
            is_initialized,
            has_uncommitted_changes,
            has_revision_divergence,
        });
    }

    submodules
}
