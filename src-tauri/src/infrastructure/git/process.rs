use crate::domain::{ApplicationError, GitBinaryInformation};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

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
    pub async fn discover_system_git(
        custom_path: Option<&str>,
    ) -> Result<GitBinaryInformation, ApplicationError> {
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
            .map_err(|_| {
                ApplicationError::TimeoutOrCancelled(
                    "Git binary discovery timed out after 5 seconds".into(),
                )
            })?
            .map_err(|error| {
                ApplicationError::GitBinaryNotFound(format!(
                    "Failed to execute {:?}: {}",
                    binary_to_test, error
                ))
            })?;

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

        let debug_command = format!("git {}", arguments.join(" "));
        crate::infrastructure::diagnostics::append_diagnostic_log("debug", &debug_command);
        tracing::debug!(
            "Git execution: {}",
            crate::infrastructure::diagnostics::sanitize_sensitive_string(&debug_command)
        );

        // Standardize locale to C for predictable, machine-readable output
        command.env("LC_ALL", "C");

        let execution_result = tokio::time::timeout(DEFAULT_COMMAND_TIMEOUT, command.output())
            .await
            .map_err(|_| {
                let message = format!(
                    "Command 'git {:?}' timed out after {:?}",
                    arguments, DEFAULT_COMMAND_TIMEOUT
                );
                crate::infrastructure::diagnostics::append_diagnostic_log("error", &message);
                ApplicationError::TimeoutOrCancelled(message)
            })?
            .map_err(|error| {
                let message = format!(
                    "Failed to spawn process {:?}: {}",
                    self.executable_path, error
                );
                crate::infrastructure::diagnostics::append_diagnostic_log("error", &message);
                ApplicationError::FilesystemError(message)
            })?;

        if !execution_result.status.success() {
            let stderr = String::from_utf8_lossy(&execution_result.stderr)
                .trim()
                .to_string();
            let exit_code = execution_result.status.code().unwrap_or(-1);
            let message = format!("Git command failed (exit code {}): {}", exit_code, stderr);
            crate::infrastructure::diagnostics::append_diagnostic_log("warn", &message);
            return Err(ApplicationError::GitExecutionFailed {
                exit_code,
                message: stderr,
            });
        }

        Ok(String::from_utf8_lossy(&execution_result.stdout).to_string())
    }

    /// Executes a Git command and returns exit code, stdout, and stderr without erroring on non-zero exit code.
    pub async fn execute_git_command_with_status(
        &self,
        working_directory: Option<&Path>,
        arguments: &[&str],
    ) -> Result<(i32, String, String), ApplicationError> {
        let mut command = Command::new(&self.executable_path);

        if let Some(directory) = working_directory {
            command.current_dir(directory);
        }

        for argument in arguments {
            command.arg(argument);
        }

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

        let exit_code = execution_result.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&execution_result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&execution_result.stderr).to_string();

        Ok((exit_code, stdout, stderr))
    }

    /// Executes a Git command safely with standard input piped into the child process.
    pub async fn execute_git_command_with_stdin(
        &self,
        working_directory: Option<&Path>,
        arguments: &[&str],
        standard_input_bytes: &[u8],
    ) -> Result<String, ApplicationError> {
        let mut command = Command::new(&self.executable_path);

        if let Some(directory) = working_directory {
            command.current_dir(directory);
        }

        for argument in arguments {
            command.arg(argument);
        }

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.env("LC_ALL", "C");

        let mut child = command.spawn().map_err(|error| {
            ApplicationError::FilesystemError(format!(
                "Failed to spawn process {:?}: {}",
                self.executable_path, error
            ))
        })?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(standard_input_bytes).await;
            let _ = stdin.flush().await;
            drop(stdin);
        }

        let execution_result =
            tokio::time::timeout(DEFAULT_COMMAND_TIMEOUT, child.wait_with_output())
                .await
                .map_err(|_| {
                    ApplicationError::TimeoutOrCancelled(format!(
                        "Command 'git {:?}' timed out after {:?}",
                        arguments, DEFAULT_COMMAND_TIMEOUT
                    ))
                })?
                .map_err(|error| {
                    ApplicationError::FilesystemError(format!(
                        "Failed to read child process output: {}",
                        error
                    ))
                })?;

        if !execution_result.status.success() {
            let stderr = String::from_utf8_lossy(&execution_result.stderr)
                .trim()
                .to_string();
            let exit_code = execution_result.status.code().unwrap_or(-1);
            let message = format!(
                "Git command with stdin failed (exit code {}): {}",
                exit_code, stderr
            );
            crate::infrastructure::diagnostics::append_diagnostic_log("warn", &message);
            return Err(ApplicationError::GitExecutionFailed {
                exit_code,
                message: stderr,
            });
        }

        Ok(String::from_utf8_lossy(&execution_result.stdout).to_string())
    }

    /// Executes a Git command and returns the raw stdout bytes (for binary files, images, etc.).
    pub async fn execute_git_command_raw(
        &self,
        working_directory: Option<&Path>,
        arguments: &[&str],
    ) -> Result<Vec<u8>, ApplicationError> {
        let mut command = Command::new(&self.executable_path);

        if let Some(directory) = working_directory {
            command.current_dir(directory);
        }

        for argument in arguments {
            command.arg(argument);
        }

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
            let stderr = String::from_utf8_lossy(&execution_result.stderr)
                .trim()
                .to_string();
            let exit_code = execution_result.status.code().unwrap_or(-1);
            let message = format!(
                "Git command with stdin failed (exit code {}): {}",
                exit_code, stderr
            );
            crate::infrastructure::diagnostics::append_diagnostic_log("warn", &message);
            return Err(ApplicationError::GitExecutionFailed {
                exit_code,
                message: stderr,
            });
        }

        Ok(execution_result.stdout)
    }
}
