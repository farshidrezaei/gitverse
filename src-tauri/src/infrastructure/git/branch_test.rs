#[cfg(test)]
mod tests {
    use super::super::branch::{
        parse_local_branches, parse_remote_branches, parse_remotes, parse_tags, REF_FIELD_DELIMITER,
    };

    #[test]
    fn test_parse_local_branches_with_ahead_behind_and_head() {
        let raw = format!(
            "main{del}*{del}origin/main{del}ahead 3, behind 1{del}8f2a1b9\nfeature/graph{del} {del}origin/feature/graph{del}ahead 2{del}1a9f3d4\nfix/bug{del} {del}{del}{del}3c8e5f1\n",
            del = REF_FIELD_DELIMITER
        );

        let branches = parse_local_branches(&raw);
        assert_eq!(branches.len(), 3);

        // Branch 1: main (HEAD)
        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_head);
        assert_eq!(branches[0].upstream_name.as_deref(), Some("origin/main"));
        assert_eq!(branches[0].ahead_count, 3);
        assert_eq!(branches[0].behind_count, 1);
        assert_eq!(branches[0].head_commit_hash.as_deref(), Some("8f2a1b9"));

        // Branch 2: feature/graph
        assert_eq!(branches[1].name, "feature/graph");
        assert!(!branches[1].is_head);
        assert_eq!(branches[1].upstream_name.as_deref(), Some("origin/feature/graph"));
        assert_eq!(branches[1].ahead_count, 2);
        assert_eq!(branches[1].behind_count, 0);

        // Branch 3: fix/bug (no upstream)
        assert_eq!(branches[2].name, "fix/bug");
        assert!(!branches[2].is_head);
        assert_eq!(branches[2].upstream_name, None);
        assert_eq!(branches[2].ahead_count, 0);
        assert_eq!(branches[2].behind_count, 0);
    }

    #[test]
    fn test_parse_remote_branches_filters_head() {
        let raw = format!(
            "origin/HEAD{del}8f2a1b9\norigin{del}8f2a1b9{del}refs/remotes/origin/main\norigin/main{del}8f2a1b9{del}\nupstream/feature{del}4d7e2c0{del}\n",
            del = REF_FIELD_DELIMITER
        );

        let remotes = parse_remote_branches(&raw);
        assert_eq!(remotes.len(), 2);
        assert_eq!(remotes[0].name, "origin/main");
        assert_eq!(remotes[0].remote_name, "origin");
        assert_eq!(remotes[1].name, "upstream/feature");
        assert_eq!(remotes[1].remote_name, "upstream");
    }

    #[test]
    fn test_parse_tags() {
        let raw = format!("v0.1.0{del}8f2a1b9\nv0.0.9{del}09b9058\n", del = REF_FIELD_DELIMITER);
        let tags = parse_tags(&raw);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "v0.1.0");
        assert_eq!(tags[0].commit_hash, "8f2a1b9");
        assert_eq!(tags[1].name, "v0.0.9");
        assert_eq!(tags[1].commit_hash, "09b9058");
    }

    #[test]
    fn test_parse_remotes() {
        let raw = "origin\thttps://github.com/user/gitverse.git (fetch)\norigin\thttps://github.com/user/gitverse.git (push)\nupstream\thttps://github.com/upstream/gitverse.git (fetch)\n";
        let remotes = parse_remotes(raw);
        assert_eq!(remotes.len(), 2);

        let origin = &remotes[0];
        assert_eq!(origin.name, "origin");
        assert_eq!(origin.fetch_url.as_deref(), Some("https://github.com/user/gitverse.git"));
        assert_eq!(origin.push_url.as_deref(), Some("https://github.com/user/gitverse.git"));

        let upstream = &remotes[1];
        assert_eq!(upstream.name, "upstream");
        assert_eq!(upstream.fetch_url.as_deref(), Some("https://github.com/upstream/gitverse.git"));
        assert_eq!(upstream.push_url, None);
    }

    #[test]
    fn test_validate_branch_name_rules() {
        use super::super::branch::validate_branch_name;

        // Valid branch names
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("feature/login").is_ok());
        assert!(validate_branch_name("bugfix/issue-123_part2").is_ok());
        assert!(validate_branch_name("v1.0.0-rc1").is_ok());

        // Invalid branch names
        assert!(validate_branch_name("").is_err());
        assert!(validate_branch_name("   ").is_err());
        assert!(validate_branch_name("feature name with spaces").is_err());
        assert!(validate_branch_name("/leading-slash").is_err());
        assert!(validate_branch_name("trailing-slash/").is_err());
        assert!(validate_branch_name("-leading-dash").is_err());
        assert!(validate_branch_name("trailing-dash-").is_err());
        assert!(validate_branch_name(".leading-dot").is_err());
        assert!(validate_branch_name("double//slash").is_err());
        assert!(validate_branch_name("double..dot").is_err());
        assert!(validate_branch_name("feature.lock").is_err());
        assert!(validate_branch_name("feature@{upstream}").is_err());
        assert!(validate_branch_name("invalid:colon").is_err());
        assert!(validate_branch_name("invalid~tilde").is_err());
        assert!(validate_branch_name("invalid^caret").is_err());
    }
}
