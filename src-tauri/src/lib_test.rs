#[cfg(test)]
mod tests {
    use tempfile::tempdir;
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

        // Remove
        store
            .remove_recent_repository("/home/user/project_alpha")
            .expect("Failed to remove");
        let updated_recents = store.get_recent_repositories(10).expect("Failed to fetch");
        assert_eq!(updated_recents.len(), 0);
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
}
