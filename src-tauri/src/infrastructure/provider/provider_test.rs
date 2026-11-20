#[cfg(test)]
mod tests {
    use super::super::github::GitHubClient;
    use super::super::gitlab::GitLabClient;
    use crate::domain::{ProviderKind, PullRequestState};

    #[test]
    fn test_github_client_instance_url_resolution() {
        let default_client = GitHubClient::new(None).expect("Default client");
        assert!(format!("{:?}", default_client).contains("https://api.github.com"));

        let standard_client = GitHubClient::new(Some("https://github.com")).expect("Standard client");
        assert!(format!("{:?}", standard_client).contains("https://api.github.com"));

        let enterprise_client = GitHubClient::new(Some("https://github.mycompany.internal/")).expect("Enterprise client");
        assert!(format!("{:?}", enterprise_client).contains("https://github.mycompany.internal/api/v3"));
    }

    #[test]
    fn test_gitlab_client_instance_url_resolution() {
        let default_client = GitLabClient::new(None).expect("Default client");
        assert!(format!("{:?}", default_client).contains("https://gitlab.com/api/v4"));

        let custom_client = GitLabClient::new(Some("https://gitlab.example.org/")).expect("Custom client");
        assert!(format!("{:?}", custom_client).contains("https://gitlab.example.org/api/v4"));
    }

    #[test]
    fn test_provider_kind_display_and_serialization() {
        assert_eq!(ProviderKind::GitHub.to_string(), "GitHub");
        assert_eq!(ProviderKind::GitLab.to_string(), "GitLab");

        let serialized_github = serde_json::to_string(&ProviderKind::GitHub).expect("Serialize");
        assert_eq!(serialized_github, "\"github\"");

        let deserialized_gitlab: ProviderKind = serde_json::from_str("\"gitlab\"").expect("Deserialize");
        assert_eq!(deserialized_gitlab, ProviderKind::GitLab);
    }

    #[test]
    fn test_pull_request_state_serialization() {
        let states = vec![
            (PullRequestState::Open, "\"open\""),
            (PullRequestState::Closed, "\"closed\""),
            (PullRequestState::Merged, "\"merged\""),
        ];

        for (state, expected_json) in states {
            let json = serde_json::to_string(&state).expect("Serialize");
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_pipeline_summary_and_check_runs_serialization() {
        use crate::domain::{CiCheckRun, PipelineState, PipelineSummary};

        let summary = PipelineSummary {
            state: PipelineState::Success,
            web_url: Some("https://github.com/org/repo/actions/runs/123".into()),
            commit_sha: Some("abcdef123456".into()),
            total_checks_count: 5,
            successful_checks_count: 5,
            failed_checks_count: 0,
        };

        let json = serde_json::to_string(&summary).expect("Serialize PipelineSummary");
        assert!(json.contains("\"success\""));
        assert!(json.contains("\"total_checks_count\":5"));

        let check = CiCheckRun {
            id: "github:999".into(),
            name: "test-suite".into(),
            status: "completed".into(),
            conclusion: Some("success".into()),
            html_url: Some("https://github.com/org/repo/runs/999".into()),
            started_at_timestamp: Some(1700000000),
            completed_at_timestamp: Some(1700000100),
        };

        let check_json = serde_json::to_string(&check).expect("Serialize CiCheckRun");
        assert!(check_json.contains("\"test-suite\""));
        assert!(check_json.contains("\"github:999\""));
    }

    #[test]
    fn test_pull_request_review_and_merge_serialization() {
        use crate::domain::{
            PullRequestChangedFile, PullRequestMergeMethod, PullRequestMergeResult,
            PullRequestReviewAction, RemoteReviewComment,
        };

        let file = PullRequestChangedFile {
            file_path: "src/main.rs".into(),
            previous_file_path: None,
            status: "modified".into(),
            additions_count: 12,
            deletions_count: 3,
            changes_count: 15,
            patch: Some("@@ -1,3 +1,12 @@".into()),
        };
        let file_json = serde_json::to_string(&file).expect("Serialize file");
        assert!(file_json.contains("\"src/main.rs\""));
        assert!(file_json.contains("\"additions_count\":12"));

        assert_eq!(
            serde_json::to_string(&PullRequestReviewAction::Approve).expect("action"),
            "\"APPROVE\""
        );
        assert_eq!(
            serde_json::to_string(&PullRequestReviewAction::RequestChanges).expect("action"),
            "\"REQUEST_CHANGES\""
        );
        assert_eq!(
            serde_json::to_string(&PullRequestReviewAction::Comment).expect("action"),
            "\"COMMENT\""
        );

        assert_eq!(
            serde_json::to_string(&PullRequestMergeMethod::Squash).expect("method"),
            "\"squash\""
        );
        assert_eq!(
            serde_json::to_string(&PullRequestMergeMethod::Rebase).expect("method"),
            "\"rebase\""
        );
        assert_eq!(
            serde_json::to_string(&PullRequestMergeMethod::Merge).expect("method"),
            "\"merge\""
        );

        let merge_result = PullRequestMergeResult {
            is_merged: true,
            message: "Pull Request successfully merged".into(),
            commit_sha: Some("1234567890abcdef".into()),
        };
        let merge_json = serde_json::to_string(&merge_result).expect("Serialize merge result");
        assert!(merge_json.contains("\"is_merged\":true"));
        assert!(merge_json.contains("\"1234567890abcdef\""));

        let comment = RemoteReviewComment {
            id: "github:101".into(),
            author_username: "alice".into(),
            author_avatar_url: Some("https://example.com/avatar.png".into()),
            body: "Looks good to me!".into(),
            path: None,
            line_number: None,
            created_at_timestamp: 1700000000,
            updated_at_timestamp: 1700000000,
        };
        let comment_json = serde_json::to_string(&comment).expect("Serialize comment");
        assert!(comment_json.contains("\"alice\""));
        assert!(comment_json.contains("\"Looks good to me!\""));
    }
}
