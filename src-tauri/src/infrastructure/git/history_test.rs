#[cfg(test)]
pub mod tests {
    use super::super::history::{
        parse_commit_changed_files, parse_commit_decorations, parse_delimited_git_log,
        COMMIT_FIELD_DELIMITER, COMMIT_RECORD_DELIMITER,
    };
    use crate::domain::{FileChangeStatus, GitReferenceType};

    #[test]
    fn test_parse_commit_decorations_with_head_branch_and_tag() {
        let raw = "HEAD -> master, origin/master, tag: v0.1.0, feature/new-branch";
        let decorations = parse_commit_decorations(raw);

        assert_eq!(decorations.len(), 5);

        // HEAD reference
        assert_eq!(decorations[0].name, "HEAD");
        assert_eq!(decorations[0].reference_type, GitReferenceType::Head);
        assert!(decorations[0].is_head);

        // Current branch master
        assert_eq!(decorations[1].name, "master");
        assert_eq!(decorations[1].reference_type, GitReferenceType::LocalBranch);
        assert!(decorations[1].is_head);

        // Remote branch origin/master
        assert_eq!(decorations[2].name, "origin/master");
        assert_eq!(decorations[2].reference_type, GitReferenceType::RemoteBranch);
        assert!(!decorations[2].is_head);

        // Tag v0.1.0
        assert_eq!(decorations[3].name, "v0.1.0");
        assert_eq!(decorations[3].reference_type, GitReferenceType::Tag);
        assert!(!decorations[3].is_head);

        // Other branch
        assert_eq!(decorations[4].name, "feature/new-branch");
        assert_eq!(decorations[4].reference_type, GitReferenceType::RemoteBranch);
        assert!(!decorations[4].is_head);
    }

    #[test]
    fn test_parse_delimited_git_log_with_merge_commit() {
        let raw_record = format!(
            "a1b2c3d4e5{del}parent1 parent2{del}Author Man{del}author@test.com{del}1700000000{del}Committer Man{del}committer@test.com{del}1700000000{del}Merge branch 'feature'{del}Detailed merge commit body{del}HEAD -> master, tag: v1.0.0{rec}",
            del = COMMIT_FIELD_DELIMITER,
            rec = COMMIT_RECORD_DELIMITER,
        );

        let commits = parse_delimited_git_log(&raw_record);
        assert_eq!(commits.len(), 1);

        let commit = &commits[0];
        assert_eq!(commit.hash, "a1b2c3d4e5");
        assert_eq!(commit.parent_hashes, vec!["parent1", "parent2"]);
        assert_eq!(commit.author_name, "Author Man");
        assert_eq!(commit.author_email, "author@test.com");
        assert_eq!(commit.author_timestamp, 1700000000);
        assert_eq!(commit.subject, "Merge branch 'feature'");
        assert_eq!(commit.body, "Detailed merge commit body");
        assert_eq!(commit.decorations.len(), 3);
    }

    #[test]
    fn test_parse_commit_changed_files_with_various_statuses() {
        let name_status = "M\tsrc/main.rs\nA\tsrc/domain/commit.rs\nD\told_file.txt\nR100\told_doc.md\tnew_doc.md";
        let numstat = "12\t4\tsrc/main.rs\n45\t0\tsrc/domain/commit.rs\n0\t20\told_file.txt\n0\t0\told_doc.md => new_doc.md";

        let changed_files = parse_commit_changed_files(name_status, numstat);
        assert_eq!(changed_files.len(), 4);

        // Modified
        assert_eq!(changed_files[0].path, "src/main.rs");
        assert_eq!(changed_files[0].status, FileChangeStatus::Modified);
        assert_eq!(changed_files[0].additions, 12);
        assert_eq!(changed_files[0].deletions, 4);

        // Added
        assert_eq!(changed_files[1].path, "src/domain/commit.rs");
        assert_eq!(changed_files[1].status, FileChangeStatus::Added);
        assert_eq!(changed_files[1].additions, 45);
        assert_eq!(changed_files[1].deletions, 0);

        // Deleted
        assert_eq!(changed_files[2].path, "old_file.txt");
        assert_eq!(changed_files[2].status, FileChangeStatus::Deleted);
        assert_eq!(changed_files[2].additions, 0);
        assert_eq!(changed_files[2].deletions, 20);

        // Renamed
        assert_eq!(changed_files[3].path, "new_doc.md");
        assert_eq!(changed_files[3].original_path, Some("old_doc.md".to_string()));
        assert_eq!(changed_files[3].status, FileChangeStatus::Renamed);
        assert_eq!(changed_files[3].additions, 0);
        assert_eq!(changed_files[3].deletions, 0);
    }
}
