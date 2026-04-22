//! Core types shared across the system.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Agent Types ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentName {
    Analyst,
    Planner,
    Builder,
    Tester,
    Critic,
}

impl std::fmt::Display for AgentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentName::Analyst => write!(f, "ANALYST"),
            AgentName::Planner => write!(f, "PLANNER"),
            AgentName::Builder => write!(f, "BUILDER"),
            AgentName::Tester => write!(f, "TESTER"),
            AgentName::Critic => write!(f, "CRITIC"),
        }
    }
}

// ─── AgentMessage ──────────────────────────────────────────────

/// Typed message passing between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: AgentName,
    pub to: AgentName,
    pub iteration: u32,
    pub confidence: f64,
    pub summary: String,
    pub details: serde_json::Value,
    pub artifacts: Vec<Artifact>,
    pub suggestions: Vec<Suggestion>,
}

impl AgentMessage {
    pub fn new(from: AgentName, to: AgentName, iteration: u32, confidence: f64, summary: String, details: serde_json::Value) -> Self {
        Self {
            from,
            to,
            iteration,
            confidence,
            summary,
            details,
            artifacts: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_artifact(mut self, artifact: Artifact) -> Self {
        self.artifacts.push(artifact);
        self
    }

    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub content: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub agent: AgentName,
    pub issue: String,
    pub priority: Priority,
}

// ─── Artifacts ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub repo_url: String,
    pub architecture: String,
    pub tech_stack: Vec<String>,
    pub issues: Vec<Issue>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub category: String,
    pub description: String,
    pub file_path: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Critical => write!(f, "CRITICAL"),
            IssueSeverity::High => write!(f, "HIGH"),
            IssueSeverity::Medium => write!(f, "MEDIUM"),
            IssueSeverity::Low => write!(f, "LOW"),
        }
    }
}

// ─── Plan / Roadmap ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub roadmap: Vec<RoadmapTask>,
    pub total_estimated_cost: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapTask {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub effort: TaskEffort,
    pub category: String,
    pub depends_on: Vec<Uuid>,
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    P0, // Must fix
    P1, // Should fix
    P2, // Nice to have
    P3, // Low priority
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::P0 => write!(f, "P0"),
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
            Priority::P3 => write!(f, "P3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskEffort {
    Small,   // < 1 hour
    Medium,  // 1-4 hours
    Large,   // 4-8 hours
    XLarge,  // > 8 hours
}

// ─── Code Changes ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOutput {
    pub changes: Vec<FileChange>,
    pub summary: String,
    pub tests_run: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub operation: ChangeOperation,
    pub content: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeOperation {
    Create,
    Update,
    Delete,
}

// ─── Test Results ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutput {
    pub workflows_tested: Vec<Workflow>,
    pub results: Vec<TestResult>,
    pub usability_issues: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub steps: Vec<String>,
    pub expected_behavior: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub workflow: String,
    pub passed: bool,
    pub issues: Vec<String>,
    pub confidence: f64,
}

// ─── Critic Verdict ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueOutput {
    pub verdict: Verdict,
    pub rationale: String,
    pub required_fixes: Vec<RequiredFix>,
    pub confidence: f64,
    pub session_trace: Vec<SessionTraceEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Verdict {
    Approve,
    Continue,
    Abort,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Approve => write!(f, "APPROVE"),
            Verdict::Continue => write!(f, "CONTINUE"),
            Verdict::Abort => write!(f, "ABORT"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFix {
    pub agent: AgentName,
    pub issue: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTraceEntry {
    pub timestamp: String,
    pub agent: AgentName,
    pub action: String,
    pub confidence: f64,
}

// ─── Final Output ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalOutput {
    pub repo_url: String,
    pub audit_report: AuditReport,
    pub plan: Plan,
    pub build_output: Option<BuildOutput>,
    pub test_output: Option<TestOutput>,
    pub final_verdict: Verdict,
    pub total_iterations: u32,
    pub abilities_extracted: Vec<String>,
    pub session_trace: Vec<SessionTraceEntry>,
}

// ─── Confidence Types ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High(f64),      // >= 0.95
    Medium(f64),    // >= 0.80
    Low(f64),       // >= 0.50
    Uncertain(f64), // < 0.50
}

impl ConfidenceLevel {
    pub fn from(score: f64) -> Self {
        if score >= 0.95 {
            Self::High(score)
        } else if score >= 0.80 {
            Self::Medium(score)
        } else if score >= 0.50 {
            Self::Low(score)
        } else {
            Self::Uncertain(score)
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            ConfidenceLevel::High(s) => *s,
            ConfidenceLevel::Medium(s) => *s,
            ConfidenceLevel::Low(s) => *s,
            ConfidenceLevel::Uncertain(s) => *s,
        }
    }

    pub fn is_above(&self, threshold: f64) -> bool {
        self.score() >= threshold
    }
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfidenceLevel::High(s) => write!(f, "HIGH ({:.2})", s),
            ConfidenceLevel::Medium(s) => write!(f, "MEDIUM ({:.2})", s),
            ConfidenceLevel::Low(s) => write!(f, "LOW ({:.2})", s),
            ConfidenceLevel::Uncertain(s) => write!(f, "UNCERTAIN ({:.2})", s),
        }
    }
}

// ─── Loop Config ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    pub max_iterations: u32,
    pub min_confidence: f64,
    pub abort_threshold: f64,
    pub parallel_execution: bool,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            min_confidence: 0.85,
            abort_threshold: 0.30,
            parallel_execution: true,
        }
    }
}

impl std::fmt::Display for LoopConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoopConfig {{ max_iter: {}, min_conf: {:.2}, abort_thresh: {:.2}, parallel: {} }}",
            self.max_iterations, self.min_confidence, self.abort_threshold, self.parallel_execution
        )
    }
}

// ─── Config Types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: AgentName,
    pub role: String,
    pub goal: String,
    pub mcp_tools: Vec<String>,
    pub min_confidence: f64,
    pub max_iterations: u32,
}
