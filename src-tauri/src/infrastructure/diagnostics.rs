use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const MAXIMUM_DIAGNOSTIC_LOG_ENTRIES: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticLogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticBundle {
    pub application_version: String,
    pub operating_system: String,
    pub architecture: String,
    pub git_binary_path: String,
    pub git_version: String,
    pub current_repository_path: Option<String>,
    pub generated_at_iso: String,
    pub connected_providers: Vec<String>,
    pub recent_log_entries: Vec<DiagnosticLogEntry>,
}

struct DiagnosticLoggerState {
    log_entries: VecDeque<DiagnosticLogEntry>,
    log_file_path: Option<PathBuf>,
}

static DIAGNOSTIC_LOGGER: OnceLock<Mutex<DiagnosticLoggerState>> = OnceLock::new();

fn get_logger_state() -> &'static Mutex<DiagnosticLoggerState> {
    DIAGNOSTIC_LOGGER.get_or_init(|| {
        Mutex::new(DiagnosticLoggerState {
            log_entries: VecDeque::with_capacity(MAXIMUM_DIAGNOSTIC_LOG_ENTRIES),
            log_file_path: None,
        })
    })
}

/// Redacts any sensitive tokens, passwords, or URLs before recording.
pub fn sanitize_sensitive_string(raw_text: &str) -> String {
    let mut sanitized = raw_text.to_string();

    // Redact Git HTTP URLs with credentials: protocol://user:token@host
    if sanitized.contains("://") && sanitized.contains('@') {
        let parts: Vec<&str> = sanitized.split_whitespace().collect();
        let mut replaced_parts = Vec::new();
        for part in parts {
            if part.contains("://") && part.contains('@')
                && let Some((proto, rest)) = part.split_once("://")
                    && let Some((_, host_and_path)) = rest.split_once('@') {
                        replaced_parts.push(format!("{}://***@{}", proto, host_and_path));
                        continue;
                    }
            replaced_parts.push(part.to_string());
        }
        sanitized = replaced_parts.join(" ");
    }

    // Redact GitHub Personal Access Tokens (ghp_*, gho_*, etc.)
    if sanitized.contains("ghp_") || sanitized.contains("gho_") || sanitized.contains("github_pat_")
    {
        let tokens = ["ghp_", "gho_", "github_pat_"];
        for token_prefix in &tokens {
            while let Some(index) = sanitized.find(token_prefix) {
                let end = sanitized[index..]
                    .find(|character: char| {
                        character.is_whitespace() || character == '"' || character == '\''
                    })
                    .map(|offset| index + offset)
                    .unwrap_or(sanitized.len());
                sanitized.replace_range(index..end, "***REDACTED_GITHUB_TOKEN***");
            }
        }
    }

    // Redact GitLab Personal Access Tokens (glpat-*)
    if sanitized.contains("glpat-") {
        while let Some(index) = sanitized.find("glpat-") {
            let end = sanitized[index..]
                .find(|character: char| {
                    character.is_whitespace() || character == '"' || character == '\''
                })
                .map(|offset| index + offset)
                .unwrap_or(sanitized.len());
            sanitized.replace_range(index..end, "***REDACTED_GITLAB_TOKEN***");
        }
    }

    // Redact Bearer authorization headers
    if sanitized.to_lowercase().contains("bearer ") {
        let lower = sanitized.to_lowercase();
        if let Some(index) = lower.find("bearer ") {
            let start = index + 7;
            let end = sanitized[start..]
                .find(|character: char| {
                    character.is_whitespace() || character == '"' || character == '\''
                })
                .map(|offset| start + offset)
                .unwrap_or(sanitized.len());
            sanitized.replace_range(start..end, "***REDACTED_BEARER***");
        }
    }

    sanitized
}

/// Sets the persistent log file destination on disk.
pub fn configure_diagnostic_log_file(file_path: PathBuf) {
    if let Ok(mut state) = get_logger_state().lock() {
        state.log_file_path = Some(file_path);
    }
}

/// Appends a sanitized log entry into the circular buffer and active log file.
pub fn append_diagnostic_log(level: &str, raw_message: &str) {
    let sanitized_message = sanitize_sensitive_string(raw_message);
    let entry = DiagnosticLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: level.to_uppercase(),
        message: sanitized_message,
    };

    if let Ok(mut state) = get_logger_state().lock() {
        if state.log_entries.len() >= MAXIMUM_DIAGNOSTIC_LOG_ENTRIES {
            state.log_entries.pop_front();
        }
        state.log_entries.push_back(entry.clone());

        if let Some(ref path) = state.log_file_path
            && let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(
                    file,
                    "[{}] [{}] {}",
                    entry.timestamp, entry.level, entry.message
                );
            }
    }
}

/// Retrieves the most recent diagnostic log entries.
pub fn get_recent_diagnostic_logs(maximum_count: usize) -> Vec<DiagnosticLogEntry> {
    if let Ok(state) = get_logger_state().lock() {
        let count = state.log_entries.len();
        let skip_count = count.saturating_sub(maximum_count);
        state.log_entries.iter().skip(skip_count).cloned().collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_redacts_credentials_in_git_url() {
        let input = "remote add origin https://oauth2:secret_password@gitlab.com/group/project.git";
        let sanitized = sanitize_sensitive_string(input);
        assert!(!sanitized.contains("secret_password"));
        assert!(sanitized.contains("https://***@gitlab.com/group/project.git"));
    }

    #[test]
    fn test_sanitize_redacts_github_personal_access_token() {
        let input = "Connecting with token ghp_ABC123XYZ999 to API";
        let sanitized = sanitize_sensitive_string(input);
        assert!(!sanitized.contains("ghp_ABC123XYZ999"));
        assert!(sanitized.contains("***REDACTED_GITHUB_TOKEN***"));
    }

    #[test]
    fn test_sanitize_redacts_gitlab_personal_access_token() {
        let input = "Header PRIVATE-TOKEN: glpat-secrettoken123456";
        let sanitized = sanitize_sensitive_string(input);
        assert!(!sanitized.contains("glpat-secrettoken123456"));
        assert!(sanitized.contains("***REDACTED_GITLAB_TOKEN***"));
    }

    #[test]
    fn test_sanitize_redacts_bearer_token() {
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsIn";
        let sanitized = sanitize_sensitive_string(input);
        assert!(!sanitized.contains("eyJhbGciOiJIUzI1NiIsIn"));
        assert!(sanitized.contains("***REDACTED_BEARER***"));
    }

    #[test]
    fn test_diagnostic_logger_stores_and_caps_entries() {
        for index in 0..10 {
            append_diagnostic_log("info", &format!("Log message {}", index));
        }

        let logs = get_recent_diagnostic_logs(5);
        assert_eq!(logs.len(), 5);
        assert_eq!(logs.last().unwrap().message, "Log message 9");
    }
}
