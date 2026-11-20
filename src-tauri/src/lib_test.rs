#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use crate::domain::{ProviderAccount, ProviderKind};
    use crate::infrastructure::git::GitProcessExecutor;
    use crate::infrastructure::storage::SQLiteRepositoryStore;

    #[tokio::test]
    async fn test_git_binary_discovery() {
        let discovery_result = GitProcessExecutor::discover_system_git(None).await;
        assert!(discovery_result.is_ok(), "Git discovery should succeed on system");
        let info = discovery_result.unwrap();
        assert!(info.is_valid);
        assert!(info.version_string.contains("git version"));
    }

    #[test]
    fn test_sqlite_repository_store_crud() {
        let temporary_dir = tempdir().expect("Failed to create tempdir");
        let database_file = temporary_dir.path().join("test_store.sqlite");

        let store = SQLiteRepositoryStore::new(&database_file).expect("Failed to create store");

        // Record opened repository
        let record = store
            .record_opened_repository("/home/user/project_alpha", "project_alpha")
            .expect("Failed to record");
        assert_eq!(record.name, "project_alpha");

        // Retrieve recents
        let recents = store.get_recent_repositories(10).expect("Failed to fetch recents");
        assert_eq!(recents.len(), 1);
        assert_eq!(recents[0].path, "/home/user/project_alpha");
        assert!(recents[0].is_missing);

        // Record another and test clear
        store
            .record_opened_repository("/home/user/project_beta", "project_beta")
            .expect("Failed to record beta");
        assert_eq!(store.get_recent_repositories(10).expect("fetch").len(), 2);
        store.clear_recent_repositories().expect("clear failed");
        assert_eq!(store.get_recent_repositories(10).expect("fetch").len(), 0);

        // Re-record and Remove
        store
            .record_opened_repository("/home/user/project_alpha", "project_alpha")
            .expect("Failed to record");
        store
            .remove_recent_repository("/home/user/project_alpha")
            .expect("Failed to remove");
        let updated_recents = store.get_recent_repositories(10).expect("Failed to fetch");
        assert_eq!(updated_recents.len(), 0);
    }

    #[test]
    fn test_sqlite_provider_account_crud() {
        let temporary_dir = tempdir().expect("Failed to create tempdir");
        let database_file = temporary_dir.path().join("test_provider_store.sqlite");

        let store = SQLiteRepositoryStore::new(&database_file).expect("Failed to create store");

        let account = ProviderAccount {
            id: "github:octocat".to_string(),
            provider: ProviderKind::GitHub,
            username: "octocat".to_string(),
            display_name: Some("The Octocat".to_string()),
            email: Some("octocat@github.com".to_string()),
            avatar_url: Some("https://avatars.githubusercontent.com/u/583231".to_string()),
            instance_url: "https://github.com".to_string(),
            created_at_timestamp: 1700000000,
        };

        // Save account
        store.save_provider_account(&account).expect("Save account should succeed");

        // List accounts
        let accounts = store.list_provider_accounts().expect("List should succeed");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].username, "octocat");
        assert_eq!(accounts[0].provider, ProviderKind::GitHub);

        // Delete account
        let deleted = store.delete_provider_account("github:octocat").expect("Delete should succeed");
        assert!(deleted);

        // Verify empty
        let after_delete = store.list_provider_accounts().expect("List after delete");
        assert_eq!(after_delete.len(), 0);
    }

    #[tokio::test]
    async fn test_get_commit_file_diff_on_merge_commit() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create tempdir");
        let repository_path = temporary_dir.path();

        // Initialize Git repo
        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        // Initial commit on main
        std::fs::write(repository_path.join("base.txt"), "base content\n").expect("write base.txt");
        Command::new("git").args(["add", "base.txt"]).current_dir(repository_path).output().expect("git add");
        Command::new("git").args(["commit", "-m", "initial commit"]).current_dir(repository_path).output().expect("git commit");

        // Branch feature
        Command::new("git").args(["checkout", "-b", "feature"]).current_dir(repository_path).output().expect("git checkout -b");
        std::fs::write(repository_path.join("feature.txt"), "feature content line 1\nfeature content line 2\n").expect("write feature.txt");
        Command::new("git").args(["add", "feature.txt"]).current_dir(repository_path).output().expect("git add feature");
        Command::new("git").args(["commit", "-m", "feat: add feature"]).current_dir(repository_path).output().expect("git commit feat");

        // Back to main and merge
        Command::new("git").args(["checkout", "main"]).current_dir(repository_path).output().expect("git checkout main");
        Command::new("git").args(["merge", "--no-ff", "feature", "-m", "Merge pull request #1"]).current_dir(repository_path).output().expect("git merge");

        let merge_hash_output = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repository_path).output().expect("git rev-parse");
        let merge_hash = String::from_utf8_lossy(&merge_hash_output.stdout).trim().to_string();

        let executor_info = GitProcessExecutor::discover_system_git(None).await.expect("discover git");
        let executor = GitProcessExecutor::new(executor_info.executable_path.into());
        let inspector = GitRepositoryInspector::new(executor);

        let diff = inspector
            .get_commit_file_diff(repository_path, &merge_hash, Some("feature.txt"))
            .await
            .expect("get_commit_file_diff");

        assert_eq!(diff.new_path, "feature.txt");
        assert!(!diff.is_binary);
        assert_eq!(diff.total_additions, 2);
        assert_eq!(diff.total_deletions, 0);
        assert_eq!(diff.hunks.len(), 1);
    }

    #[tokio::test]
    async fn test_stage_unstage_and_discard_patch() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        std::fs::write(repository_path.join("doc.txt"), "line1\nline2\nline3\nline4\n").expect("write doc.txt");
        Command::new("git").args(["add", "doc.txt"]).current_dir(repository_path).output().expect("git add doc.txt");
        Command::new("git").args(["commit", "-m", "init doc"]).current_dir(repository_path).output().expect("git commit");

        std::fs::write(repository_path.join("doc.txt"), "line1\nline2_modified\nline3\nline4_modified\n").expect("write modified doc.txt");

        let executor_info = GitProcessExecutor::discover_system_git(None).await.expect("discover git");
        let executor = GitProcessExecutor::new(executor_info.executable_path.into());
        let inspector = GitRepositoryInspector::new(executor);

        // Stage only the first hunk/change via patch
        let patch_for_first_change = "--- a/doc.txt\n+++ b/doc.txt\n@@ -1,3 +1,3 @@\n line1\n-line2\n+line2_modified\n line3\n";
        inspector.stage_patch(repository_path, patch_for_first_change).await.expect("stage patch");

        let status_after_staging = inspector.get_detailed_status(repository_path).await.expect("get status");
        assert_eq!(status_after_staging.staged_files.len(), 1, "doc.txt should be in staged");
        assert_eq!(status_after_staging.unstaged_files.len(), 1, "doc.txt should also be in unstaged");

        // Unstage that patch
        inspector.unstage_patch(repository_path, patch_for_first_change).await.expect("unstage patch");
        let status_after_unstaging = inspector.get_detailed_status(repository_path).await.expect("get status");
        assert_eq!(status_after_unstaging.staged_files.len(), 0, "staged should be empty after unstage_patch");
        assert_eq!(status_after_unstaging.unstaged_files.len(), 1, "unstaged should still contain doc.txt");

        // Discard that first change from working tree
        inspector.discard_patch(repository_path, patch_for_first_change).await.expect("discard patch");
        let content_after_discard = std::fs::read_to_string(repository_path.join("doc.txt")).expect("read file");
        assert_eq!(content_after_discard, "line1\nline2\nline3\nline4_modified\n", "line2_modified should be reverted while line4_modified remains");
    }

    #[tokio::test]
    async fn test_compare_commits_and_get_comparison_file_diff() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        // Commit 1
        std::fs::write(repository_path.join("alpha.txt"), "hello alpha\n").expect("write alpha");
        Command::new("git").args(["add", "alpha.txt"]).current_dir(repository_path).output().expect("add alpha");
        Command::new("git").args(["commit", "-m", "first commit"]).current_dir(repository_path).output().expect("commit 1");
        let hash_1_output = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repository_path).output().expect("rev-parse 1");
        let hash_1 = String::from_utf8_lossy(&hash_1_output.stdout).trim().to_string();

        // Commit 2
        std::fs::write(repository_path.join("alpha.txt"), "hello alpha updated\n").expect("write alpha 2");
        std::fs::write(repository_path.join("beta.txt"), "hello beta\n").expect("write beta");
        Command::new("git").args(["add", "alpha.txt", "beta.txt"]).current_dir(repository_path).output().expect("add alpha beta");
        Command::new("git").args(["commit", "-m", "second commit"]).current_dir(repository_path).output().expect("commit 2");
        let hash_2_output = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repository_path).output().expect("rev-parse 2");
        let hash_2 = String::from_utf8_lossy(&hash_2_output.stdout).trim().to_string();

        let executor_info = GitProcessExecutor::discover_system_git(None).await.expect("discover git");
        let executor = GitProcessExecutor::new(executor_info.executable_path.into());
        let inspector = GitRepositoryInspector::new(executor);

        let comparison = inspector.compare_commits(repository_path, &hash_1, &hash_2).await.expect("compare commits");
        assert_eq!(comparison.base_hash, hash_1);
        assert_eq!(comparison.target_hash, hash_2);
        assert_eq!(comparison.base_subject.as_deref(), Some("first commit"));
        assert_eq!(comparison.target_subject.as_deref(), Some("second commit"));
        assert_eq!(comparison.changed_files.len(), 2, "alpha.txt and beta.txt changed between commits");

        let diff_alpha = inspector.get_comparison_file_diff(repository_path, &hash_1, &hash_2, Some("alpha.txt")).await.expect("comparison diff alpha");
        assert_eq!(diff_alpha.new_path, "alpha.txt");
        assert_eq!(diff_alpha.total_additions, 1);
        assert_eq!(diff_alpha.total_deletions, 1);
    }

    #[tokio::test]
    async fn test_remote_management_operations() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create tempdir");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");

        let executor_info = GitProcessExecutor::discover_system_git(None).await.expect("discover git");
        let executor = GitProcessExecutor::new(executor_info.executable_path.into());
        let inspector = GitRepositoryInspector::new(executor);

        // 1. Add remote
        inspector.add_remote(repository_path, "origin", "https://example.com/gitverse/core.git")
            .await
            .expect("add origin");

        let references = inspector.get_repository_references(repository_path).await.expect("refs");
        let origin = references.remotes.iter().find(|r| r.name == "origin").expect("origin exists");
        assert_eq!(origin.fetch_url.as_deref(), Some("https://example.com/gitverse/core.git"));

        // 2. Set remote URL
        inspector.set_remote_url(repository_path, "origin", "https://example.com/gitverse/updated.git", false)
            .await
            .expect("set url");
        let references_updated = inspector.get_repository_references(repository_path).await.expect("refs updated");
        let origin_updated = references_updated.remotes.iter().find(|r| r.name == "origin").expect("origin updated");
        assert_eq!(origin_updated.fetch_url.as_deref(), Some("https://example.com/gitverse/updated.git"));

        // 3. Rename remote
        inspector.rename_remote(repository_path, "origin", "upstream")
            .await
            .expect("rename origin to upstream");
        let references_renamed = inspector.get_repository_references(repository_path).await.expect("refs renamed");
        assert!(references_renamed.remotes.iter().any(|r| r.name == "upstream"));
        assert!(!references_renamed.remotes.iter().any(|r| r.name == "origin"));

        // 4. Remove remote
        inspector.remove_remote(repository_path, "upstream")
            .await
            .expect("remove upstream");
        let references_cleaned = inspector.get_repository_references(repository_path).await.expect("refs cleaned");
        assert!(references_cleaned.remotes.is_empty());
    }

    #[tokio::test]
    async fn test_get_file_blame_extraction() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        let test_file = repository_path.join("tracked.txt");
        std::fs::write(&test_file, "First line\nSecond line\n").expect("write file");

        Command::new("git").args(["add", "tracked.txt"]).current_dir(repository_path).output().expect("git add");
        Command::new("git").args(["commit", "-m", "initial commit of tracked file"]).current_dir(repository_path).output().expect("git commit");

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        let blame = inspector.get_file_blame(repository_path, "tracked.txt", None)
            .await
            .expect("Extract file blame");

        assert_eq!(blame.file_path, "tracked.txt");
        assert_eq!(blame.lines.len(), 2);
        assert_eq!(blame.lines[0].final_line_number, 1);
        assert_eq!(blame.lines[0].line_content, "First line");
        assert_eq!(blame.lines[0].commit_summary, "initial commit of tracked file");
        assert_eq!(blame.lines[1].final_line_number, 2);
        assert_eq!(blame.lines[1].line_content, "Second line");
    }

    #[tokio::test]
    async fn test_get_stash_details_and_diff() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        std::fs::write(repository_path.join("file.txt"), "Original text\n").expect("write file");
        Command::new("git").args(["add", "file.txt"]).current_dir(repository_path).output().expect("git add");
        Command::new("git").args(["commit", "-m", "initial commit"]).current_dir(repository_path).output().expect("git commit");

        std::fs::write(repository_path.join("file.txt"), "Modified stash text\n").expect("modify file");
        Command::new("git").args(["stash", "push", "-m", "work in progress"]).current_dir(repository_path).output().expect("git stash");

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        let details = inspector.get_stash_details(repository_path, 0)
            .await
            .expect("Get stash details");

        assert_eq!(details.index, 0);
        assert!(details.message.contains("work in progress"));
        assert_eq!(details.changed_files.len(), 1);
        assert_eq!(details.changed_files[0].path, "file.txt");

        let diff = inspector.get_stash_file_diff(repository_path, 0, "file.txt")
            .await
            .expect("Get stash file diff");

        assert_eq!(diff.new_path, "file.txt");
        assert_eq!(diff.total_additions, 1);
        assert_eq!(diff.total_deletions, 1);
    }

    #[tokio::test]
    async fn test_open_in_external_editor_validation() {
        use crate::commands::repository::open_in_external_editor;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path().to_string_lossy().to_string();

        let unsupported_result = open_in_external_editor(
            repository_path.clone(),
            None,
            "unsupported_editor_name".to_string(),
        ).await;

        assert!(unsupported_result.is_err());

        let invalid_directory_result = open_in_external_editor(
            "/nonexistent/directory/path/that/cannot/exist/ever".to_string(),
            None,
            "vscode".to_string(),
        ).await;

        assert!(invalid_directory_result.is_err());
    }

    #[tokio::test]
    async fn test_commit_with_bypass_hooks() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        // Write a failing pre-commit hook that exits with 1
        let hooks_dir = repository_path.join(".git").join("hooks");
        std::fs::create_dir_all(&hooks_dir).expect("create hooks dir");
        let pre_commit_path = hooks_dir.join("pre-commit");
        std::fs::write(&pre_commit_path, "#!/bin/sh\nexit 1\n").expect("write pre-commit");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&pre_commit_path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&pre_commit_path, permissions).expect("set permissions");
        }

        std::fs::write(repository_path.join("hook_test.txt"), "testing hooks\n").expect("write file");
        Command::new("git").args(["add", "hook_test.txt"]).current_dir(repository_path).output().expect("git add");

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        // Without bypass_hooks, the commit should fail because pre-commit exits with 1
        let failed_commit = inspector.commit(repository_path, "failing hook commit", false, false).await;
        assert!(failed_commit.is_err());

        // With bypass_hooks = true (--no-verify), the commit should succeed!
        let successful_commit = inspector.commit(repository_path, "bypassed hook commit", false, true).await;
        assert!(successful_commit.is_ok());
    }

    #[tokio::test]
    async fn test_get_commit_template() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        // Initially no template
        let empty_template = inspector.get_commit_template(repository_path).await.expect("template check");
        assert_eq!(empty_template, None);

        // Create .gitmessage fallback
        let message_content = "feat: type short summary here\n\nDetailed description...";
        std::fs::write(repository_path.join(".gitmessage"), message_content).expect("write .gitmessage");

        let detected_template = inspector.get_commit_template(repository_path).await.expect("detected template");
        assert_eq!(detected_template, Some(message_content.to_string()));
    }

    #[tokio::test]
    async fn test_git_bisect_workflow() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        let mut commit_hashes = Vec::new();
        for index in 0..5 {
            std::fs::write(repository_path.join("file.txt"), format!("content {index}\n")).expect("write file");
            Command::new("git").args(["add", "file.txt"]).current_dir(repository_path).output().expect("git add");
            Command::new("git").args(["commit", "-m", &format!("commit {index}")]).current_dir(repository_path).output().expect("git commit");
            let output = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repository_path).output().expect("rev-parse");
            commit_hashes.push(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        // Verify inactive initially
        let initial_status = inspector.get_bisect_status(repository_path).await.expect("get bisect status");
        assert!(!initial_status.is_active);

        // Start bisect with bad = commit_hashes[4] and good = commit_hashes[0]
        let started_status = inspector
            .bisect_start(repository_path, &commit_hashes[4], &commit_hashes[0])
            .await
            .expect("bisect start");

        assert!(started_status.is_active);
        assert!(started_status.current_commit_hash.is_some());

        // Reset bisect session
        let reset_result = inspector.bisect_reset(repository_path).await;
        assert!(reset_result.is_ok());

        let final_status = inspector.get_bisect_status(repository_path).await.expect("get bisect status after reset");
        assert!(!final_status.is_active);
    }

    #[tokio::test]
    async fn test_export_check_and_apply_patch() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(repository_path).output().expect("git config email");

        std::fs::write(repository_path.join("file.txt"), "line 1\nline 2\n").expect("write file");
        Command::new("git").args(["add", "file.txt"]).current_dir(repository_path).output().expect("git add");
        Command::new("git").args(["commit", "-m", "initial commit"]).current_dir(repository_path).output().expect("git commit");

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        // Modify file
        std::fs::write(repository_path.join("file.txt"), "line 1\nmodified line 2\nline 3\n").expect("write file");

        // Export patch for unstaged changes
        let patch = inspector.export_patch(repository_path, "unstaged", None).await.expect("export patch");
        assert!(patch.contains("modified line 2"));

        // Restore clean working tree
        Command::new("git").args(["checkout", "--", "."]).current_dir(repository_path).output().expect("git checkout");
        let clean_content = std::fs::read_to_string(repository_path.join("file.txt")).expect("read file");
        assert_eq!(clean_content, "line 1\nline 2\n");

        // Pre-check valid patch
        let check_clean = inspector.check_patch(repository_path, &patch).await.expect("check patch");
        assert!(check_clean.can_apply);
        assert_eq!(check_clean.affected_files, vec!["file.txt"]);

        // Apply patch
        inspector.apply_patch_file(repository_path, &patch, false, false).await.expect("apply patch");
        let applied_content = std::fs::read_to_string(repository_path.join("file.txt")).expect("read file");
        assert_eq!(applied_content, "line 1\nmodified line 2\nline 3\n");

        // Verify invalid patch check fails gracefully
        let invalid_patch = patch.replace("line 1", "nonexistent line");
        let check_invalid = inspector.check_patch(repository_path, &invalid_patch).await.expect("check invalid patch");
        assert!(!check_invalid.can_apply);
        assert!(check_invalid.error_message.is_some());
    }

    #[tokio::test]
    async fn test_submodules_operations() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let root_path = temporary_dir.path();

        let sub_path = root_path.join("child");
        let parent_path = root_path.join("parent");
        std::fs::create_dir_all(&sub_path).expect("create child dir");
        std::fs::create_dir_all(&parent_path).expect("create parent dir");

        // Init child repo
        Command::new("git").args(["init", "-b", "main"]).current_dir(&sub_path).output().expect("child init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(&sub_path).output().expect("child name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(&sub_path).output().expect("child email");
        std::fs::write(sub_path.join("child_file.txt"), "hello submodule\n").expect("write child file");
        Command::new("git").args(["add", "child_file.txt"]).current_dir(&sub_path).output().expect("child add");
        Command::new("git").args(["commit", "-m", "child initial"]).current_dir(&sub_path).output().expect("child commit");

        // Init parent repo
        Command::new("git").args(["init", "-b", "main"]).current_dir(&parent_path).output().expect("parent init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(&parent_path).output().expect("parent name");
        Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(&parent_path).output().expect("parent email");
        std::fs::write(parent_path.join("parent_file.txt"), "hello parent\n").expect("write parent file");
        Command::new("git").args(["add", "parent_file.txt"]).current_dir(&parent_path).output().expect("parent add");
        Command::new("git").args(["commit", "-m", "parent initial"]).current_dir(&parent_path).output().expect("parent commit");

        // Add submodule
        let add_output = Command::new("git")
            .args(["-c", "protocol.file.allow=always", "submodule", "add", sub_path.to_str().unwrap(), "libs/child"])
            .current_dir(&parent_path)
            .output()
            .expect("submodule add");
        assert!(add_output.status.success());

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        let submodules = inspector.get_submodules(&parent_path).await.expect("get submodules");
        assert_eq!(submodules.len(), 1);
        assert_eq!(submodules[0].path, "libs/child");
        assert_eq!(submodules[0].name, "child");
        assert!(submodules[0].is_initialized);

        let sync_result = inspector.sync_submodules(&parent_path, None, true).await;
        assert!(sync_result.is_ok());

        let update_result = inspector.update_submodules(&parent_path, None, true, true).await;
        assert!(update_result.is_ok());
    }

    #[tokio::test]
    async fn test_repository_insights_extraction() {
        use std::process::Command;
        use crate::infrastructure::git::GitRepositoryInspector;

        let temporary_dir = tempdir().expect("Failed to create temporary directory");
        let repository_path = temporary_dir.path();

        Command::new("git").args(["init", "-b", "main"]).current_dir(repository_path).output().expect("git init");
        Command::new("git").args(["config", "user.name", "Tester"]).current_dir(repository_path).output().expect("git config name");
        Command::new("git").args(["config", "user.email", "tester@test.com"]).current_dir(repository_path).output().expect("git config email");

        for index in 0..3 {
            std::fs::write(repository_path.join(format!("file_{index}.txt")), format!("line {index}\n")).expect("write file");
            Command::new("git").args(["add", "."]).current_dir(repository_path).output().expect("git add");
            Command::new("git").args(["commit", "-m", &format!("commit {index}")]).current_dir(repository_path).output().expect("git commit");
        }

        let executor = GitProcessExecutor::new(std::path::PathBuf::from("git"));
        let inspector = GitRepositoryInspector::new(executor);

        let insights = inspector.get_repository_insights(repository_path).await.expect("get insights");
        assert_eq!(insights.total_commits, 3);
        assert_eq!(insights.total_contributors, 1);
        assert_eq!(insights.top_contributors[0].name, "Tester");
        assert_eq!(insights.top_contributors[0].commit_count, 3);
        assert_eq!(insights.activity_by_weekday.len(), 7);
        let total_activity: u32 = insights.activity_by_weekday.iter().map(|day| day.count).sum();
        assert_eq!(total_activity, 3);
    }
}
