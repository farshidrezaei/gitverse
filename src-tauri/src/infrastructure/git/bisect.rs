use crate::domain::BisectState;

/// Parses output from `git bisect` commands (`start`, `good`, `bad`, `skip`).
pub fn parse_bisect_step_output(
    command_output: &str,
    is_active: bool,
    current_head_hash: Option<String>,
    current_head_summary: Option<String>,
    log_output: String,
) -> BisectState {
    let mut revisions_remaining = None;
    let mut estimated_steps_remaining = None;
    let mut culprit_commit_hash = None;

    for line in command_output.lines() {
        let trimmed_line = line.trim();

        // Check for culprit: "<hash> is the first 'bad' commit" or "<hash> is the first bad commit"
        if trimmed_line.contains(" is the first ") && trimmed_line.ends_with("commit")
            && let Some(index) = trimmed_line.find(" is the first ") {
                let candidate_hash = trimmed_line[..index].trim();
                if candidate_hash
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
                    && !candidate_hash.is_empty()
                {
                    culprit_commit_hash = Some(candidate_hash.to_string());
                }
            }

        // Check for "Bisecting: X revision(s) left to test after this (roughly Y step(s))"
        if trimmed_line.starts_with("Bisecting:") {
            let after_prefix = trimmed_line.strip_prefix("Bisecting:").unwrap_or("").trim();
            if let Some(revisions_end) = after_prefix.find(" revision")
                && let Ok(count) = after_prefix[..revisions_end].trim().parse::<u32>() {
                    revisions_remaining = Some(count);
                }

            if let Some(steps_start) = after_prefix.find("(roughly ") {
                let after_roughly = &after_prefix[steps_start + 9..];
                if let Some(steps_end) = after_roughly.find(" step")
                    && let Ok(count) = after_roughly[..steps_end].trim().parse::<u32>() {
                        estimated_steps_remaining = Some(count);
                    }
            }
        }
    }

    BisectState {
        is_active,
        current_commit_hash: current_head_hash,
        current_commit_summary: current_head_summary,
        revisions_remaining,
        estimated_steps_remaining,
        culprit_commit_hash,
        culprit_commit_summary: None,
        log_output,
    }
}
