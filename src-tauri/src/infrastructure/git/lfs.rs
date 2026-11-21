use crate::domain::{GitLfsLockEntry, GitLfsOverview, GitLfsTrackedFile};

/// Parses `.gitattributes` or `git lfs track` output to extract tracked file patterns.
pub fn parse_lfs_attributes(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            if trimmed.contains("filter=lfs") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(pattern) = parts.first() {
                    return Some((*pattern).to_string());
                }
            }
            None
        })
        .collect()
}

/// Parses output from `git lfs ls-files -l` into tracked file entries.
/// Format: `<oid> * <path>` or `<oid> - <path>`
pub fn parse_lfs_ls_files(output: &str) -> Vec<GitLfsTrackedFile> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let object_identifier = parts[0].to_string();
                let file_path = parts[2..].join(" ");
                Some(GitLfsTrackedFile {
                    file_path,
                    object_identifier,
                    size_in_bytes: 0,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Parses output from `git lfs locks` into lock entries.
/// Format: `<path>\t<owner>\tID:<id>`
pub fn parse_lfs_locks(output: &str) -> Vec<GitLfsLockEntry> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() >= 3 {
                let file_path = parts[0].trim().to_string();
                let owner_name = parts[1].trim().to_string();
                let lock_identifier = parts[2]
                    .trim()
                    .strip_prefix("ID:")
                    .unwrap_or(parts[2].trim())
                    .to_string();
                Some(GitLfsLockEntry {
                    lock_identifier,
                    file_path,
                    owner_name,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Constructs a fallback or empty LFS overview when LFS is uninstalled or empty.
pub fn build_empty_lfs_overview(is_lfs_installed: bool) -> GitLfsOverview {
    GitLfsOverview {
        is_lfs_installed,
        tracked_patterns: Vec::new(),
        tracked_files: Vec::new(),
        locks: Vec::new(),
        total_lfs_size_in_bytes: 0,
    }
}
