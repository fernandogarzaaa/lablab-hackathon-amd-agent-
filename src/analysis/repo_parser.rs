//! RepoParser — parses directory structure and identifies tech stack.

use serde::{Deserialize, Serialize};

/// Analyzed directory structure of a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStructure {
    pub directories: Vec<String>,
    pub files: Vec<String>,
    pub file_types: std::collections::HashMap<String, u32>,
}

/// Repository parser — analyzes file structure and tech stack.
pub struct RepoParser;

impl RepoParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a repository structure (simulated for now; will use git2 in production).
    pub fn parse_structure(&self) -> RepoStructure {
        // Simulated analysis — in production, this walks the actual cloned repo
        let directories = vec![
            "src".to_string(),
            "tests".to_string(),
            "benches".to_string(),
            "config".to_string(),
            "prompts".to_string(),
            "src/agents".to_string(),
            "src/core".to_string(),
            "src/analysis".to_string(),
            "src/middleware".to_string(),
            "src/memory".to_string(),
            "src/execution".to_string(),
            "src/cli".to_string(),
            "src/llm".to_string(),
        ];

        let files = vec![
            "Cargo.toml".to_string(),
            "main.rs".to_string(),
            "lib.rs".to_string(),
            "src/core/types.rs".to_string(),
            "src/core/orchestrator.rs".to_string(),
            "src/core/state.rs".to_string(),
            "src/agents/base.rs".to_string(),
            "src/agents/analyst.rs".to_string(),
            "src/agents/planner.rs".to_string(),
            "src/agents/builder.rs".to_string(),
            "src/agents/tester.rs".to_string(),
            "src/agents/critic.rs".to_string(),
            "src/middleware/chimera.rs".to_string(),
            "src/middleware/gate.rs".to_string(),
            "src/middleware/types.rs".to_string(),
            "src/memory/store.rs".to_string(),
            "src/memory/semantic.rs".to_string(),
            "src/memory/search.rs".to_string(),
            "src/memory/compression.rs".to_string(),
            "src/analysis/repo_parser.rs".to_string(),
            "src/analysis/dep_mapper.rs".to_string(),
            "src/analysis/issue_detect.rs".to_string(),
            "src/cli/mod.rs".to_string(),
            "src/cli/analyze.rs".to_string(),
            "src/llm/client.rs".to_string(),
            "src/llm/routing.rs".to_string(),
        ];

        let mut file_types = std::collections::HashMap::new();
        for file in &files {
            let ext = file.split('.').last().unwrap_or("rs");
            *file_types.entry(ext.to_string()).or_insert(0u32) += 1;
        }

        RepoStructure {
            directories,
            files,
            file_types,
        }
    }
}

/// Detect technology stack from file structure and package files.
pub struct TechStackDetector;

impl TechStackDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect tech stack from a repo structure.
    pub fn detect(&self) -> Vec<String> {
        // Simulated detection — in production, this checks actual package files
        vec![
            "Rust".to_string(),
            "tokio".to_string(),
            "clap".to_string(),
            "serde".to_string(),
            "rusqlite".to_string(),
            "tracing".to_string(),
            "axum".to_string(),
        ]
    }
}
