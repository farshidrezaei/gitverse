use crate::domain::{RepositorySearchResult, SearchMatchEntry};

/// Parses raw output from `git grep -n` into structured search match entries.
pub fn parse_git_grep_output(
    raw_output: &str,
    query: &str,
    reference_prefix: Option<&str>,
) -> RepositorySearchResult {
    let mut matches = Vec::new();
    let mut unique_files = Vec::new();

    for line in raw_output.lines() {
        let trimmed_line = line.trim_end();
        if trimmed_line.is_empty() {
            continue;
        }

        let line_to_parse = if let Some(prefix) = reference_prefix {
            let prefix_with_colon = format!("{}:", prefix.trim());
            if let Some(remaining) = trimmed_line.strip_prefix(&prefix_with_colon) {
                remaining
            } else {
                trimmed_line
            }
        } else {
            trimmed_line
        };

        // Split by ':' -> file_path:line_number:content
        let parts: Vec<&str> = line_to_parse.splitn(3, ':').collect();
        if parts.len() < 3 {
            continue;
        }

        let file_path = parts[0].trim().to_string();
        let line_number = parts[1].trim().parse::<u32>().unwrap_or(0);
        let line_content = parts[2].to_string();

        if !unique_files.contains(&file_path) {
            unique_files.push(file_path.clone());
        }

        matches.push(SearchMatchEntry {
            file_path,
            line_number,
            line_content,
        });
    }

    let total_matches = matches.len() as u32;
    let file_count = unique_files.len() as u32;

    RepositorySearchResult {
        query: query.to_string(),
        total_matches,
        file_count,
        matches,
    }
}
