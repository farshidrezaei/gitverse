use std::cmp::max;
use crate::domain::{CommitGraphHistoryResponse, CommitGraphNode, GitCommit, GraphEdge};

/// Service responsible for DAG construction and deterministic lane assignment.
pub struct CommitGraphEngine;

impl CommitGraphEngine {
    /// Builds a deterministic commit graph with lane assignments and routing edges
    /// from a topologically sorted sequence of Git commits.
    pub fn build_graph(commits: Vec<GitCommit>) -> CommitGraphHistoryResponse {
        let mut nodes = Vec::with_capacity(commits.len());
        let mut active_lanes: Vec<Option<String>> = Vec::new();
        let mut max_lane_index: usize = 0;

        for commit in commits {
            let is_head = commit.decorations.iter().any(|decoration| decoration.is_head);
            let is_merge = commit.parent_hashes.len() > 1;

            // Step 1: Determine the lane for this commit
            let current_lane = if let Some(existing_index) = active_lanes
                .iter()
                .position(|lane_target| lane_target.as_ref() == Some(&commit.hash))
            {
                // Found expected lane for this commit
                active_lanes[existing_index] = None;
                existing_index
            } else {
                // Not in active lanes: find first vacant lane or create a new one
                if let Some(vacant_index) = active_lanes.iter().position(|slot| slot.is_none()) {
                    vacant_index
                } else {
                    let new_index = active_lanes.len();
                    active_lanes.push(None);
                    new_index
                }
            };

            max_lane_index = max(max_lane_index, current_lane);

            // Step 2: Determine edges and forward parent lane expectations
            let mut edges = Vec::new();

            for (parent_index, parent_hash) in commit.parent_hashes.iter().enumerate() {
                let is_merge_edge = parent_index > 0;

                // Check if this parent is already expected in another lane
                if let Some(existing_parent_lane) = active_lanes
                    .iter()
                    .position(|lane_target| lane_target.as_ref() == Some(parent_hash))
                {
                    edges.push(GraphEdge {
                        parent_hash: parent_hash.clone(),
                        from_lane: current_lane,
                        to_lane: existing_parent_lane,
                        is_merge_edge,
                    });
                } else if parent_index == 0 && active_lanes.get(current_lane).map_or(true, |slot| slot.is_none()) {
                    // First parent continues down the same lane if the slot is free
                    if current_lane >= active_lanes.len() {
                        active_lanes.resize(current_lane + 1, None);
                    }
                    active_lanes[current_lane] = Some(parent_hash.clone());

                    edges.push(GraphEdge {
                        parent_hash: parent_hash.clone(),
                        from_lane: current_lane,
                        to_lane: current_lane,
                        is_merge_edge: false,
                    });
                } else {
                    // Subsequent parent or primary slot already taken: allocate next free lane
                    let allocated_lane = if let Some(vacant_index) = active_lanes.iter().position(|slot| slot.is_none()) {
                        vacant_index
                    } else {
                        let new_index = active_lanes.len();
                        active_lanes.push(None);
                        new_index
                    };

                    active_lanes[allocated_lane] = Some(parent_hash.clone());
                    max_lane_index = max(max_lane_index, allocated_lane);

                    edges.push(GraphEdge {
                        parent_hash: parent_hash.clone(),
                        from_lane: current_lane,
                        to_lane: allocated_lane,
                        is_merge_edge,
                    });
                }
            }

            nodes.push(CommitGraphNode {
                commit,
                lane: current_lane,
                edges,
                is_head,
                is_merge,
            });

            // Shrink trailing None lanes to prevent unneeded lane growth
            while active_lanes.last().map_or(false, |slot| slot.is_none()) {
                active_lanes.pop();
            }
        }

        let total_commits_count = nodes.len();

        CommitGraphHistoryResponse {
            nodes,
            max_lane_index,
            total_commits_count: Some(total_commits_count),
            has_more_commits: false,
        }
    }
}
