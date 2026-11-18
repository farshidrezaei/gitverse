use crate::domain::{DiffHunk, DiffLine, DiffLineType, FileDiff};

/// Parses unified git diff output into a strongly-typed `FileDiff` structure.
pub fn parse_unified_diff(raw_diff: &str, file_path: &str) -> FileDiff {
    if raw_diff.trim().is_empty() {
        return FileDiff {
            old_path: None,
            new_path: file_path.to_string(),
            is_binary: false,
            is_new_file: false,
            is_deleted_file: false,
            hunks: Vec::new(),
            total_additions: 0,
            total_deletions: 0,
            old_content_base64: None,
            new_content_base64: None,
        };
    }

    if raw_diff.contains("Binary files ") || raw_diff.contains("GIT binary patch") {
        return FileDiff {
            old_path: None,
            new_path: file_path.to_string(),
            is_binary: true,
            is_new_file: false,
            is_deleted_file: false,
            hunks: Vec::new(),
            total_additions: 0,
            total_deletions: 0,
            old_content_base64: None,
            new_content_base64: None,
        };
    }

    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;

    let mut is_new = false;
    let mut is_deleted = false;
    let mut old_path: Option<String> = None;
    let mut new_path: String = file_path.to_string();

    let mut current_old_line: u32 = 0;
    let mut current_new_line: u32 = 0;

    let mut total_additions: u32 = 0;
    let mut total_deletions: u32 = 0;

    for line in raw_diff.lines() {
        // Metadata headers
        if line.starts_with("new file mode") {
            is_new = true;
            continue;
        }
        if line.starts_with("deleted file mode") {
            is_deleted = true;
            continue;
        }
        if let Some(path) = line.strip_prefix("--- a/") {
            old_path = Some(path.to_string());
            continue;
        }
        if let Some(path) = line.strip_prefix("+++ b/") {
            new_path = path.to_string();
            continue;
        }

        // Hunk header: @@ -old_start,old_lines +new_start,new_lines @@
        if line.starts_with("@@") {
            if let Some(hunk) = current_hunk.take() {
                hunks.push(hunk);
            }

            if let Some((old_start, old_lines, new_start, new_lines)) = parse_hunk_header(line) {
                current_old_line = old_start;
                current_new_line = new_start;

                current_hunk = Some(DiffHunk {
                    header: line.to_string(),
                    old_start,
                    old_lines,
                    new_start,
                    new_lines,
                    lines: Vec::new(),
                });
            }
            continue;
        }

        // Within a hunk
        if let Some(ref mut hunk) = current_hunk {
            if let Some(content) = line.strip_prefix('+') {
                total_additions += 1;
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Addition,
                    old_line_number: None,
                    new_line_number: Some(current_new_line),
                    content: content.to_string(),
                });
                current_new_line += 1;
            } else if let Some(content) = line.strip_prefix('-') {
                total_deletions += 1;
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Deletion,
                    old_line_number: Some(current_old_line),
                    new_line_number: None,
                    content: content.to_string(),
                });
                current_old_line += 1;
            } else if let Some(content) = line.strip_prefix(' ') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    old_line_number: Some(current_old_line),
                    new_line_number: Some(current_new_line),
                    content: content.to_string(),
                });
                current_old_line += 1;
                current_new_line += 1;
            } else if line == "\\ No newline at end of file" {
                // Ignore EOF marker
                continue;
            }
        }
    }

    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    FileDiff {
        old_path,
        new_path,
        is_binary: false,
        is_new_file: is_new,
        is_deleted_file: is_deleted,
        hunks,
        total_additions,
        total_deletions,
        old_content_base64: None,
        new_content_base64: None,
    }
}

/// Parses "@@ -old_start,old_lines +new_start,new_lines @@"
fn parse_hunk_header(header: &str) -> Option<(u32, u32, u32, u32)> {
    let parts: Vec<&str> = header.split("@@").collect();
    if parts.len() < 3 {
        return None;
    }
    let range_section = parts[1].trim();
    let sections: Vec<&str> = range_section.split_whitespace().collect();
    if sections.len() < 2 {
        return None;
    }

    let (old_start, old_lines) = parse_range(sections[0].strip_prefix('-')?)?;
    let (new_start, new_lines) = parse_range(sections[1].strip_prefix('+')?)?;

    Some((old_start, old_lines, new_start, new_lines))
}

fn parse_range(range_str: &str) -> Option<(u32, u32)> {
    if let Some((start, count)) = range_str.split_once(',') {
        Some((start.parse().ok()?, count.parse().ok()?))
    } else {
        Some((range_str.parse().ok()?, 1))
    }
}
