use crate::domain::{CommitChangedFile, FileChangeStatus, GitCommit, GitCommitDecoration, GitReferenceType};

/// Delimiter constants for machine-readable Git log output.
pub const COMMIT_RECORD_DELIMITER: char = '\x1e';
pub const COMMIT_FIELD_DELIMITER: char = '\x1f';
pub const COMMIT_LOG_FORMAT: &str = "%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%b%x1f%D%x1e";

/// Parse a decoration item from Git's `%D` placeholder.
pub fn parse_commit_decorations(decorations_text: &str) -> Vec<GitCommitDecoration> {
    let mut decorations = Vec::new();
    let trimmed = decorations_text.trim();
    if trimmed.is_empty() {
        return decorations;
    }

    for part in trimmed.split(',') {
        let entry = part.trim();
        if entry.is_empty() {
            continue;
        }

        if let Some(branch_target) = entry.strip_prefix("HEAD -> ") {
            decorations.push(GitCommitDecoration {
                name: "HEAD".to_string(),
                reference_type: GitReferenceType::Head,
                is_head: true,
            });
            decorations.push(GitCommitDecoration {
                name: branch_target.trim().to_string(),
                reference_type: GitReferenceType::LocalBranch,
                is_head: true,
            });
        } else if entry == "HEAD" {
            decorations.push(GitCommitDecoration {
                name: "HEAD".to_string(),
                reference_type: GitReferenceType::Head,
                is_head: true,
            });
        } else if let Some(tag_name) = entry.strip_prefix("tag: ") {
            decorations.push(GitCommitDecoration {
                name: tag_name.trim().to_string(),
                reference_type: GitReferenceType::Tag,
                is_head: false,
            });
        } else if entry.contains('/') {
            decorations.push(GitCommitDecoration {
                name: entry.to_string(),
                reference_type: GitReferenceType::RemoteBranch,
                is_head: false,
            });
        } else {
            decorations.push(GitCommitDecoration {
                name: entry.to_string(),
                reference_type: GitReferenceType::LocalBranch,
                is_head: false,
            });
        }
    }

    decorations
}

/// Parse delimited Git log output into a sequence of `GitCommit` items.
pub fn parse_delimited_git_log(raw_output: &str) -> Vec<GitCommit> {
    let mut commits = Vec::new();

    for record in raw_output.split(COMMIT_RECORD_DELIMITER) {
        let trimmed_record = record.trim();
        if trimmed_record.is_empty() {
            continue;
        }

        let fields: Vec<&str> = trimmed_record.split(COMMIT_FIELD_DELIMITER).collect();
        if fields.len() < 11 {
            // Insufficient fields for a complete commit record
            continue;
        }

        let hash = fields[0].trim().to_string();
        if hash.is_empty() {
            continue;
        }

        let parent_hashes = fields[1]
            .split_whitespace()
            .map(|parent| parent.trim().to_string())
            .filter(|parent| !parent.is_empty())
            .collect();

        let author_name = fields[2].trim().to_string();
        let author_email = fields[3].trim().to_string();
        let author_timestamp = fields[4].trim().parse::<i64>().unwrap_or(0);

        let committer_name = fields[5].trim().to_string();
        let committer_email = fields[6].trim().to_string();
        let committer_timestamp = fields[7].trim().parse::<i64>().unwrap_or(0);

        let subject = fields[8].trim().to_string();
        let body = fields[9].trim().to_string();
        let decorations = parse_commit_decorations(fields[10]);

        commits.push(GitCommit {
            hash,
            parent_hashes,
            author_name,
            author_email,
            author_timestamp,
            committer_name,
            committer_email,
            committer_timestamp,
            subject,
            body,
            decorations,
        });
    }

    commits
}

/// Parse git diff-tree name-status and numstat outputs into structured `CommitChangedFile` list.
pub fn parse_commit_changed_files(name_status_output: &str, numstat_output: &str) -> Vec<CommitChangedFile> {
    let name_status_lines: Vec<&str> = name_status_output
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    let numstat_lines: Vec<&str> = numstat_output
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    let mut changed_files = Vec::new();

    for (index, name_line) in name_status_lines.iter().enumerate() {
        let columns: Vec<&str> = name_line.split('\t').collect();
        if columns.is_empty() {
            continue;
        }

        let raw_status = columns[0].trim();
        let (status, original_path, path) = if raw_status.starts_with('R') || raw_status.starts_with('C') {
            if columns.len() >= 3 {
                let status_variant = if raw_status.starts_with('R') {
                    FileChangeStatus::Renamed
                } else {
                    FileChangeStatus::Copied
                };
                (status_variant, Some(columns[1].to_string()), columns[2].to_string())
            } else if columns.len() == 2 {
                (FileChangeStatus::Renamed, None, columns[1].to_string())
            } else {
                continue;
            }
        } else {
            let status_variant = match raw_status.chars().next() {
                Some('A') => FileChangeStatus::Added,
                Some('D') => FileChangeStatus::Deleted,
                Some('M') => FileChangeStatus::Modified,
                Some('T') => FileChangeStatus::TypeChanged,
                _ => FileChangeStatus::Modified,
            };
            if columns.len() >= 2 {
                (status_variant, None, columns[1].to_string())
            } else {
                continue;
            }
        };

        // Extract additions and deletions from corresponding numstat line if available
        let (additions, deletions) = if let Some(num_line) = numstat_lines.get(index) {
            let num_parts: Vec<&str> = num_line.split('\t').collect();
            if num_parts.len() >= 2 {
                let adds = num_parts[0].trim().parse::<u32>().unwrap_or(0);
                let dels = num_parts[1].trim().parse::<u32>().unwrap_or(0);
                (adds, dels)
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        changed_files.push(CommitChangedFile {
            path,
            original_path,
            status,
            additions,
            deletions,
        });
    }

    changed_files
}

/// Parses Git %G?%x1f%GS%x1f%GK signature format into a structured CommitSignature.
pub fn parse_commit_signature(output: &str) -> Option<crate::domain::CommitSignature> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parts: Vec<&str> = trimmed.split('\x1f').collect();
    let status_code = parts.first().copied().unwrap_or("N").trim();

    let status = match status_code {
        "G" | "U" => crate::domain::CommitSignatureStatus::Verified,
        "B" | "E" | "X" | "Y" | "R" => crate::domain::CommitSignatureStatus::Unverified,
        _ => crate::domain::CommitSignatureStatus::None,
    };

    if status == crate::domain::CommitSignatureStatus::None {
        return None;
    }

    let signer_name = parts
        .get(1)
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .map(String::from);

    let key_id = parts
        .get(2)
        .map(|key| key.trim())
        .filter(|key| !key.is_empty())
        .map(String::from);

    Some(crate::domain::CommitSignature {
        status,
        signer_name,
        key_id,
    })
}

/// Parses delimited reflog records produced with format `%gd%x1f%h%x1f%H%x1f%gs%x1f%ct%x1f%an%x1e`.
pub fn parse_reflog_output(output: &str) -> Vec<crate::domain::ReflogRecord> {
    let mut records = Vec::new();
    for entry in output.split('\x1e') {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }

        let fields: Vec<&str> = trimmed.split('\x1f').collect();
        if fields.len() < 6 {
            continue;
        }

        let selector = fields[0].trim().to_string();
        let commit_hash = fields[1].trim().to_string();
        let full_hash = fields[2].trim().to_string();
        let action = fields[3].trim().to_string();
        let timestamp_seconds = fields[4].trim().parse::<i64>().unwrap_or(0);
        let author_name = fields[5].trim().to_string();

        records.push(crate::domain::ReflogRecord {
            selector,
            commit_hash,
            full_hash,
            action,
            timestamp_seconds,
            author_name,
        });
    }

    records
}
