//! Planner Agent — roadmap generation and task prioritization.

use crate::agents::base::{Agent, AgentContext};
use crate::core::types::*;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;
use uuid::Uuid;

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

        let issues: Vec<Issue> = input["issues"].as_array()
            .map(|a| a.iter().map(|v| serde_json::from_value(v.clone()).unwrap()).collect())
            .unwrap_or_default();

        let tech_stack: Vec<String> = input["tech_stack"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

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

        roadmap.sort_by_key(|t| match t.priority {
            Priority::P0 => 0,
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
        });

        let total_cost = roadmap.iter().map(|t| match t.effort {
            TaskEffort::Small => 1.0,
            TaskEffort::Medium => 3.0,
            TaskEffort::Large => 6.0,
            TaskEffort::XLarge => 12.0,
        }).sum::<f64>();

        let plan = Plan {
            roadmap,
            total_estimated_cost: total_cost,
            confidence: 0.88,
        };

        Ok(json!({
            "plan": plan,
            "tech_stack": tech_stack,
            "confidence": 0.88,
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
