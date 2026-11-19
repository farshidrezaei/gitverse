pub mod github;
pub mod gitlab;
#[cfg(test)]
mod provider_test;

pub use github::GitHubClient;
pub use gitlab::GitLabClient;
