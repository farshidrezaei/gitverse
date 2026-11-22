#[cfg(test)]
mod tests {
    use super::super::graph::CommitGraphEngine;
    use crate::domain::GitCommit;
    use proptest::prelude::*;

    fn create_test_commit(hash: &str, parent_hashes: Vec<&str>) -> GitCommit {
        GitCommit {
            hash: hash.to_string(),
            parent_hashes: parent_hashes.into_iter().map(String::from).collect(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            author_timestamp: 1700000000,
            committer_name: "Test Committer".to_string(),
            committer_email: "committer@example.com".to_string(),
            committer_timestamp: 1700000000,
            subject: format!("Commit {}", hash),
            body: String::new(),
            decorations: Vec::new(),
        }
    }

    #[test]
    fn test_linear_commit_history_stays_in_lane_zero() {
        let commits = vec![
            create_test_commit("c3", vec!["c2"]),
            create_test_commit("c2", vec!["c1"]),
            create_test_commit("c1", vec![]),
        ];

        let result = CommitGraphEngine::build_graph(commits);

        assert_eq!(result.nodes.len(), 3);
        assert_eq!(result.max_lane_index, 0);

        for node in &result.nodes {
            assert_eq!(node.lane, 0);
        }

        // c3 -> c2
        assert_eq!(result.nodes[0].edges.len(), 1);
        assert_eq!(result.nodes[0].edges[0].from_lane, 0);
        assert_eq!(result.nodes[0].edges[0].to_lane, 0);

        // c2 -> c1
        assert_eq!(result.nodes[1].edges.len(), 1);
        assert_eq!(result.nodes[1].edges[0].from_lane, 0);
        assert_eq!(result.nodes[1].edges[0].to_lane, 0);

        // c1 (root)
        assert_eq!(result.nodes[2].edges.len(), 0);
    }

    #[test]
    fn test_divergent_branch_and_merge_lane_assignment() {
        // Topology:
        // m2 (merge m1, f1)
        // f1 (parent r1)
        // m1 (parent r1)
        // r1 (root)
        let commits = vec![
            create_test_commit("m2", vec!["m1", "f1"]),
            create_test_commit("f1", vec!["r1"]),
            create_test_commit("m1", vec!["r1"]),
            create_test_commit("r1", vec![]),
        ];

        let result = CommitGraphEngine::build_graph(commits);
        assert_eq!(result.nodes.len(), 4);
        assert!(result.max_lane_index >= 1);

        let m2 = &result.nodes[0];
        assert_eq!(m2.lane, 0);
        assert_eq!(m2.edges.len(), 2);
        assert_eq!(m2.edges[0].from_lane, 0);
        assert_eq!(m2.edges[0].to_lane, 0); // primary parent m1 continues in lane 0
        assert_eq!(m2.edges[1].from_lane, 0);
        assert_eq!(m2.edges[1].to_lane, 1); // secondary parent f1 routes to lane 1
        assert!(m2.edges[1].is_merge_edge);

        let f1 = &result.nodes[1];
        assert_eq!(f1.lane, 1);

        let m1 = &result.nodes[2];
        assert_eq!(m1.lane, 0);

        let r1 = &result.nodes[3];
        assert_eq!(r1.edges.len(), 0);
    }

    proptest! {
        #[test]
        fn prop_test_graph_engine_determinism(chain_length in 1usize..20) {
            let mut commits = Vec::new();
            for i in (1..=chain_length).rev() {
                let parent = if i > 1 { vec![format!("c{}", i - 1)] } else { vec![] };
                commits.push(GitCommit {
                    hash: format!("c{}", i),
                    parent_hashes: parent,
                    author_name: "Author".to_string(),
                    author_email: "author@test.com".to_string(),
                    author_timestamp: 1700000000 + i as i64,
                    committer_name: "Committer".to_string(),
                    committer_email: "committer@test.com".to_string(),
                    committer_timestamp: 1700000000 + i as i64,
                    subject: format!("Subject {}", i),
                    body: String::new(),
                    decorations: Vec::new(),
                });
            }

            let result1 = CommitGraphEngine::build_graph(commits.clone());
            let result2 = CommitGraphEngine::build_graph(commits);

            prop_assert_eq!(&result1, &result2);
            prop_assert_eq!(result1.max_lane_index, 0);
            prop_assert_eq!(result1.nodes.len(), chain_length);
        }
    }

    #[test]
    fn test_benchmark_graph_construction_2000_commits() {
        use std::time::Instant;

        let total_commits = 2000;
        let mut commits = Vec::with_capacity(total_commits);

        for index in (1..=total_commits).rev() {
            let mut parents = Vec::new();
            if index > 1 {
                parents.push(format!("hash_{}", index - 1));
            }
            if index % 5 == 0 && index > 10 {
                parents.push(format!("hash_{}", index - 8));
            }

            commits.push(create_test_commit(
                &format!("hash_{}", index),
                parents.iter().map(|s| s.as_str()).collect(),
            ));
        }

        let start_time = Instant::now();
        let graph = CommitGraphEngine::build_graph(commits);
        let duration = start_time.elapsed();

        assert_eq!(graph.nodes.len(), total_commits);
        assert!(graph.max_lane_index >= 1);
        assert!(
            duration.as_millis() < 250,
            "Graph layout for 2,000 commits must complete within 250ms, took {:?}",
            duration
        );
    }

    #[test]
    fn test_benchmark_graph_construction_10000_commits() {
        use std::time::Instant;

        let total_commits = 10000;
        let mut commits = Vec::with_capacity(total_commits);

        for index in (1..=total_commits).rev() {
            let mut parents = Vec::new();
            if index > 1 {
                parents.push(format!("hash_{}", index - 1));
            }
            if index % 7 == 0 && index > 15 {
                parents.push(format!("hash_{}", index - 12));
            }

            commits.push(create_test_commit(
                &format!("hash_{}", index),
                parents.iter().map(|s| s.as_str()).collect(),
            ));
        }

        let start_time = Instant::now();
        let graph = CommitGraphEngine::build_graph(commits);
        let duration = start_time.elapsed();

        assert_eq!(graph.nodes.len(), total_commits);
        assert!(graph.max_lane_index >= 1);
        assert!(
            duration.as_millis() < 1200,
            "Graph layout for 10,000 commits must complete within 1200ms, took {:?}",
            duration
        );
    }
}
