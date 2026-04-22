//! IssueDetector — detects code quality and architecture issues from a real repo.

use crate::analysis::repo_parser::RepoStructure;
use crate::core::types::*;

/// Detector for code quality and architecture issues.
pub struct IssueDetector;

impl IssueDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect issues in a real repository directory.
    pub fn detect(&self, repo_path: &str, structure: &RepoStructure) -> Vec<Issue> {
        let mut issues = Vec::new();
        let base = std::path::Path::new(repo_path);

        // Check for documentation
        let doc_files = ["README.md", "CONTRIBUTING.md", "LICENSE", "CHANGELOG.md"];
        if !doc_files.iter().any(|f| base.join(f).exists()) {
            issues.push(Issue {
                severity: IssueSeverity::Medium,
                category: "Documentation".to_string(),
                description: "No documentation files found (README.md, LICENSE, etc.)".to_string(),
                file_path: None,
                line: None,
            });
        }

        // Check for test files
        let test_dirs = ["tests", "test", "__tests__", "spec"];
        if !test_dirs.iter().any(|d| base.join(d).is_dir()) {
            issues.push(Issue {
                severity: IssueSeverity::High,
                category: "Testing".to_string(),
                description: "No test directory found".to_string(),
                file_path: None,
                line: None,
            });
        }

        // Check for CI configuration
        if !base.join(".github").is_dir() && !base.join(".gitlab-ci.yml").exists() {
            issues.push(Issue {
                severity: IssueSeverity::Low,
                category: "CI/CD".to_string(),
                description: "No CI/CD configuration found".to_string(),
                file_path: None,
                line: None,
            });
        }

        // Check for large files (potential performance issue)
        for file in &structure.files {
            let full = base.join(file);
            if let Ok(metadata) = std::fs::metadata(&full) {
                if metadata.len() > 1024 * 1024 {
                    // > 1MB
                    issues.push(Issue {
                        severity: IssueSeverity::Medium,
                        category: "Performance".to_string(),
                        description: format!("Large file detected: {} ({} MB)", file, metadata.len() / 1024 / 1024),
                        file_path: Some(file.clone()),
                        line: None,
                    });
                }
            }
        }

        // Check for build system
        let build_files = ["Cargo.toml", "package.json", "go.mod", "pom.xml", "build.gradle", "Makefile"];
        if !build_files.iter().any(|f| base.join(f).exists()) {
            issues.push(Issue {
                severity: IssueSeverity::Critical,
                category: "Build System".to_string(),
                description: "No recognized build system configuration found".to_string(),
                file_path: None,
                line: None,
            });
        }

        // Check for error handling in Rust files
        if base.join("Cargo.toml").exists() {
            for file in &structure.files {
                if file.ends_with(".rs") {
                    let full = base.join(file);
                    if let Ok(content) = std::fs::read_to_string(&full) {
                        // Count unwrap() calls
                        let unwrap_count = content.matches(".unwrap()").count();
                        if unwrap_count > 5 {
                            issues.push(Issue {
                                severity: IssueSeverity::High,
                                category: "Error Handling".to_string(),
                                description: format!("{} has {} unwrap() calls — consider proper error handling", file, unwrap_count),
                                file_path: Some(file.clone()),
                                line: None,
                            });
                        }
                    }
                }
            }
        }

        issues
    }
}
