use crate::domain::ApplicationError;

/// Validates a Git branch name according to git-check-ref-format specification.
pub fn validate_branch_name(candidate_name: &str) -> Result<(), ApplicationError> {
    let trimmed = candidate_name.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot be empty".into(),
        ));
    }

    if trimmed.len() > 250 {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot exceed 250 characters".into(),
        ));
    }

    if trimmed
        .chars()
        .any(|character| character.is_whitespace() || character.is_ascii_control())
    {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot contain whitespace or control characters".into(),
        ));
    }

    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot start or end with a slash".into(),
        ));
    }

    if trimmed.starts_with('-') || trimmed.ends_with('-') {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot start or end with a hyphen".into(),
        ));
    }

    if trimmed.starts_with('.') || trimmed.ends_with('.') {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot start or end with a dot".into(),
        ));
    }

    if trimmed.contains("//") || trimmed.contains("..") {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot contain consecutive slashes (//) or dots (..)".into(),
        ));
    }

    if trimmed.ends_with(".lock") {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot end with .lock".into(),
        ));
    }

    if trimmed.contains("@{") {
        return Err(ApplicationError::InvalidInput(
            "Branch name cannot contain '@{'".into(),
        ));
    }

    let has_invalid_characters = trimmed
        .chars()
        .any(|character| matches!(character, '~' | '^' | ':' | '?' | '*' | '[' | '\\' | '@'));

    if has_invalid_characters {
        return Err(ApplicationError::InvalidInput(
            "Branch name contains invalid characters (~, ^, :, ?, *, [, \\, @)".into(),
        ));
    }

    Ok(())
}

/// Validates a Git remote name.
pub fn validate_remote_name(candidate_name: &str) -> Result<(), ApplicationError> {
    let trimmed = candidate_name.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Remote name cannot be empty".into(),
        ));
    }

    if trimmed.len() > 100 {
        return Err(ApplicationError::InvalidInput(
            "Remote name cannot exceed 100 characters".into(),
        ));
    }

    if trimmed
        .chars()
        .any(|character| character.is_whitespace() || character.is_ascii_control())
    {
        return Err(ApplicationError::InvalidInput(
            "Remote name cannot contain whitespace or control characters".into(),
        ));
    }

    if trimmed == "." || trimmed == ".." {
        return Err(ApplicationError::InvalidInput(
            "Remote name cannot be '.' or '..'".into(),
        ));
    }

    if trimmed.contains('/') {
        return Err(ApplicationError::InvalidInput(
            "Remote name cannot contain a slash".into(),
        ));
    }

    if trimmed.starts_with('-') || trimmed.ends_with('-') {
        return Err(ApplicationError::InvalidInput(
            "Remote name cannot start or end with a hyphen".into(),
        ));
    }

    if trimmed.contains("//") || trimmed.contains("..") {
        return Err(ApplicationError::InvalidInput(
            "Remote name cannot contain consecutive slashes (//) or dots (..)".into(),
        ));
    }

    let has_invalid_characters = trimmed
        .chars()
        .any(|character| matches!(character, '~' | '^' | ':' | '?' | '*' | '[' | '\\' | '@'));

    if has_invalid_characters {
        return Err(ApplicationError::InvalidInput(
            "Remote name contains invalid characters (~, ^, :, ?, *, [, \\, @)".into(),
        ));
    }

    Ok(())
}

/// Validates a Git remote URL.
pub fn validate_remote_url(candidate_url: &str) -> Result<(), ApplicationError> {
    let trimmed = candidate_url.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Remote URL cannot be empty".into(),
        ));
    }

    if trimmed.len() > 2000 {
        return Err(ApplicationError::InvalidInput(
            "Remote URL cannot exceed 2000 characters".into(),
        ));
    }

    if trimmed
        .chars()
        .any(|character| character.is_whitespace() || character.is_ascii_control())
    {
        return Err(ApplicationError::InvalidInput(
            "Remote URL cannot contain whitespace or control characters".into(),
        ));
    }

    // Supported Git URL formats:
    // 1. Standard protocol schemes: https://, http://, ssh://, git://, file://
    // 2. SCP-like SSH syntax: user@host:path/to/repo.git or host:path/to/repo.git
    // 3. Local filesystem paths: /absolute/path or ./relative or C:\path
    let is_standard_url = trimmed.starts_with("https://")
        || trimmed.starts_with("http://")
        || trimmed.starts_with("ssh://")
        || trimmed.starts_with("git://")
        || trimmed.starts_with("file://");

    let is_scp_ssh = trimmed.contains(':')
        && !trimmed.contains("://")
        && (trimmed.contains('@')
            || trimmed
                .chars()
                .take_while(|c| *c != ':')
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-'));

    let is_local_path = trimmed.starts_with('/')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || (trimmed.len() >= 3
            && trimmed
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
            && &trimmed[1..3] == ":\\");

    if !is_standard_url && !is_scp_ssh && !is_local_path {
        return Err(ApplicationError::InvalidInput(
            "Remote URL must be a valid Git URL (HTTPS, SSH, git://, or local path)".into(),
        ));
    }

    Ok(())
}

/// Validates a commit summary and description message.
pub fn validate_commit_message(summary: &str) -> Result<(), ApplicationError> {
    let trimmed = summary.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Commit summary cannot be empty".into(),
        ));
    }

    if trimmed.len() > 1000 {
        return Err(ApplicationError::InvalidInput(
            "Commit summary cannot exceed 1000 characters".into(),
        ));
    }

    if trimmed.contains('\0') {
        return Err(ApplicationError::InvalidInput(
            "Commit summary cannot contain null bytes".into(),
        ));
    }

    Ok(())
}

/// Validates an optional stash message.
pub fn validate_stash_message(message: &str) -> Result<(), ApplicationError> {
    let trimmed = message.trim();

    if trimmed.len() > 500 {
        return Err(ApplicationError::InvalidInput(
            "Stash message cannot exceed 500 characters".into(),
        ));
    }

    if trimmed
        .chars()
        .any(|character| character.is_ascii_control() && character != '\n' && character != '\t')
    {
        return Err(ApplicationError::InvalidInput(
            "Stash message cannot contain control characters".into(),
        ));
    }

    Ok(())
}

/// Validates a worktree path.
pub fn validate_worktree_path(path_str: &str) -> Result<(), ApplicationError> {
    let trimmed = path_str.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Worktree destination path cannot be empty".into(),
        ));
    }

    if trimmed.contains('\0') {
        return Err(ApplicationError::InvalidInput(
            "Worktree destination path cannot contain null bytes".into(),
        ));
    }

    Ok(())
}

/// Validates an archive or bundle filesystem path.
pub fn validate_archive_path(path_str: &str) -> Result<(), ApplicationError> {
    let trimmed = path_str.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Archive or bundle path cannot be empty".into(),
        ));
    }

    if trimmed.len() > 1000 {
        return Err(ApplicationError::InvalidInput(
            "Archive or bundle path cannot exceed 1000 characters".into(),
        ));
    }

    if trimmed.contains('\0') {
        return Err(ApplicationError::InvalidInput(
            "Archive or bundle path cannot contain null bytes".into(),
        ));
    }

    Ok(())
}

/// Validates a Git LFS file pattern.
pub fn validate_lfs_pattern(pattern: &str) -> Result<(), ApplicationError> {
    let trimmed = pattern.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Git LFS pattern cannot be empty".into(),
        ));
    }

    if trimmed.len() > 500 {
        return Err(ApplicationError::InvalidInput(
            "Git LFS pattern cannot exceed 500 characters".into(),
        ));
    }

    if trimmed.contains('\0') || trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(ApplicationError::InvalidInput(
            "Git LFS pattern cannot contain control characters or newlines".into(),
        ));
    }

    Ok(())
}

/// Validates a custom action definition.
pub fn validate_custom_action_definition(
    action: &crate::domain::CustomActionDefinition,
) -> Result<(), ApplicationError> {
    let id_trimmed = action.action_identifier.trim();
    if id_trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Custom action identifier cannot be empty".into(),
        ));
    }

    if id_trimmed.len() > 100 {
        return Err(ApplicationError::InvalidInput(
            "Custom action identifier cannot exceed 100 characters".into(),
        ));
    }

    if !id_trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
    {
        return Err(ApplicationError::InvalidInput(
            "Custom action identifier can only contain alphanumeric characters, underscores, and hyphens".into(),
        ));
    }

    let name_trimmed = action.display_name.trim();
    if name_trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Custom action display name cannot be empty".into(),
        ));
    }

    if name_trimmed.len() > 100 {
        return Err(ApplicationError::InvalidInput(
            "Custom action display name cannot exceed 100 characters".into(),
        ));
    }

    let command_trimmed = action.command_string.trim();
    if command_trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Custom action command string cannot be empty".into(),
        ));
    }

    if command_trimmed.len() > 2000 {
        return Err(ApplicationError::InvalidInput(
            "Custom action command string cannot exceed 2000 characters".into(),
        ));
    }

    Ok(())
}

/// Validates a Git search query.
pub fn validate_search_query(query: &str) -> Result<(), ApplicationError> {
    let trimmed = query.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Search query cannot be empty".into(),
        ));
    }

    if trimmed.len() > 1000 {
        return Err(ApplicationError::InvalidInput(
            "Search query cannot exceed 1000 characters".into(),
        ));
    }

    if trimmed.contains('\0') {
        return Err(ApplicationError::InvalidInput(
            "Search query cannot contain null bytes".into(),
        ));
    }

    Ok(())
}

/// Validates a Git commit reference or hash for bisect / checkout.
pub fn validate_commit_reference(reference: &str) -> Result<(), ApplicationError> {
    let trimmed = reference.trim();

    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Commit reference cannot be empty".into(),
        ));
    }

    if trimmed.len() > 250 {
        return Err(ApplicationError::InvalidInput(
            "Commit reference cannot exceed 250 characters".into(),
        ));
    }

    if trimmed.contains('\0') || trimmed.chars().any(|character| character.is_whitespace()) {
        return Err(ApplicationError::InvalidInput(
            "Commit reference cannot contain whitespace or null bytes".into(),
        ));
    }

    Ok(())
}

/// Validates patch content to be applied.
pub fn validate_patch_content(patch: &str) -> Result<(), ApplicationError> {
    if patch.trim().is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Patch content cannot be empty".into(),
        ));
    }

    if patch.len() > 10_000_000 {
        return Err(ApplicationError::InvalidInput(
            "Patch content cannot exceed 10 megabytes".into(),
        ));
    }

    Ok(())
}

/// Validates workflow run ID (numeric or alphanumeric, non-empty, safe string).
pub fn validate_workflow_run_id(run_id: &str) -> Result<(), ApplicationError> {
    let trimmed = run_id.trim();
    if trimmed.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Workflow run ID cannot be empty".into(),
        ));
    }

    if trimmed.contains('\0') {
        return Err(ApplicationError::InvalidInput(
            "Workflow run ID cannot contain null bytes".into(),
        ));
    }

    if trimmed.len() > 100 {
        return Err(ApplicationError::InvalidInput(
            "Workflow run ID cannot exceed 100 characters".into(),
        ));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ApplicationError::InvalidInput(
            "Workflow run ID contains illegal characters".into(),
        ));
    }

    Ok(())
}

/// Validates pull request creation arguments.
pub fn validate_create_pull_request(
    title: &str,
    source_branch: &str,
    target_branch: &str,
) -> Result<(), ApplicationError> {
    let trimmed_title = title.trim();
    if trimmed_title.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "Pull request title cannot be empty".into(),
        ));
    }
    if trimmed_title.contains('\0') {
        return Err(ApplicationError::InvalidInput(
            "Pull request title cannot contain null bytes".into(),
        ));
    }
    if trimmed_title.len() > 300 {
        return Err(ApplicationError::InvalidInput(
            "Pull request title cannot exceed 300 characters".into(),
        ));
    }

    validate_branch_name(source_branch)?;
    validate_branch_name(target_branch)?;

    if source_branch.trim() == target_branch.trim() {
        return Err(ApplicationError::InvalidInput(
            "Source and target branches cannot be identical".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_branch_name() {
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("feature/add-remotes").is_ok());
        assert!(validate_branch_name("release-v1.0.0").is_ok());

        assert!(validate_branch_name("").is_err());
        assert!(validate_branch_name("   ").is_err());
        assert!(validate_branch_name("feature branch").is_err());
        assert!(validate_branch_name("/leading").is_err());
        assert!(validate_branch_name("trailing/").is_err());
        assert!(validate_branch_name("-hyphen").is_err());
        assert!(validate_branch_name(".dot").is_err());
        assert!(validate_branch_name("double//slash").is_err());
        assert!(validate_branch_name("double..dot").is_err());
        assert!(validate_branch_name("branch.lock").is_err());
        assert!(validate_branch_name("branch@{upstream}").is_err());
        assert!(validate_branch_name("branch~1").is_err());
    }

    #[test]
    fn test_validate_remote_name() {
        assert!(validate_remote_name("origin").is_ok());
        assert!(validate_remote_name("upstream").is_ok());
        assert!(validate_remote_name("fork_2026").is_ok());

        assert!(validate_remote_name("").is_err());
        assert!(validate_remote_name("origin with space").is_err());
        assert!(validate_remote_name("/origin").is_err());
        assert!(validate_remote_name(".").is_err());
        assert!(validate_remote_name("..").is_err());
        assert!(validate_remote_name("origin:bad").is_err());
    }

    #[test]
    fn test_validate_remote_url() {
        assert!(validate_remote_url("https://github.com/owner/repo.git").is_ok());
        assert!(validate_remote_url("http://github.com/owner/repo.git").is_ok());
        assert!(validate_remote_url("git@github.com:owner/repo.git").is_ok());
        assert!(validate_remote_url("ssh://git@github.com:22/owner/repo.git").is_ok());
        assert!(validate_remote_url("git://example.com/repo.git").is_ok());
        assert!(validate_remote_url("/home/user/repos/local.git").is_ok());

        assert!(validate_remote_url("").is_err());
        assert!(validate_remote_url("   ").is_err());
        assert!(validate_remote_url("not a url").is_err());
        assert!(validate_remote_url("ftp://unsupported.com").is_err());
    }

    #[test]
    fn test_validate_commit_and_stash_message() {
        assert!(validate_commit_message("feat: initial commit").is_ok());
        assert!(validate_commit_message("").is_err());
        assert!(validate_commit_message("   ").is_err());

        assert!(validate_stash_message("WIP on new feature").is_ok());
        assert!(validate_stash_message("").is_ok());
    }

    #[test]
    fn test_validate_archive_and_lfs_and_search() {
        assert!(validate_archive_path("/tmp/bundle.bundle").is_ok());
        assert!(validate_archive_path("").is_err());
        assert!(validate_archive_path("/tmp/bad\0path").is_err());

        assert!(validate_lfs_pattern("*.psd").is_ok());
        assert!(validate_lfs_pattern("assets/*.bin").is_ok());
        assert!(validate_lfs_pattern("").is_err());
        assert!(validate_lfs_pattern("bad\npattern").is_err());

        assert!(validate_search_query("my_function").is_ok());
        assert!(validate_search_query("").is_err());
        assert!(validate_search_query("bad\0query").is_err());

        assert!(validate_commit_reference("HEAD").is_ok());
        assert!(validate_commit_reference("a1b2c3d4e5f6").is_ok());
        assert!(validate_commit_reference("v1.0.0").is_ok());
        assert!(validate_commit_reference("").is_err());
        assert!(validate_commit_reference("bad ref with spaces").is_err());

        assert!(validate_patch_content("diff --git a/f b/f").is_ok());
        assert!(validate_patch_content("").is_err());
    }

    #[test]
    fn test_validate_workflow_run_id() {
        assert!(validate_workflow_run_id("12345678").is_ok());
        assert!(validate_workflow_run_id("run-abc-123").is_ok());
        assert!(validate_workflow_run_id("").is_err());
        assert!(validate_workflow_run_id("   ").is_err());
        assert!(validate_workflow_run_id("bad id with spaces").is_err());
        assert!(validate_workflow_run_id("bad\0null").is_err());
    }

    #[test]
    fn test_validate_create_pull_request() {
        assert!(
            validate_create_pull_request("feat: add cool feature", "feature-x", "main").is_ok()
        );
        assert!(validate_create_pull_request("", "feature-x", "main").is_err());
        assert!(validate_create_pull_request("title", "same-branch", "same-branch").is_err());
        assert!(validate_create_pull_request("title", "invalid branch name", "main").is_err());
    }

    #[test]
    fn test_domain_validation_table_driven_matrix() {
        // Table 1: Branch Name Matrix
        let branch_cases: Vec<(&str, bool)> = vec![
            ("main", true),
            ("feature/my-task_123", true),
            ("release-2.0.0-rc1", true),
            ("user/jane.doe/branch", true),
            ("", false),
            ("   ", false),
            ("has space", false),
            ("/leading-slash", false),
            ("trailing-slash/", false),
            ("-leading-dash", false),
            ("trailing-dash-", false),
            (".leading-dot", false),
            ("trailing-dot.", false),
            ("consecutive//slashes", false),
            ("consecutive..dots", false),
            ("branch.lock", false),
            ("branch@{upstream}", false),
            ("branch~1", false),
            ("branch^2", false),
            ("branch:colon", false),
            ("branch?mark", false),
            ("branch*star", false),
            ("branch[bracket", false),
            ("branch\\backslash", false),
            ("branch\0null", false),
        ];

        for (input, expected_valid) in branch_cases {
            assert_eq!(
                validate_branch_name(input).is_ok(),
                expected_valid,
                "Branch validation mismatch for input: '{}'",
                input
            );
        }

        // Table 2: Remote Name Matrix
        let remote_name_cases: Vec<(&str, bool)> = vec![
            ("origin", true),
            ("upstream", true),
            ("fork-2026", true),
            ("backup_mirror", true),
            ("", false),
            ("   ", false),
            ("origin with space", false),
            ("origin/slash", false),
            (".", false),
            ("..", false),
            ("remote:bad", false),
            ("remote\0null", false),
        ];

        for (input, expected_valid) in remote_name_cases {
            assert_eq!(
                validate_remote_name(input).is_ok(),
                expected_valid,
                "Remote name validation mismatch for input: '{}'",
                input
            );
        }

        // Table 3: Remote URL Matrix
        let remote_url_cases: Vec<(&str, bool)> = vec![
            ("https://github.com/owner/repo.git", true),
            ("http://gitlab.company.org/team/project.git", true),
            ("git@github.com:owner/repo.git", true),
            ("ssh://git@host.com:22/repo.git", true),
            ("git://kernel.org/pub/scm/git/git.git", true),
            ("/local/path/to/repo.git", true),
            ("", false),
            ("   ", false),
            ("ftp://unsupported.com/repo.git", false),
            ("random string without protocol", false),
            ("https://\0bad", false),
        ];

        for (input, expected_valid) in remote_url_cases {
            assert_eq!(
                validate_remote_url(input).is_ok(),
                expected_valid,
                "Remote URL validation mismatch for input: '{}'",
                input
            );
        }

        // Table 4: Commit Message Matrix
        let commit_message_cases: Vec<(&str, bool)> = vec![
            ("feat: standard commit message", true),
            ("fix(core): detailed multi-line\n\nBody of message.", true),
            ("پیام تغییر به زبان فارسی با فونت استاندارد", true),
            ("", false),
            ("   \n\t  ", false),
            ("message\0with-null", false),
        ];

        for (input, expected_valid) in commit_message_cases {
            assert_eq!(
                validate_commit_message(input).is_ok(),
                expected_valid,
                "Commit message validation mismatch for input: '{}'",
                input
            );
        }
    }
}
