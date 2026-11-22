use crate::domain::{DetailedRepositoryStatus, FileChangeStatus, FileStatusEntry};

/// Parses machine-readable Git status output (porcelain v2, null-terminated or newline-separated).
pub fn parse_porcelain_v2_status(raw_output: &str) -> DetailedRepositoryStatus {
    let mut status = DetailedRepositoryStatus::default();

    // Check if output is null-delimited or newline-delimited
    let entries: Vec<&str> = if raw_output.contains('\0') {
        raw_output.split('\0').filter(|s| !s.is_empty()).collect()
    } else {
        raw_output
            .lines()
            .filter(|s| !s.trim().is_empty())
            .collect()
    };

    let mut iterator = entries.iter().peekable();

    while let Some(entry) = iterator.next() {
        let line = entry.trim();
        if line.is_empty() {
            continue;
        }

        // Branch headers: # branch.<key> <value>
        if let Some(header) = line.strip_prefix('#') {
            let parts: Vec<&str> = header.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "branch.oid" => {
                    if parts.len() > 1 && parts[1] != "(initial)" {
                        status.head_commit_hash = Some(parts[1].to_string());
                    }
                }
                "branch.head" => {
                    if parts.len() > 1 {
                        if parts[1] == "(detached)" {
                            status.is_detached_head = true;
                        } else {
                            status.branch_name = Some(parts[1].to_string());
                        }
                    }
                }
                "branch.upstream" => {
                    if parts.len() > 1 {
                        status.upstream_branch_name = Some(parts[1].to_string());
                    }
                }
                "branch.ab"
                    // Example: +2 -0
                    if parts.len() >= 3 => {
                        if let Some(ahead_str) = parts[1].strip_prefix('+') {
                            status.ahead_count = ahead_str.parse().unwrap_or(0);
                        }
                        if let Some(behind_str) = parts[2].strip_prefix('-') {
                            status.behind_count = behind_str.parse().unwrap_or(0);
                        }
                    }
                _ => {}
            }
            continue;
        }

        // Type 1: Ordinary changed entry
        // Format: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
        if line.starts_with("1 ") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() >= 9 {
                let xy = tokens[1];
                let path = tokens[8..].join(" ");
                let staged_code = xy.chars().next().unwrap_or('.');
                let unstaged_code = xy.chars().nth(1).unwrap_or('.');

                // Check staged
                if let Some(change_status) = map_porcelain_code_to_status(staged_code) {
                    status.staged_files.push(FileStatusEntry {
                        path: path.clone(),
                        original_path: None,
                        status: change_status,
                        is_staged: true,
                    });
                }

                // Check unstaged
                if let Some(change_status) = map_porcelain_code_to_status(unstaged_code) {
                    status.unstaged_files.push(FileStatusEntry {
                        path,
                        original_path: None,
                        status: change_status,
                        is_staged: false,
                    });
                }
            }
        }
        // Type 2: Renamed or copied entry
        // Format: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
        else if line.starts_with("2 ") {
            // First 9 tokens are metadata: 2, <XY>, <sub>, <mH>, <mI>, <mW>, <hH>, <hI>, <X><score>
            // The remainder contains <path><sep><origPath> where sep is tab in normal mode or \0 in -z mode
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            if parts.len() == 10 {
                let xy_tokens: Vec<&str> = parts[1].split_whitespace().collect();
                let xy = xy_tokens.first().copied().unwrap_or(parts[1]);
                let staged_code = xy.chars().next().unwrap_or('.');
                let unstaged_code = xy.chars().nth(1).unwrap_or('.');

                let path_and_orig = parts[9];
                let (path, orig_path) = if let Some((p, orig)) = path_and_orig.split_once('\t') {
                    (p.trim().to_string(), Some(orig.trim().to_string()))
                } else if let Some(next_token) = iterator.peek() {
                    let next_str = next_token.to_string();
                    let _ = iterator.next();
                    (
                        path_and_orig.trim().to_string(),
                        Some(next_str.trim().to_string()),
                    )
                } else {
                    (path_and_orig.trim().to_string(), None)
                };

                if let Some(change_status) = map_porcelain_code_to_status(staged_code) {
                    status.staged_files.push(FileStatusEntry {
                        path: path.clone(),
                        original_path: orig_path.clone(),
                        status: change_status,
                        is_staged: true,
                    });
                }

                if let Some(change_status) = map_porcelain_code_to_status(unstaged_code) {
                    status.unstaged_files.push(FileStatusEntry {
                        path,
                        original_path: orig_path,
                        status: change_status,
                        is_staged: false,
                    });
                }
            }
        }
        // Type u: Unmerged / Conflicted entry
        // Format: u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
        else if line.starts_with("u ") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() >= 11 {
                let path = tokens[10..].join(" ");
                status.conflicted_files.push(FileStatusEntry {
                    path,
                    original_path: None,
                    status: FileChangeStatus::Conflicted,
                    is_staged: false,
                });
            }
        }
        // Type ?: Untracked entry
        // Format: ? <path>
        else if let Some(path) = line.strip_prefix("? ") {
            status.unstaged_files.push(FileStatusEntry {
                path: path.trim().to_string(),
                original_path: None,
                status: FileChangeStatus::Untracked,
                is_staged: false,
            });
        }
    }

    status
}

fn map_porcelain_code_to_status(code: char) -> Option<FileChangeStatus> {
    match code {
        'M' => Some(FileChangeStatus::Modified),
        'A' => Some(FileChangeStatus::Added),
        'D' => Some(FileChangeStatus::Deleted),
        'R' => Some(FileChangeStatus::Renamed),
        'C' => Some(FileChangeStatus::Copied),
        'T' => Some(FileChangeStatus::TypeChanged),
        'U' => Some(FileChangeStatus::Conflicted),
        '?' => Some(FileChangeStatus::Untracked),
        _ => None,
    }
}
