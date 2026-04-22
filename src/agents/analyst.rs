//! Analyst Agent — repository understanding.

use crate::agents::base::{Agent, AgentContext};
use crate::analysis::{IssueDetector, RepoParser, RepoStructure, TechStackDetector};
use crate::core::types::*;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json;
use tracing::info;

#[derive(Debug, Deserialize)]
struct LlmAuditReport {
    architecture: String,
    tech_stack: Vec<String>,
    issues: Vec<LlmIssue>,
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct LlmIssue {
    severity: String,
    category: String,
    description: String,
    file_path: Option<String>,
    line: Option<u32>,
}

impl From<LlmIssue> for Issue {
    fn from(issue: LlmIssue) -> Self {
        let severity = match issue.severity.to_lowercase().as_str() {
            "critical" => IssueSeverity::Critical,
            "high" => IssueSeverity::High,
            "medium" => IssueSeverity::Medium,
            _ => IssueSeverity::Low,
        };
        Issue {
            severity,
            category: issue.category,
            description: issue.description,
            file_path: issue.file_path,
            line: issue.line,
        }
    }
}

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
        let tech_stack = if repo_path == "/dev/null" || tech_stack_is_empty(repo_path) {
            // Try LLM to detect tech stack from file contents
            let detected = tech_stack_from_files(repo_path);
            if detected.is_empty() { self.tech_detector.detect("") } else { detected }
        } else {
            self.tech_detector.detect(repo_path)
        };

        let mut issues = if repo_path == "/dev/null" { Vec::new() }
            else { self.issue_detector.detect(repo_path, &structure) };

        // Try LLM for deeper analysis
        if let Some(ref llm) = ctx.llm_client {
            if let Some(parsed) = self.try_llm_analysis(&structure, &tech_stack, repo_path, llm).await {
                if !parsed.issues.is_empty() || parsed.tech_stack.len() > tech_stack.len().max(1) {
                    return Ok(serde_json::json!({
                        "audit_report": AuditReport {
                            repo_url: String::new(),
                            architecture: parsed.architecture,
                            tech_stack: parsed.tech_stack,
                            issues: parsed.issues.into_iter().map(Into::into).collect(),
                            confidence: parsed.confidence,
                        },
                        "structure": serde_json::json!({
                            "directories": structure.directories,
                            "files": structure.files,
                            "file_types": structure.file_types,
                        }),
                    }));
                }
            }
        }

        if issues.is_empty() && repo_path != "/dev/null" {
            issues = self.issue_detector.detect(repo_path, &structure);
        }

        let audit_report = AuditReport {
            repo_url: String::new(),
            architecture: format!("Detected {} directories, {} files", structure.directories.len(), structure.files.len()),
            tech_stack,
            issues,
            confidence: 0.92,
        };

        Ok(serde_json::json!({
            "audit_report": audit_report,
            "structure": serde_json::json!({
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

impl AnalystAgent {
    async fn try_llm_analysis(
        &self,
        structure: &RepoStructure,
        tech_stack: &[String],
        repo_path: &str,
        llm: &crate::llm::LlmClient,
    ) -> Option<LlmAuditReport> {
        let system = self.prompt_template();
        let _tech_content = tech_stack_from_files(repo_path);
        let file_contents = read_key_files(repo_path, &structure.files);

        let prompt = format!(
            "Analyze this repository.\n\n\
             Structure: {} directories, {} files\n\
             Detected tech: {:?}\n\
             Key file contents:\n{}\n\n\
             Provide a detailed audit report with architecture summary, tech stack, and code quality issues.",
            structure.directories.len(),
            structure.files.len(),
            tech_stack,
            file_contents
        );

        let response = llm.generate(system, &prompt).await.ok()?;

        // Try to parse as JSON
        // The LLM might wrap JSON in markdown code blocks
        let json_str = if response.contains("```") {
            response.split("```").nth(1).unwrap_or(&response)
        } else {
            &response
        };

        serde_json::from_str::<LlmAuditReport>(json_str).ok()
    }
}

fn tech_stack_is_empty(repo_path: &str) -> bool {
    !std::path::Path::new(repo_path).join("Cargo.toml").exists()
        && !std::path::Path::new(repo_path).join("package.json").exists()
}

fn tech_stack_from_files(repo_path: &str) -> Vec<String> {
    let mut stack = Vec::new();
    let base = std::path::Path::new(repo_path);

    if base.join("Cargo.toml").exists() {
        stack.push("Rust".to_string());
    }
    if base.join("package.json").exists() {
        stack.push("Node.js".to_string());
    }
    if base.join("go.mod").exists() {
        stack.push("Go".to_string());
    }
    if base.join("pyproject.toml").exists() {
        stack.push("Python".to_string());
    }

    stack
}

fn read_key_files(repo_path: &str, _files: &[String]) -> String {
    let base = std::path::Path::new(repo_path);
    let key_files = ["Cargo.toml", "package.json", "go.mod", "pyproject.toml", "README.md", "Makefile"];
    let mut output = String::new();

    for key in &key_files {
        let path = base.join(key);
        if let Ok(content) = std::fs::read_to_string(&path) {
            output.push_str(&format!("\n--- {} ---\n{}\n", key, content.chars().take(1000).collect::<String>()));
        }
    }

    output
}
