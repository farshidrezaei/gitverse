use crate::domain::{ActivityDay, ContributorInsight, RepositoryInsights};

/// Parses `git shortlog -sne HEAD` output into contributor insights.
pub fn parse_shortlog_contributors(
    shortlog_output: &str,
    total_commits: u32,
) -> (Vec<ContributorInsight>, u32) {
    let mut contributors = Vec::new();

    for line in shortlog_output.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        // Format: "94\tFarshid Rezaei <farshid.net1@gmail.com>"
        let parts: Vec<&str> = trimmed_line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }

        let commit_count = parts[0].trim().parse::<u32>().unwrap_or(0);
        let name_and_email = if parts.len() > 1 { parts[1].trim() } else { "" };

        let (name, email) = if let Some(open_bracket) = name_and_email.find('<') {
            let name_part = name_and_email[..open_bracket].trim();
            let email_part = name_and_email[open_bracket + 1..]
                .trim_end_matches('>')
                .trim();
            (name_part.to_string(), email_part.to_string())
        } else {
            (name_and_email.to_string(), String::new())
        };

        let percentage = if total_commits > 0 {
            (commit_count as f32 / total_commits as f32) * 100.0
        } else {
            0.0
        };

        contributors.push(ContributorInsight {
            name,
            email,
            commit_count,
            percentage,
        });
    }

    let total_contributors = contributors.len() as u32;
    (contributors, total_contributors)
}

/// Parses unix timestamp lines from `git log --format=%ct` and aggregates commit frequency by weekday.
pub fn parse_weekday_activity(timestamps_output: &str) -> Vec<ActivityDay> {
    let mut weekday_counts = [0u32; 7];

    for line in timestamps_output.lines() {
        let trimmed_line = line.trim();
        if let Ok(timestamp) = trimmed_line.parse::<i64>()
            && timestamp >= 0
        {
            // 1970-01-01 was Thursday = 4. (0 = Sunday, 1 = Monday, ..., 6 = Saturday)
            let days_since_epoch = timestamp / 86400;
            let weekday_index = ((days_since_epoch + 4) % 7) as usize;
            weekday_counts[weekday_index] += 1;
        }
    }

    (0..7)
        .map(|day_of_week| ActivityDay {
            day_of_week: day_of_week as u8,
            count: weekday_counts[day_of_week],
        })
        .collect()
}

/// Parses git `--shortstat` output into additions and deletions count.
pub fn parse_shortstat(shortstat_output: &str) -> (u64, u64) {
    let mut lines_added = 0u64;
    let mut lines_deleted = 0u64;

    for part in shortstat_output.split(',') {
        let trimmed_part = part.trim();
        if trimmed_part.contains("insertion") {
            let count_string = trimmed_part.split_whitespace().next().unwrap_or("0");
            lines_added = count_string.parse::<u64>().unwrap_or(0);
        } else if trimmed_part.contains("deletion") {
            let count_string = trimmed_part.split_whitespace().next().unwrap_or("0");
            lines_deleted = count_string.parse::<u64>().unwrap_or(0);
        }
    }

    (lines_added, lines_deleted)
}

/// Assembles complete repository insights from parsed pieces.
pub fn assemble_repository_insights(
    shortlog_output: &str,
    timestamps_output: &str,
    shortstat_output: &str,
    total_commits: u32,
) -> RepositoryInsights {
    let (top_contributors, total_contributors) =
        parse_shortlog_contributors(shortlog_output, total_commits);
    let activity_by_weekday = parse_weekday_activity(timestamps_output);
    let (lines_added, lines_deleted) = parse_shortstat(shortstat_output);

    RepositoryInsights {
        total_commits,
        total_contributors,
        top_contributors,
        activity_by_weekday,
        lines_added,
        lines_deleted,
    }
}
