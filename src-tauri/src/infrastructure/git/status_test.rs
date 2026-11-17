#[cfg(test)]
mod tests {
    use crate::infrastructure::git::parse_porcelain_v2_status;
    use crate::domain::FileChangeStatus;

    #[test]
    fn test_parse_porcelain_v2_status_comprehensive() {
        let sample_output = "\
# branch.oid 3ad5d4c9f1a2\n\
# branch.head master\n\
# branch.upstream origin/master\n\
# branch.ab +3 -1\n\
1 .M N... 100644 100644 100644 039562 039562 src/main.rs\n\
1 M. N... 100644 100644 100644 039562 039562 app.vue\n\
1 A. N... 000000 100644 100644 000000 039562 new_file.txt\n\
2 R. N... 100644 100644 100644 039562 039562 R100 renamed.rs\told_name.rs\n\
u UU N... 100644 100644 100644 100644 039562 039562 039562 conflict.txt\n\
? untracked_script.py\n";

        let parsed = parse_porcelain_v2_status(sample_output);

        assert_eq!(parsed.branch_name, Some("master".to_string()));
        assert_eq!(parsed.upstream_branch_name, Some("origin/master".to_string()));
        assert_eq!(parsed.ahead_count, 3);
        assert_eq!(parsed.behind_count, 1);
        assert!(!parsed.is_detached_head);

        // Staged files: app.vue (Modified), new_file.txt (Added), renamed.rs (Renamed)
        assert_eq!(parsed.staged_files.len(), 3);
        assert_eq!(parsed.staged_files[0].path, "app.vue");
        assert_eq!(parsed.staged_files[0].status, FileChangeStatus::Modified);
        assert_eq!(parsed.staged_files[1].path, "new_file.txt");
        assert_eq!(parsed.staged_files[1].status, FileChangeStatus::Added);
        assert_eq!(parsed.staged_files[2].path, "renamed.rs");
        assert_eq!(parsed.staged_files[2].original_path, Some("old_name.rs".to_string()));

        // Unstaged files: src/main.rs (Modified), untracked_script.py (Untracked)
        assert_eq!(parsed.unstaged_files.len(), 2);
        assert_eq!(parsed.unstaged_files[0].path, "src/main.rs");
        assert_eq!(parsed.unstaged_files[0].status, FileChangeStatus::Modified);
        assert_eq!(parsed.unstaged_files[1].path, "untracked_script.py");
        assert_eq!(parsed.unstaged_files[1].status, FileChangeStatus::Untracked);

        // Conflicted files: conflict.txt
        assert_eq!(parsed.conflicted_files.len(), 1);
        assert_eq!(parsed.conflicted_files[0].path, "conflict.txt");
        assert_eq!(parsed.conflicted_files[0].status, FileChangeStatus::Conflicted);
    }

    #[test]
    fn test_repository_operation_state_serialization() {
        use crate::domain::RepositoryOperationState;

        let states = vec![
            (RepositoryOperationState::None, "\"none\""),
            (RepositoryOperationState::Merging, "\"merging\""),
            (RepositoryOperationState::Rebasing, "\"rebasing\""),
            (RepositoryOperationState::CherryPicking, "\"cherrypicking\""),
            (RepositoryOperationState::Reverting, "\"reverting\""),
        ];

        for (state, expected_json) in states {
            let serialized = serde_json::to_string(&state).expect("serialize state");
            assert_eq!(serialized, expected_json);
            let deserialized: RepositoryOperationState = serde_json::from_str(&serialized).expect("deserialize state");
            assert_eq!(deserialized, state);
        }
    }
}
