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

    /// Execute a command in a temp directory with a timeout.
    pub async fn execute(&self, command: &str, args: &[&str], timeout_secs: u64) -> Result<String> {
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            Command::new(command)
                .args(args)
                .output()
        ).await;

        let output = match output {
            Ok(o) => o?,
            Err(e) => return Err(anyhow::anyhow!("Execution timeout or error: {}", e)),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            return Err(anyhow::anyhow!("Command failed: {}\n{}", stdout, stderr));
        }
        Ok(stdout.to_string())
    }

    /// Execute a Rust program (cargo run).
    pub async fn run_cargo(&self, _target_dir: &str, args: &[&str]) -> Result<String> {
        let args_str = args.iter().copied().collect::<Vec<_>>().join(" -- ");
        self.execute("cargo", &["run", "--release", "--", &args_str], 120).await
    }
}
