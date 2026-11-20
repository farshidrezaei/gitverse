use super::insights::{parse_shortlog_contributors, parse_shortstat, parse_weekday_activity};

#[test]
fn test_parse_shortlog_contributors() {
    let output = "    75\tFarshid Rezaei <farshid.net1@gmail.com>\n    25\tAlice Developer <alice@example.com>\n";
    let (contributors, total_contributors) = parse_shortlog_contributors(output, 100);

    assert_eq!(total_contributors, 2);
    assert_eq!(contributors[0].name, "Farshid Rezaei");
    assert_eq!(contributors[0].email, "farshid.net1@gmail.com");
    assert_eq!(contributors[0].commit_count, 75);
    assert!((contributors[0].percentage - 75.0).abs() < 0.01);

    assert_eq!(contributors[1].name, "Alice Developer");
    assert_eq!(contributors[1].commit_count, 25);
    assert!((contributors[1].percentage - 25.0).abs() < 0.01);
}

#[test]
fn test_parse_shortstat() {
    let output = " 290 files changed, 62988 insertions(+), 1205 deletions(-)\n";
    let (added, deleted) = parse_shortstat(output);
    assert_eq!(added, 62988);
    assert_eq!(deleted, 1205);

    let output_add_only = " 5 files changed, 42 insertions(+)\n";
    let (added_only, deleted_only) = parse_shortstat(output_add_only);
    assert_eq!(added_only, 42);
    assert_eq!(deleted_only, 0);
}

#[test]
fn test_parse_weekday_activity() {
    // 0 = 1970-01-01 (Thursday = 4)
    // 86400 = 1970-01-02 (Friday = 5)
    let timestamps = "0\n86400\n";
    let activity = parse_weekday_activity(timestamps);

    assert_eq!(activity.len(), 7);
    assert_eq!(activity[4].count, 1); // Thursday
    assert_eq!(activity[5].count, 1); // Friday
    assert_eq!(activity[0].count, 0); // Sunday
}
