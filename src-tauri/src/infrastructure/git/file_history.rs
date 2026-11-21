use crate::domain::{FileHistoryTimeline, FileRevisionEntry};

pub const DELIMITER: char = '\x1f';

/// Parses raw output from `git log --follow --format=... --name-status` into a FileHistoryTimeline.
pub fn parse_file_history_output(raw_output: &str, file_path: &str) -> FileHistoryTimeline {
    let mut revisions = Vec::new();
    let mut pending_entry: Option<FileRevisionEntry> = None;

    for line in raw_output.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        if trimmed_line.contains(DELIMITER) {
            // Push previous pending entry if any
            if let Some(entry) = pending_entry.take() {
                revisions.push(entry);
            }

            let fields: Vec<&str> = trimmed_line.split(DELIMITER).collect();
            if fields.len() >= 5 {
                let commit_hash = fields[0].trim().to_string();
                let author_name = fields[1].trim().to_string();
                let author_email = fields[2].trim().to_string();
                let author_timestamp = fields[3].trim().parse::<i64>().unwrap_or(0);
                let commit_summary = fields[4].trim().to_string();

                pending_entry = Some(FileRevisionEntry {
                    commit_hash,
                    author_name,
                    author_email,
                    author_timestamp,
                    commit_summary,
                    change_status: "M".to_string(), // Default to modified if status line missing
                });
            }
        } else if let Some(ref mut entry) = pending_entry {
            // This is a status line like "M\tsrc/domain/mod.rs" or "A\tpath" or "R100\told\tnew"
            let status_code = trimmed_line.split_whitespace().next().unwrap_or("M");
            entry.change_status = status_code.to_string();
            revisions.push(pending_entry.take().unwrap());
        }
    }

    if let Some(entry) = pending_entry {
        revisions.push(entry);
    }

    let total_revisions = revisions.len() as u32;
    FileHistoryTimeline {
        file_path: file_path.to_string(),
        revisions,
        total_revisions,
    }
}
