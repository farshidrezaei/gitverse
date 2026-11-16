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
}
