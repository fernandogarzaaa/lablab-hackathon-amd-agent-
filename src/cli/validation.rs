//! CLI argument and configuration validation.

use anyhow::Result;
use std::path::Path;

/// Validates and analyzes CLI arguments before execution.
/// Returns (overall_result, warnings_for_non_fatal_issues).
pub fn validate_analyze_command(
    repo_url: &str,
    provider: &str,
    config_dir: &str,
    max_iterations: u32,
    min_confidence: f64,
) -> (Result<()>, Vec<String>) {
    let mut warnings = Vec::new();

    // Validate URL is non-empty and has a scheme
    if repo_url.is_empty() {
        return (
            Err(anyhow::anyhow!("Repository URL cannot be empty")),
            warnings,
        );
    }
    if !repo_url.contains("://") {
        warnings.push(format!(
            "URL may be missing scheme (http:// or https://): {repo_url}. Adding https://."
        ));
    }

    // If it looks like a GitHub URL, verify owner/repo are present
    if repo_url.contains("github.com") {
        let path_segments: Vec<&str> = repo_url
            .trim_matches(|c| c == '/' || c == ':' || c == '.')
            .split('/')
            .collect();
        let last_two: Vec<&str> = path_segments
            .iter()
            .rev()
            .take(2)
            .rev()
            .copied()
            .collect();
        if last_two.len() < 2 || last_two[0].is_empty() || last_two[1].is_empty() {
            return (
                Err(anyhow::anyhow!(
                    "GitHub URL must include owner and repo: {repo_url}"
                )),
                warnings,
            );
        }
    }

    // Validate provider type
    match provider.to_lowercase().as_str() {
        "anthropic" | "openai" | "ollama" | "openai-compatible" | "compatible" => {}
        other => {
            return (
                Err(anyhow::anyhow!(
                    "Unknown provider: {other}. Valid values: anthropic, openai, ollama, openai-compatible"
                )),
                warnings,
            );
        }
    }

    // Validate config directory exists
    let config_path = Path::new(config_dir);
    if !config_path.exists() {
        warnings.push(format!(
            "Config directory does not exist: {config_dir}. Using built-in defaults."
        ));
    } else if !config_path.is_dir() {
        warnings.push(format!(
            "Config path is not a directory: {config_dir}. Using built-in defaults."
        ));
    }

    // Validate LoopConfig bounds
    if max_iterations == 0 {
        return (
            Err(anyhow::anyhow!("max-iterations must be at least 1")),
            warnings,
        );
    }
    if max_iterations > 100 {
        warnings.push(
            "max-iterations > 100 may result in long execution times".to_string(),
        );
    }
    if !(0.0..=1.0).contains(&min_confidence) {
        return (
            Err(anyhow::anyhow!(
                "min-confidence must be between 0.0 and 1.0, got {min_confidence}"
            )),
            warnings,
        );
    }

    (Ok(()), warnings)
}
