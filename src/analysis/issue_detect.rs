//! IssueDetector — detects code quality and architecture issues.

use crate::analysis::repo_parser::RepoStructure;
use crate::core::types::*;

/// Detector for code quality and architecture issues.
pub struct IssueDetector;

impl IssueDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect issues in a repo structure (simulated).
    pub fn detect(&self, structure: &RepoStructure) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Check for missing components
        if !structure.files.iter().any(|f| f.ends_with(".md")) {
            issues.push(Issue {
                severity: IssueSeverity::Medium,
                category: "Documentation".to_string(),
                description: "No markdown documentation found in repository".to_string(),
                file_path: None,
                line: None,
            });
        }

        if !structure.files.iter().any(|f| f.starts_with("test") || f.starts_with("tests")) {
            issues.push(Issue {
                severity: IssueSeverity::High,
                category: "Testing".to_string(),
                description: "No test files found in repository".to_string(),
                file_path: None,
                line: None,
            });
        }

        // Check for potential architecture issues
        if structure.files.len() > 100 {
            issues.push(Issue {
                severity: IssueSeverity::Medium,
                category: "Architecture".to_string(),
                description: "Large repository — consider modularizing into sub-projects".to_string(),
                file_path: None,
                line: None,
            });
        }

        // Check for missing build configuration
        if !structure.files.iter().any(|f| f == "Cargo.toml" || f == "package.json" || f == "pom.xml") {
            issues.push(Issue {
                severity: IssueSeverity::Critical,
                category: "Build System".to_string(),
                description: "No recognized build system configuration found".to_string(),
                file_path: None,
                line: None,
            });
        }

        issues
    }
}
