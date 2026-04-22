//! CodeRunner — sandboxed code execution.

use anyhow::Result;
use tokio::process::Command;

/// Sandboxed code execution engine.
///
/// Executes generated code in an isolated process — never evals in the main process.
pub struct CodeRunner;

impl CodeRunner {
    pub fn new() -> Self {
        Self
    }

    /// Execute a command with a timeout.
    ///
    /// On timeout: kills the child process and reaps it to prevent orphans.
    pub async fn execute(&self, command: &str, args: &[&str], timeout_secs: u64) -> Result<String> {
        let mut child = Command::new(command)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn {}: {}", command, e))?;

        match tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            child.wait(),
        ).await {
            Ok(Ok(status)) => {
                let output = child.wait_with_output().await?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                if status.success() {
                    Ok(stdout.to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(anyhow::anyhow!("Command failed: {}\n{}", stdout, stderr))
                }
            }
            _ => {
                // Timeout: kill_on_drop(true) handles the kill automatically
                // but we need to explicitly wait for reaping
                let _ = child.start_kill();
                let _ = child.wait().await;
                Err(anyhow::anyhow!(
                    "Command '{}' timed out after {}s",
                    command,
                    timeout_secs
                ))
            }
        }
    }

    /// Execute a Rust program (cargo run).
    pub async fn run_cargo(&self, _target_dir: &str, args: &[&str]) -> Result<String> {
        let args_str = args.iter().copied().collect::<Vec<_>>().join(" -- ");
        self.execute("cargo", &["run", "--release", "--", &args_str], 120).await
    }
}
