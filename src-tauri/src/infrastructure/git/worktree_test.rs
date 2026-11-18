#[cfg(test)]
mod tests {
    use super::super::worktree::parse_worktree_list;

    #[test]
    fn test_parse_worktree_list_empty_input() {
        assert_eq!(parse_worktree_list(""), vec![]);
        assert_eq!(parse_worktree_list("   \n\n  "), vec![]);
    }

    #[test]
    fn test_parse_worktree_list_standard_multiple() {
        let raw = "\
worktree /home/user/project
HEAD 1234567890abcdef1234567890abcdef12345678
branch refs/heads/master

worktree /home/user/project-feature
HEAD abcdef1234567890abcdef1234567890abcdef12
branch refs/heads/feature/login

worktree /home/user/project-detached
HEAD 9999888877776666555544443333222211110000
detached

worktree /home/user/project-locked
HEAD 1111222233334444555566667777888899990000
branch refs/heads/maintenance
locked Maintenance in progress

worktree /home/user/project-prunable
HEAD 0000111122223333444455556666777788889999
branch refs/heads/stale
prunable gitdir file points to non-existent location
";

        let parsed = parse_worktree_list(raw);
        assert_eq!(parsed.len(), 5);

        // Main worktree
        assert_eq!(parsed[0].directory_path, "/home/user/project");
        assert_eq!(parsed[0].head_commit_hash, "1234567890abcdef1234567890abcdef12345678");
        assert_eq!(parsed[0].branch_name.as_deref(), Some("master"));
        assert!(parsed[0].is_main);
        assert!(!parsed[0].is_bare);
        assert!(!parsed[0].is_locked);
        assert!(!parsed[0].is_prunable);

        // Feature worktree
        assert_eq!(parsed[1].directory_path, "/home/user/project-feature");
        assert_eq!(parsed[1].branch_name.as_deref(), Some("feature/login"));
        assert!(!parsed[1].is_main);

        // Detached worktree
        assert_eq!(parsed[2].directory_path, "/home/user/project-detached");
        assert_eq!(parsed[2].branch_name, None);
        assert!(!parsed[2].is_main);

        // Locked worktree
        assert_eq!(parsed[3].directory_path, "/home/user/project-locked");
        assert!(parsed[3].is_locked);
        assert_eq!(parsed[3].lock_reason.as_deref(), Some("Maintenance in progress"));

        // Prunable worktree
        assert_eq!(parsed[4].directory_path, "/home/user/project-prunable");
        assert!(parsed[4].is_prunable);
    }
}
