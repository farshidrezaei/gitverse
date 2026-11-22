#[cfg(test)]
mod tests {
    use crate::domain::FileChangeStatus;
    use crate::infrastructure::git::parse_porcelain_v2_status;

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
        assert_eq!(
            parsed.upstream_branch_name,
            Some("origin/master".to_string())
        );
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
        assert_eq!(
            parsed.staged_files[2].original_path,
            Some("old_name.rs".to_string())
        );

        // Unstaged files: src/main.rs (Modified), untracked_script.py (Untracked)
        assert_eq!(parsed.unstaged_files.len(), 2);
        assert_eq!(parsed.unstaged_files[0].path, "src/main.rs");
        assert_eq!(parsed.unstaged_files[0].status, FileChangeStatus::Modified);
        assert_eq!(parsed.unstaged_files[1].path, "untracked_script.py");
        assert_eq!(parsed.unstaged_files[1].status, FileChangeStatus::Untracked);

        // Conflicted files: conflict.txt
        assert_eq!(parsed.conflicted_files.len(), 1);
        assert_eq!(parsed.conflicted_files[0].path, "conflict.txt");
        assert_eq!(
            parsed.conflicted_files[0].status,
            FileChangeStatus::Conflicted
        );
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
            let deserialized: RepositoryOperationState =
                serde_json::from_str(&serialized).expect("deserialize state");
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_porcelain_v2_status_matrix() {
        struct StatusMatrixTestCase {
            description: &'static str,
            raw_input: &'static str,
            expected_branch: Option<&'static str>,
            expected_head_hash: Option<&'static str>,
            expected_upstream: Option<&'static str>,
            expected_ahead: u32,
            expected_behind: u32,
            expected_is_detached: bool,
            expected_staged_count: usize,
            expected_unstaged_count: usize,
            expected_conflicted_count: usize,
        }

        let matrix_cases = vec![
            StatusMatrixTestCase {
                description: "Clean attached repository on main with upstream in sync",
                raw_input: "# branch.oid 112233445566778899aabbccddeeff0011223344\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +0 -0\n",
                expected_branch: Some("main"),
                expected_head_hash: Some("112233445566778899aabbccddeeff0011223344"),
                expected_upstream: Some("origin/main"),
                expected_ahead: 0,
                expected_behind: 0,
                expected_is_detached: false,
                expected_staged_count: 0,
                expected_unstaged_count: 0,
                expected_conflicted_count: 0,
            },
            StatusMatrixTestCase {
                description: "Detached HEAD with ahead/behind and unstaged deletion",
                raw_input: "# branch.oid deadbeefcafebabe0123456789abcdef01234567\n# branch.head (detached)\n1 .D N... 100644 100644 000000 039562 000000 deleted_unstaged.txt\n",
                expected_branch: None,
                expected_head_hash: Some("deadbeefcafebabe0123456789abcdef01234567"),
                expected_upstream: None,
                expected_ahead: 0,
                expected_behind: 0,
                expected_is_detached: true,
                expected_staged_count: 0,
                expected_unstaged_count: 1,
                expected_conflicted_count: 0,
            },
            StatusMatrixTestCase {
                description: "Initial unborn branch with staged addition and untracked file",
                raw_input: "# branch.oid (initial)\n# branch.head feature/initial-setup\n1 A. N... 000000 100644 100644 000000 039562 staged_initial.txt\n? untracked_readme.md\n",
                expected_branch: Some("feature/initial-setup"),
                expected_head_hash: None,
                expected_upstream: None,
                expected_ahead: 0,
                expected_behind: 0,
                expected_is_detached: false,
                expected_staged_count: 1,
                expected_unstaged_count: 1,
                expected_conflicted_count: 0,
            },
            StatusMatrixTestCase {
                description: "Diverged upstream tracking with +12 ahead and -4 behind",
                raw_input: "# branch.oid abcdef0123456789abcdef0123456789abcdef01\n# branch.head develop\n# branch.upstream origin/develop\n# branch.ab +12 -4\n",
                expected_branch: Some("develop"),
                expected_head_hash: Some("abcdef0123456789abcdef0123456789abcdef01"),
                expected_upstream: Some("origin/develop"),
                expected_ahead: 12,
                expected_behind: 4,
                expected_is_detached: false,
                expected_staged_count: 0,
                expected_unstaged_count: 0,
                expected_conflicted_count: 0,
            },
            StatusMatrixTestCase {
                description: "Active merge conflict with both modified and deleted",
                raw_input: "# branch.oid c0ffee0123456789abcdef0123456789abcdef01\n# branch.head bugfix/conflict\nu UU N... 100644 100644 100644 100644 039562 039562 039562 both_modified.rs\nu UD N... 100644 100644 000000 100644 039562 000000 039562 updated_by_us_deleted_by_them.rs\n",
                expected_branch: Some("bugfix/conflict"),
                expected_head_hash: Some("c0ffee0123456789abcdef0123456789abcdef01"),
                expected_upstream: None,
                expected_ahead: 0,
                expected_behind: 0,
                expected_is_detached: false,
                expected_staged_count: 0,
                expected_unstaged_count: 0,
                expected_conflicted_count: 2,
            },
            StatusMatrixTestCase {
                description: "Null-terminated (-z) output parsing",
                raw_input: "# branch.oid aabbccdd00112233445566778899aabbccddeeff\0# branch.head release/2.0\x001 .M N... 100644 100644 100644 039562 039562 file_modified.txt\0? untracked_zero.txt\0",
                expected_branch: Some("release/2.0"),
                expected_head_hash: Some("aabbccdd00112233445566778899aabbccddeeff"),
                expected_upstream: None,
                expected_ahead: 0,
                expected_behind: 0,
                expected_is_detached: false,
                expected_staged_count: 0,
                expected_unstaged_count: 2,
                expected_conflicted_count: 0,
            },
        ];

        for test_case in matrix_cases {
            let parsed = parse_porcelain_v2_status(test_case.raw_input);
            assert_eq!(
                parsed.branch_name.as_deref(),
                test_case.expected_branch,
                "Branch mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.head_commit_hash.as_deref(),
                test_case.expected_head_hash,
                "HEAD commit hash mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.upstream_branch_name.as_deref(),
                test_case.expected_upstream,
                "Upstream branch mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.ahead_count, test_case.expected_ahead,
                "Ahead count mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.behind_count, test_case.expected_behind,
                "Behind count mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.is_detached_head, test_case.expected_is_detached,
                "Detached head mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.staged_files.len(),
                test_case.expected_staged_count,
                "Staged count mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.unstaged_files.len(),
                test_case.expected_unstaged_count,
                "Unstaged count mismatch for scenario: {}",
                test_case.description
            );
            assert_eq!(
                parsed.conflicted_files.len(),
                test_case.expected_conflicted_count,
                "Conflicted count mismatch for scenario: {}",
                test_case.description
            );
        }
    }
}
