use crate::domain::StashRecord;

pub const STASH_FIELD_DELIMITER: char = '\x1f';

/// Parses raw `git stash list --format=...` output into structured StashRecord entries.
pub fn parse_stash_list(raw_output: &str) -> Vec<StashRecord> {
    let mut stashes = Vec::new();

    for line in raw_output.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = trimmed_line.split(STASH_FIELD_DELIMITER).collect();
        if fields.len() < 4 {
            continue;
        }

        let stash_identifier = fields[0].trim().to_string();
        let message = fields[1].trim().to_string();
        let commit_hash = fields[2].trim().to_string();
        let timestamp_seconds = fields[3].trim().parse::<i64>().unwrap_or(0);

        // Extract numeric index from "stash@{N}"
        let index = stash_identifier
            .strip_prefix("stash@{")
            .and_then(|remaining| remaining.strip_suffix('}'))
            .and_then(|index_string| index_string.parse::<usize>().ok())
            .unwrap_or(stashes.len());

        stashes.push(StashRecord {
            index,
            stash_identifier,
            message,
            commit_hash,
            timestamp_seconds,
        });
    }

    stashes
}
