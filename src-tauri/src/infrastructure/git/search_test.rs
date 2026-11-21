use super::search::parse_git_grep_output;

#[test]
fn test_parse_git_grep_output_standard() {
    let output = "src-tauri/src/domain/mod.rs:443:pub struct FileRevisionEntry {\nsrc-tauri/src/main.rs:12:use crate::domain::FileRevisionEntry;\n";
    let result = parse_git_grep_output(output, "FileRevisionEntry", None);

    assert_eq!(result.total_matches, 2);
    assert_eq!(result.file_count, 2);
    assert_eq!(result.query, "FileRevisionEntry");

    assert_eq!(result.matches[0].file_path, "src-tauri/src/domain/mod.rs");
    assert_eq!(result.matches[0].line_number, 443);
    assert_eq!(result.matches[0].line_content, "pub struct FileRevisionEntry {");

    assert_eq!(result.matches[1].file_path, "src-tauri/src/main.rs");
    assert_eq!(result.matches[1].line_number, 12);
}

#[test]
fn test_parse_git_grep_output_with_reference_prefix() {
    let output = "HEAD:src/lib.rs:25:const DEFAULT_TIMEOUT: u64 = 30;\nHEAD:src/lib.rs:50:println!(\"done\");\n";
    let result = parse_git_grep_output(output, "timeout", Some("HEAD"));

    assert_eq!(result.total_matches, 2);
    assert_eq!(result.file_count, 1);
    assert_eq!(result.matches[0].file_path, "src/lib.rs");
    assert_eq!(result.matches[0].line_number, 25);
}

#[test]
fn test_parse_git_grep_output_empty() {
    let result = parse_git_grep_output("", "nonexistent", None);
    assert_eq!(result.total_matches, 0);
    assert_eq!(result.file_count, 0);
    assert!(result.matches.is_empty());
}
