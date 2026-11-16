use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use crate::domain::{ApplicationError, GitBinaryInformation};

const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// Safe, argument-array-based runner for executing Git commands without shell interpolation.
#[derive(Debug, Clone)]
pub struct GitProcessExecutor {
    executable_path: PathBuf,
}

impl GitProcessExecutor {
    pub fn new(executable_path: PathBuf) -> Self {
        Self { executable_path }
    }

    /// Discovers and validates the native Git binary on the host machine.
    pub async fn discover_system_git(custom_path: Option<&str>) -> Result<GitBinaryInformation, ApplicationError> {
        let binary_to_test = if let Some(custom) = custom_path {
            if !custom.trim().is_empty() {
                PathBuf::from(custom.trim())
            } else {
                PathBuf::from("git")
            }
        } else {
            PathBuf::from("git")
        };

        let mut command = Command::new(&binary_to_test);
        command.arg("version");

        let output = tokio::time::timeout(Duration::from_secs(5), command.output())
            .await
            .map_err(|_| ApplicationError::TimeoutOrCancelled("Git binary discovery timed out after 5 seconds".into()))?
            .map_err(|error| ApplicationError::GitBinaryNotFound(format!("Failed to execute {:?}: {}", binary_to_test, error)))?;

        if !output.status.success() {
            let stderr_string = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(ApplicationError::GitExecutionFailed {
                exit_code: output.status.code().unwrap_or(-1),
                message: stderr_string,
            });
        }

        let raw_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // git version 2.43.0 -> parse or verify
        let has_porcelain_v2 = raw_version.contains("git version");

        let resolved_path = if binary_to_test == Path::new("git") {
            // Find absolute path via which/where if available or return "git"
            "git".to_string()
        } else {
            binary_to_test.to_string_lossy().to_string()
        };

        Ok(GitBinaryInformation {
            executable_path: resolved_path,
            version_string: raw_version,
            is_valid: true,
            has_porcelain_v2_support: has_porcelain_v2,
        })
    }

    /// Executes a Git command safely with structured argument arrays.
    pub async fn execute_git_command(
        &self,
        working_directory: Option<&Path>,
        arguments: &[&str],
    ) -> Result<String, ApplicationError> {
        let mut command = Command::new(&self.executable_path);

        if let Some(directory) = working_directory {
            command.current_dir(directory);
        }

        for argument in arguments {
            command.arg(argument);
        }

        // Standardize locale to C for predictable, machine-readable output
        command.env("LC_ALL", "C");

        let execution_result = tokio::time::timeout(DEFAULT_COMMAND_TIMEOUT, command.output())
            .await
            .map_err(|_| {
                ApplicationError::TimeoutOrCancelled(format!(
                    "Command 'git {:?}' timed out after {:?}",
                    arguments, DEFAULT_COMMAND_TIMEOUT
                ))
            })?
            .map_err(|error| {
                ApplicationError::FilesystemError(format!(
                    "Failed to spawn process {:?}: {}",
                    self.executable_path, error
                ))
            })?;

        if !execution_result.status.success() {
            let stderr = String::from_utf8_lossy(&execution_result.stderr).trim().to_string();
            let exit_code = execution_result.status.code().unwrap_or(-1);
            return Err(ApplicationError::GitExecutionFailed {
                exit_code,
                message: stderr,
            });
        }

        Ok(String::from_utf8_lossy(&execution_result.stdout).to_string())
    }
}
