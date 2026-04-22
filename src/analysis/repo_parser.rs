//! RepoParser — parses directory structure and identifies tech stack from a real repo.

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

    /// Parse a real repository directory tree.
    pub fn parse_structure(&self, repo_path: &str) -> RepoStructure {
        let mut directories = Vec::new();
        let mut files = Vec::new();
        let mut file_types = std::collections::HashMap::new();

        fn walk(path: &str, dirs: &mut Vec<String>, files: &mut Vec<String>, types: &mut std::collections::HashMap<String, u32>) {
            let mut entries = match std::fs::read_dir(path) {
                Ok(e) => e,
                Err(_) => return,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let full_path = path.to_string_lossy().to_string();

                if path.is_dir() {
                    // Skip common non-source directories
                    let skip = ["node_modules", ".git", "target", "vendor", ".cargo", "dist", "build", "__pycache__"];
                    if !skip.iter().any(|s| name.starts_with(s)) {
                        let rel = RepoParser::relative_path(path.parent().unwrap(), &path);
                        dirs.push(rel);
                        walk(&full_path, dirs, files, types);
                    }
                } else {
                    let rel = RepoParser::relative_path(path.parent().unwrap(), &path);
                    files.push(rel);
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        *types.entry(ext.to_string()).or_insert(0) += 1;
                    } else {
                        *types.entry("toml".to_string()).or_insert(0) += 1; // Cargo.toml, etc.
                    }
                }
            }
        }

        walk(repo_path, &mut directories, &mut files, &mut file_types);

        // Add root-level "files" as a pseudo-directory
        directories.insert(0, ".".to_string());

        RepoStructure {
            directories,
            files,
            file_types,
        }
    }

    fn relative_path(from: &std::path::Path, to: &std::path::Path) -> String {
        let from_str = from.to_string_lossy();
        let to_str = to.to_string_lossy();
        if to_str.starts_with(&*from_str) {
            let suffix = &to_str[from_str.len()..];
            suffix.trim_start_matches('/').to_string()
        } else {
            to_str.to_string()
        }
    }
}

/// Detect technology stack from file structure and package files in a real repo.
pub struct TechStackDetector;

impl TechStackDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect tech stack from a repository directory.
    pub fn detect(&self, repo_path: &str) -> Vec<String> {
        let mut stack = Vec::new();
        let base = std::path::Path::new(repo_path);

        // Check for Rust
        if base.join("Cargo.toml").exists() {
            stack.push("Rust".to_string());
            if let Ok(content) = std::fs::read_to_string(base.join("Cargo.toml")) {
                for dep in ["tokio", "serde", "clap", "axum", "tokio-util", "tracing", "futures", "bytes", "url", "uuid", "sha2", "rand", "async-trait", "git2", "reqwest", "rusqlite", "toml", "chrono", "anyhow", "thiserror", "colored"] {
                    if content.contains(dep) {
                        stack.push(dep.to_string());
                    }
                }
            }
        }

        // Check for Node.js/TypeScript
        if base.join("package.json").exists() {
            stack.push("Node.js".to_string());
            if let Ok(content) = std::fs::read_to_string(base.join("package.json")) {
                for dep in ["react", "vue", "next", "express", "typescript", "tailwindcss", "webpack", "vite", "jest"] {
                    if content.contains(dep) {
                        stack.push(dep.to_string());
                    }
                }
            }
        }

        // Check for Python
        if base.join("pyproject.toml").exists() || base.join("setup.py").exists() {
            stack.push("Python".to_string());
        }
        if base.join("go.mod").exists() {
            stack.push("Go".to_string());
        }
        if base.join("pom.xml").exists() || base.join("build.gradle").exists() {
            stack.push("Java".to_string());
        }
        if base.join("go.mod").exists() {
            if let Ok(content) = std::fs::read_to_string(base.join("go.mod")) {
                for dep in ["gin", "echo", "fiber", "gorm", "zap", "slog"] {
                    if content.contains(dep) {
                        stack.push(dep.to_string());
                    }
                }
            }
        }

        stack
    }
}
