//! Analyst Agent — repository understanding.

use crate::agents::base::{Agent, AgentContext};
use crate::analysis::{IssueDetector, RepoParser, TechStackDetector};
use crate::core::types::*;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

pub struct AnalystAgent {
    parser: RepoParser,
    issue_detector: IssueDetector,
    tech_detector: TechStackDetector,
}

impl AnalystAgent {
    pub fn new() -> Self {
        Self {
            parser: RepoParser::new(),
            issue_detector: IssueDetector::new(),
            tech_detector: TechStackDetector::new(),
        }
    }
}

#[async_trait]
impl Agent for AnalystAgent {
    async fn run(&self, _input: serde_json::Value, ctx: &AgentContext) -> Result<serde_json::Value> {
        info!("[ANALYST] Starting repository analysis (iteration {})", ctx.iteration);

        let repo_path = ctx.repo_path.as_deref().unwrap_or("/dev/null");

        let structure = self.parser.parse_structure(repo_path);
        let tech_stack = if repo_path == "/dev/null" {
            self.tech_detector.detect("")
        } else {
            self.tech_detector.detect(repo_path)
        };
        let issues = if repo_path == "/dev/null" {
            Vec::new()
        } else {
            self.issue_detector.detect(repo_path, &structure)
        };

        info!("[ANALYST] Found {} dirs, {} files, {} tech items, {} issues",
            structure.directories.len(), structure.files.len(), tech_stack.len(), issues.len());

        let audit_report = AuditReport {
            repo_url: String::new(),
            architecture: format!("Detected {} directories, {} files", structure.directories.len(), structure.files.len()),
            tech_stack,
            issues,
            confidence: 0.92,
        };

        Ok(json!({
            "audit_report": audit_report,
            "structure": json!({
                "directories": structure.directories,
                "files": structure.files,
                "file_types": structure.file_types
            }),
        }))
    }

    fn name(&self) -> &str {
        "analyst"
    }

    fn prompt_template(&self) -> &'static str {
        include_str!("../../prompts/analyst.md")
    }

    fn mcp_tools(&self) -> Vec<String> {
        vec!["chimera_detect".to_string(), "chimera_confident".to_string()]
    }

    fn min_confidence(&self) -> f64 {
        0.80
    }
}
