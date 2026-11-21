use super::file_history::parse_file_history_output;

#[test]
fn test_parse_file_history_output_multiple_revisions() {
    let output = "a2d87c0\x1fFarshid Rezaei\x1ffarshid@test.com\x1f1700000000\x1ffeat: update file\nM\tsrc/domain/mod.rs\n4ba8dfc\x1fAlice\x1falice@test.com\x1f1699999000\x1ffeat: initial file\nA\tsrc/domain/mod.rs\n";

    let timeline = parse_file_history_output(output, "src/domain/mod.rs");

    assert_eq!(timeline.total_revisions, 2);
    assert_eq!(timeline.file_path, "src/domain/mod.rs");

    assert_eq!(timeline.revisions[0].commit_hash, "a2d87c0");
    assert_eq!(timeline.revisions[0].author_name, "Farshid Rezaei");
    assert_eq!(timeline.revisions[0].commit_summary, "feat: update file");
    assert_eq!(timeline.revisions[0].change_status, "M");

    assert_eq!(timeline.revisions[1].commit_hash, "4ba8dfc");
    assert_eq!(timeline.revisions[1].author_name, "Alice");
    assert_eq!(timeline.revisions[1].change_status, "A");
}

#[test]
fn test_parse_file_history_output_empty() {
    let timeline = parse_file_history_output("", "readme.md");
    assert_eq!(timeline.total_revisions, 0);
    assert!(timeline.revisions.is_empty());
}
