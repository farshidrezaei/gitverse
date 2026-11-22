use crate::domain::{
    ApplicationError, BranchSummary, RemoteBranchSummary, RemoteSummary, TagSummary,
};
use std::collections::HashMap;

pub const REF_FIELD_DELIMITER: char = '\x1f';

/// Parses raw `git for-each-ref` output for local branches.
pub fn parse_local_branches(raw: &str) -> Vec<BranchSummary> {
    let mut branches = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let fields: Vec<&str> = trimmed.split(REF_FIELD_DELIMITER).collect();
        if fields.is_empty() {
            continue;
        }

        let name = fields[0].trim().to_string();
        if name.is_empty() {
            continue;
        }

        let is_head = fields.get(1).is_some_and(|val| val.trim() == "*");
        let upstream_name = fields
            .get(2)
            .map(|val| val.trim().to_string())
            .filter(|val| !val.is_empty());

        let mut ahead_count = 0;
        let mut behind_count = 0;

        if let Some(track) = fields.get(3) {
            for part in track.split(',') {
                let segment = part.trim();
                if let Some(count_str) = segment.strip_prefix("ahead ") {
                    ahead_count = count_str.trim().parse::<u32>().unwrap_or(0);
                } else if let Some(count_str) = segment.strip_prefix("behind ") {
                    behind_count = count_str.trim().parse::<u32>().unwrap_or(0);
                }
            }
        }

        let head_commit_hash = fields
            .get(4)
            .map(|val| val.trim().to_string())
            .filter(|val| !val.is_empty());

        branches.push(BranchSummary {
            name,
            is_head,
            upstream_name,
            ahead_count,
            behind_count,
            head_commit_hash,
        });
    }

    branches
}

/// Parses raw `git for-each-ref` output for remote branches.
pub fn parse_remote_branches(raw: &str) -> Vec<RemoteBranchSummary> {
    let mut remote_branches = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let fields: Vec<&str> = trimmed.split(REF_FIELD_DELIMITER).collect();
        if fields.is_empty() {
            continue;
        }

        let name = fields[0].trim().to_string();
        if name.is_empty() || name.ends_with("/HEAD") || !name.contains('/') {
            continue;
        }

        if let Some(symref) = fields.get(2)
            && !symref.trim().is_empty()
        {
            continue;
        }

        let remote_name = name.split('/').next().unwrap_or("origin").to_string();
        let head_commit_hash = fields
            .get(1)
            .map(|val| val.trim().to_string())
            .filter(|val| !val.is_empty());

        remote_branches.push(RemoteBranchSummary {
            name,
            remote_name,
            head_commit_hash,
        });
    }

    remote_branches
}

/// Parses raw `git tag -l` output.
pub fn parse_tags(raw: &str) -> Vec<TagSummary> {
    let mut tags = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let fields: Vec<&str> = trimmed.split(REF_FIELD_DELIMITER).collect();
        if fields.is_empty() {
            continue;
        }

        let name = fields[0].trim().to_string();
        if name.is_empty() {
            continue;
        }

        let commit_hash = fields
            .get(1)
            .map(|val| val.trim().to_string())
            .unwrap_or_default();

        tags.push(TagSummary { name, commit_hash });
    }

    tags
}

/// Parses raw `git remote -v` output into structured remotes with fetch/push URLs.
pub fn parse_remotes(raw: &str) -> Vec<RemoteSummary> {
    let mut map: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let name = parts[0].to_string();
        let url = parts[1].to_string();
        let kind = parts[2]; // "(fetch)" or "(push)"

        let entry = map.entry(name).or_insert((None, None));
        if kind.contains("fetch") {
            entry.0 = Some(url);
        } else if kind.contains("push") {
            entry.1 = Some(url);
        }
    }

    let mut remotes: Vec<RemoteSummary> = map
        .into_iter()
        .map(|(name, (fetch_url, push_url))| RemoteSummary {
            name,
            fetch_url,
            push_url,
        })
        .collect();

    remotes.sort_by(|a, b| a.name.cmp(&b.name));
    remotes
}

/// Validates a branch name against Git ref format rules before invocation.
pub fn validate_branch_name(candidate_name: &str) -> Result<(), ApplicationError> {
    let trimmed = candidate_name.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot be empty".into(),
        ));
    }

    if trimmed.contains(char::is_whitespace) {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot contain whitespace".into(),
        ));
    }

    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot start or end with a slash".into(),
        ));
    }

    if trimmed.starts_with('-') || trimmed.ends_with('-') {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot start or end with a hyphen".into(),
        ));
    }

    if trimmed.starts_with('.') || trimmed.ends_with('.') {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot start or end with a dot".into(),
        ));
    }

    if trimmed.contains("//") || trimmed.contains("..") {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot contain consecutive separators (// or ..)".into(),
        ));
    }

    if trimmed.ends_with(".lock") {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot end with .lock".into(),
        ));
    }

    if trimmed.contains("@{") {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot contain '@{'".into(),
        ));
    }

    let is_valid_characters = trimmed.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || character == '.'
            || character == '_'
            || character == '-'
            || character == '/'
    });

    if !is_valid_characters {
        return Err(ApplicationError::InvalidInput(
            "Branch name contains illegal characters".into(),
        ));
    }

    Ok(())
}
