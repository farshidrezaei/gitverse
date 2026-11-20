use super::submodule::parse_submodule_status;

#[test]
fn test_parse_submodule_status_clean_initialized() {
    let output = " 0afa6c2685f487180f6b074d106da34149f51f38 libs/sub (heads/master)\n";
    let submodules = parse_submodule_status(output);

    assert_eq!(submodules.len(), 1);
    let sub = &submodules[0];
    assert_eq!(sub.path, "libs/sub");
    assert_eq!(sub.name, "sub");
    assert_eq!(sub.current_commit_hash, "0afa6c2685f487180f6b074d106da34149f51f38");
    assert!(sub.is_initialized);
    assert!(!sub.has_revision_divergence);
    assert!(!sub.has_uncommitted_changes);
    assert_eq!(sub.branch.as_deref(), Some("heads/master"));
}

#[test]
fn test_parse_submodule_status_uninitialized_and_diverged() {
    let output = "-1111111111111111111111111111111111111111 vendor/liba\n+2222222222222222222222222222222222222222 vendor/libb (v1.2.0)\n";
    let submodules = parse_submodule_status(output);

    assert_eq!(submodules.len(), 2);
    assert!(!submodules[0].is_initialized);
    assert_eq!(submodules[0].name, "liba");

    assert!(submodules[1].is_initialized);
    assert!(submodules[1].has_revision_divergence);
    assert_eq!(submodules[1].branch.as_deref(), Some("v1.2.0"));
}

#[test]
fn test_parse_submodule_status_empty() {
    let submodules = parse_submodule_status("");
    assert!(submodules.is_empty());
}
