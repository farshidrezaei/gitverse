#[cfg(test)]
mod tests {
    use crate::infrastructure::git::diff::parse_unified_diff;
    use crate::domain::DiffLineType;

    #[test]
    fn test_parse_unified_diff_with_hunks() {
        let sample_diff = concat!(
            "diff --git a/src/main.rs b/src/main.rs\n",
            "index 123456..789abc 100644\n",
            "--- a/src/main.rs\n",
            "+++ b/src/main.rs\n",
            "@@ -1,5 +1,6 @@\n",
            " fn main() {\n",
            "-    println!(\"old\");\n",
            "+    println!(\"new\");\n",
            "+    println!(\"added\");\n",
            "     exit(0);\n",
            " }\n"
        );

        let diff = parse_unified_diff(sample_diff, "src/main.rs");

        assert_eq!(diff.new_path, "src/main.rs");
        assert_eq!(diff.old_path, Some("src/main.rs".to_string()));
        assert!(!diff.is_binary);
        assert_eq!(diff.total_additions, 2);
        assert_eq!(diff.total_deletions, 1);
        assert_eq!(diff.hunks.len(), 1);

        let hunk = &diff.hunks[0];
        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.old_lines, 5);
        assert_eq!(hunk.new_start, 1);
        assert_eq!(hunk.new_lines, 6);

        // Lines: Context, Deletion, Addition, Addition, Context, Context
        assert_eq!(hunk.lines.len(), 6);
        assert_eq!(hunk.lines[0].line_type, DiffLineType::Context);
        assert_eq!(hunk.lines[1].line_type, DiffLineType::Deletion);
        assert_eq!(hunk.lines[2].line_type, DiffLineType::Addition);
        assert_eq!(hunk.lines[3].line_type, DiffLineType::Addition);
    }

    #[test]
    fn test_parse_binary_diff() {
        let binary_diff = "Binary files a/logo.png and b/logo.png differ\n";
        let diff = parse_unified_diff(binary_diff, "logo.png");
        assert!(diff.is_binary);
        assert_eq!(diff.hunks.len(), 0);
    }
}
