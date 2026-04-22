//! Trajectory compression and ability extraction (hermes-agent inspired).
//!
//! After each loop iteration, extracts patterns that led to successful outcomes
//! and stores them as reusable abilities for future runs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub id: String,
    pub category: AbilityCategory,
    pub trigger_conditions: Vec<String>,
    pub action_template: String,
    pub success_count: u32,
    pub last_used: Option<String>,
}

impl Ability {
    pub fn new(category: AbilityCategory, trigger_conditions: Vec<String>, action_template: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            category,
            trigger_conditions,
            action_template,
            success_count: 0,
            last_used: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AbilityCategory {
    RepoAnalysis,
    TaskPlanning,
    CodeGeneration,
    WorkflowValidation,
    CrossAgentCommunication,
}

impl std::fmt::Display for AbilityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbilityCategory::RepoAnalysis => write!(f, "repo_analysis"),
            AbilityCategory::TaskPlanning => write!(f, "task_planning"),
            AbilityCategory::CodeGeneration => write!(f, "code_generation"),
            AbilityCategory::WorkflowValidation => write!(f, "workflow_validation"),
            AbilityCategory::CrossAgentCommunication => write!(f, "cross_agent_communication"),
        }
    }
}

/// Compress trajectory data into a summary.
pub fn compress_trajectory(input: &str, iteration: u32) -> String {
    // Extract key patterns from the trajectory
    let lines: Vec<&str> = input.lines().collect();
    let key_lines: Vec<&str> = lines.iter()
        .filter(|l| l.to_lowercase().contains("confidence") || l.to_lowercase().contains("verified"))
        .copied()
        .collect();

    format!(
        "Iteration {} summary ({} relevant events): {} key patterns extracted",
        iteration,
        key_lines.len(),
        key_lines.len().min(5)
    )
}

/// Extract abilities from a successful iteration.
pub fn extract_abilities(input: &str, _category: AbilityCategory) -> Vec<Ability> {
    let mut abilities = Vec::new();

    // Analyze input for patterns
    let has_tech_stack = input.to_lowercase().contains("tech_stack");
    let has_issues = input.to_lowercase().contains("issue") || input.to_lowercase().contains("problem");
    let has_rust = input.to_lowercase().contains("rust") || input.to_lowercase().contains("cargo");

    if has_tech_stack {
        abilities.push(Ability::new(
            AbilityCategory::RepoAnalysis,
            vec!["tech_stack".to_string(), "detected".to_string()],
            "Analyze repository tech stack by examining package files and build configuration".to_string(),
        ));
    }

    if has_issues {
        abilities.push(Ability::new(
            AbilityCategory::RepoAnalysis,
            vec!["issue".to_string(), "detected".to_string()],
            "Detect code issues by analyzing patterns and structure".to_string(),
        ));
    }

    if has_rust {
        abilities.push(Ability::new(
            AbilityCategory::CodeGeneration,
            vec!["rust".to_string(), "cargo".to_string()],
            "Generate Rust code following Cargo conventions and idiomatic patterns".to_string(),
        ));
    }

    abilities
}
