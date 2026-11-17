use serde::{Deserialize, Serialize};

/// Summary of a local Git branch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchSummary {
    pub name: String,
    pub is_head: bool,
    pub upstream_name: Option<String>,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub head_commit_hash: Option<String>,
}

/// Summary of a remote Git tracking branch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteBranchSummary {
    pub name: String,
    pub remote_name: String,
    pub head_commit_hash: Option<String>,
}

/// Summary of a Git tag reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagSummary {
    pub name: String,
    pub commit_hash: String,
}

/// Summary of a configured Git remote repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteSummary {
    pub name: String,
    pub fetch_url: Option<String>,
    pub push_url: Option<String>,
}

/// Complete references container for an open repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RepositoryReferences {
    pub branches: Vec<BranchSummary>,
    pub remote_branches: Vec<RemoteBranchSummary>,
    pub tags: Vec<TagSummary>,
    pub remotes: Vec<RemoteSummary>,
}
