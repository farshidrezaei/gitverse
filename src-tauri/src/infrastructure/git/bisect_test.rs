use super::bisect::parse_bisect_step_output;

#[test]
fn test_parse_bisect_step_output_intermediate_step() {
    let output = "Bisecting: 3 revisions left to test after this (roughly 2 steps)\n[c2381b329e528c4ee96f237ef319e27be8313e54] commit 2\n";
    let state = parse_bisect_step_output(
        output,
        true,
        Some("c2381b329e528c4ee96f237ef319e27be8313e54".to_string()),
        Some("commit 2".to_string()),
        "bisect log test".to_string(),
    );

    assert!(state.is_active);
    assert_eq!(state.revisions_remaining, Some(3));
    assert_eq!(state.estimated_steps_remaining, Some(2));
    assert_eq!(state.culprit_commit_hash, None);
    assert_eq!(state.current_commit_hash.as_deref(), Some("c2381b329e528c4ee96f237ef319e27be8313e54"));
    assert_eq!(state.current_commit_summary.as_deref(), Some("commit 2"));
}

#[test]
fn test_parse_bisect_step_output_culprit_found() {
    let output = "c86466e99565eefe9891747df843197c13c43e57 is the first 'bad' commit\ncommit c86466e99565eefe9891747df843197c13c43e57\nAuthor: Tester <test@test.com>\n";
    let state = parse_bisect_step_output(
        output,
        true,
        Some("c86466e99565eefe9891747df843197c13c43e57".to_string()),
        Some("commit 3".to_string()),
        "bisect log test".to_string(),
    );

    assert!(state.is_active);
    assert_eq!(state.culprit_commit_hash.as_deref(), Some("c86466e99565eefe9891747df843197c13c43e57"));
}

#[test]
fn test_parse_bisect_step_output_no_steps_left() {
    let output = "Bisecting: 0 revisions left to test after this (roughly 0 steps)\n[abcdef1234567890abcdef1234567890abcdef12] final check\n";
    let state = parse_bisect_step_output(
        output,
        true,
        Some("abcdef1234567890abcdef1234567890abcdef12".to_string()),
        Some("final check".to_string()),
        "".to_string(),
    );

    assert_eq!(state.revisions_remaining, Some(0));
    assert_eq!(state.estimated_steps_remaining, Some(0));
}
