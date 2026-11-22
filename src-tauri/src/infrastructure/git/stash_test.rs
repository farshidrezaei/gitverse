#[cfg(test)]
mod tests {
    use super::super::stash::{STASH_FIELD_DELIMITER, parse_stash_list};

    #[test]
    fn test_parse_stash_list_with_multiple_entries() {
        let raw_output = format!(
            "stash@{{0}}{delimiter}WIP on master: 1234567 initial commit{delimiter}a1b2c3d4e5f6{delimiter}1716300000\n\
             stash@{{1}}{delimiter}On feature: WIP experimental UI{delimiter}b2c3d4e5f6a1{delimiter}1716290000\n",
            delimiter = STASH_FIELD_DELIMITER
        );

        let stashes = parse_stash_list(&raw_output);
        assert_eq!(stashes.len(), 2);

        // Stash 0
        assert_eq!(stashes[0].index, 0);
        assert_eq!(stashes[0].stash_identifier, "stash@{0}");
        assert_eq!(stashes[0].message, "WIP on master: 1234567 initial commit");
        assert_eq!(stashes[0].commit_hash, "a1b2c3d4e5f6");
        assert_eq!(stashes[0].timestamp_seconds, 1716300000);

        // Stash 1
        assert_eq!(stashes[1].index, 1);
        assert_eq!(stashes[1].stash_identifier, "stash@{1}");
        assert_eq!(stashes[1].message, "On feature: WIP experimental UI");
        assert_eq!(stashes[1].commit_hash, "b2c3d4e5f6a1");
        assert_eq!(stashes[1].timestamp_seconds, 1716290000);
    }

    #[test]
    fn test_parse_stash_list_empty_input() {
        let stashes = parse_stash_list("");
        assert!(stashes.is_empty());
    }

    #[test]
    fn test_parse_stash_list_malformed_lines_skipped() {
        let raw_output = "malformed line\nanother bad line\n";
        let stashes = parse_stash_list(raw_output);
        assert!(stashes.is_empty());
    }
}
