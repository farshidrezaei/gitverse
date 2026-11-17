use serde::{Deserialize, Serialize};

/// Type of reference decorating a commit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GitReferenceType {
    LocalBranch,
    RemoteBranch,
    Tag,
    Head,
}

/// A reference decoration attached to a commit (e.g. branch, tag, HEAD).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitCommitDecoration {
    pub name: String,
    pub reference_type: GitReferenceType,
    pub is_head: bool,
}

/// Raw parsed commit entity from Git history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitCommit {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    pub author_timestamp: i64,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_timestamp: i64,
    pub subject: String,
    pub body: String,
    pub decorations: Vec<GitCommitDecoration>,
}

/// Routing edge between a commit and one of its parents in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    pub parent_hash: String,
    pub from_lane: usize,
    pub to_lane: usize,
    pub is_merge_edge: bool,
}

/// A node in the commit graph DAG with assigned lane and routing edges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitGraphNode {
    pub commit: GitCommit,
    pub lane: usize,
    pub edges: Vec<GraphEdge>,
    pub is_head: bool,
    pub is_merge: bool,
}

/// Paginated or segmented commit graph history response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CommitGraphHistoryResponse {
    pub nodes: Vec<CommitGraphNode>,
    pub max_lane_index: usize,
    pub total_commits_count: Option<usize>,
    pub has_more_commits: bool,
}
