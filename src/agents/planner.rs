//! Planner Agent — roadmap generation and task prioritization.

use crate::agents::base::{Agent, AgentContext};
use crate::core::types::*;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct LlmPlan {
    roadmap: Vec<LlmTask>,
    total_estimated_cost: f64,
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct LlmTask {
    title: String,
    description: String,
    priority: String,
    effort: String,
    category: String,
    file_paths: Vec<String>,
    depends_on: Vec<String>,
}

impl From<LlmTask> for RoadmapTask {
    fn from(task: LlmTask) -> Self {
        let priority = match task.priority.to_lowercase().as_str() {
            "p0" | "critical" | "must" => Priority::P0,
            "p1" | "high" | "should" => Priority::P1,
            "p2" | "medium" | "nice" => Priority::P2,
            _ => Priority::P3,
        };
        let effort = match task.effort.to_lowercase().as_str() {
            "small" | "s" | "<1h" => TaskEffort::Small,
            "medium" | "m" | "1-4h" => TaskEffort::Medium,
            "large" | "l" | "4-8h" => TaskEffort::Large,
            "xlarge" | "xl" | ">8h" => TaskEffort::XLarge,
            _ => TaskEffort::Medium,
        };
        let depends_on: Vec<Uuid> = task.depends_on
            .into_iter()
            .filter_map(|id| Uuid::parse_str(&id).ok())
            .collect();

        RoadmapTask {
            id: Uuid::new_v4(),
            title: task.title,
            description: task.description,
            priority,
            effort,
            category: task.category,
            depends_on,
            file_paths: task.file_paths,
        }
    }
}

pub struct PlannerAgent;

impl PlannerAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Agent for PlannerAgent {
    async fn run(&self, input: serde_json::Value, ctx: &AgentContext) -> Result<serde_json::Value> {
        info!("[PLANNER] Generating roadmap (iteration {})", ctx.iteration);

        // Extract data from analyst output
        let audit_report: Option<AuditReport> = serde_json::from_value(input["audit_report"].clone()).ok();
        let tech_stack: Vec<String> = input["tech_stack"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let mut roadmap = Vec::new();
        let mut total_cost = 0.0;
        let mut plan_confidence = 0.88;

        // Try LLM for roadmap generation
        if let Some(ref llm) = ctx.llm_client {
            if let Some(llm_plan) = self.try_llm_plan(&input, &tech_stack, llm).await {
                let llm_confidence = llm_plan.confidence;
                roadmap = llm_plan.roadmap.into_iter().map(Into::into).collect();
                total_cost = llm_plan.total_estimated_cost;
                plan_confidence = llm_confidence;
            }
        }

        // Fallback: generate roadmap from detected issues
        if roadmap.is_empty() {
            let issues: Vec<Issue> = audit_report.as_ref().map(|ar| ar.issues.clone()).unwrap_or_default();
            roadmap = self.generate_from_issues(&issues, &tech_stack);
            total_cost = roadmap.iter().map(|t| match t.effort {
                TaskEffort::Small => 1.0,
                TaskEffort::Medium => 3.0,
                TaskEffort::Large => 6.0,
                TaskEffort::XLarge => 12.0,
            }).sum();
        }

        roadmap.sort_by_key(|t| match t.priority {
            Priority::P0 => 0,
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
        });

        let plan = Plan {
            roadmap,
            total_estimated_cost: total_cost,
            confidence: plan_confidence,
        };

        Ok(serde_json::json!({
            "plan": plan,
            "tech_stack": tech_stack,
            "confidence": plan.confidence,
        }))
    }

    fn name(&self) -> &str {
        "planner"
    }

    fn prompt_template(&self) -> &'static str {
        include_str!("../../prompts/planner.md")
    }

    fn mcp_tools(&self) -> Vec<String> {
        vec!["chimera_confident".to_string(), "chimera_gate".to_string()]
    }

    fn min_confidence(&self) -> f64 {
        0.85
    }
}

impl PlannerAgent {
    async fn try_llm_plan(&self, input: &serde_json::Value, tech_stack: &[String], llm: &crate::llm::LlmClient) -> Option<LlmPlan> {
        let system = self.prompt_template();
        let audit: Option<AuditReport> = serde_json::from_value(input["audit_report"].clone()).ok();
        let issues_str = audit.as_ref().map(|ar| ar.issues.iter().enumerate()
            .map(|(i, iss)| format!("  {}. [{}] {}: {}", i + 1, iss.severity, iss.category, iss.description))
            .collect::<Vec<_>>()
            .join("\n")).unwrap_or_default();

        let prompt = format!(
            "Generate a prioritized roadmap for these issues:\n\n{}\n\nTech stack: {:?}.",
            issues_str, tech_stack
        );

        let response = llm.generate(system, &prompt).await.ok()?;
        let json_str = response.trim();
        serde_json::from_str(json_str).ok()
    }

    fn generate_from_issues(&self, issues: &[Issue], _tech_stack: &[String]) -> Vec<RoadmapTask> {
        let mut roadmap = Vec::new();
        for (i, issue) in issues.iter().enumerate() {
            let priority = match issue.severity {
                IssueSeverity::Critical => Priority::P0,
                IssueSeverity::High => Priority::P1,
                IssueSeverity::Medium => Priority::P2,
                IssueSeverity::Low => Priority::P3,
            };
            roadmap.push(RoadmapTask {
                id: Uuid::new_v4(),
                title: format!("Fix: {}", issue.description),
                description: format!("Address {} issue: {}", issue.severity, issue.description),
                priority,
                effort: match i % 4 {
                    0 => TaskEffort::Small,
                    1 => TaskEffort::Medium,
                    2 => TaskEffort::Large,
                    _ => TaskEffort::XLarge,
                },
                category: issue.category.clone(),
                depends_on: Vec::new(),
                file_paths: issue.file_path.clone().map(|p| vec![p]).unwrap_or_default(),
            });
        }
        roadmap
    }
}
