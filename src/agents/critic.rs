//! Critic Agent — cross-agent output review and final gate.

use crate::agents::base::{Agent, AgentContext};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

pub struct CriticAgent;

impl CriticAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Agent for CriticAgent {
    async fn run(&self, input: serde_json::Value, ctx: &AgentContext) -> Result<serde_json::Value> {
        info!("[CRITIC] Evaluating all agent outputs (iteration {})", ctx.iteration);

        let plan_confidence = input["plan"]["confidence"].as_f64().unwrap_or(0.8);
        let build_confidence = input["build"]["confidence"].as_f64().unwrap_or(0.8);
        let test_confidence = input["test"]["confidence"].as_f64().unwrap_or(0.8);

        let min_conf = plan_confidence.min(build_confidence).min(test_confidence);
        let approved = min_conf >= 0.80;

        let mut required_fixes = Vec::new();
        if plan_confidence < 0.80 {
            required_fixes.push(json!({"agent": "Planner", "issue": "Plan confidence below threshold", "priority": "P1"}));
        }
        if build_confidence < 0.85 {
            required_fixes.push(json!({"agent": "Builder", "issue": "Build confidence below threshold", "priority": "P1"}));
        }
        if test_confidence < 0.80 {
            required_fixes.push(json!({"agent": "Tester", "issue": "Test confidence below threshold", "priority": "P1"}));
        }

        let rationale = if approved {
            "All agent outputs meet confidence thresholds.".to_string()
        } else {
            format!("Minimum confidence {:.2} below threshold. Required fixes: {}", min_conf, required_fixes.len())
        };

        let verdict = if approved { "APPROVE" } else { "CONTINUE" };

        Ok(json!({
            "verdict": verdict,
            "rationale": rationale,
            "required_fixes": required_fixes,
            "confidence": min_conf,
            "session_trace": [],
        }))
    }

    fn name(&self) -> &str {
        "critic"
    }

    fn prompt_template(&self) -> &'static str {
        include_str!("../../prompts/critic.md")
    }

    fn mcp_tools(&self) -> Vec<String> {
        vec!["chimera_gate".to_string(), "chimera_audit".to_string(), "chimera_prove".to_string()]
    }

    fn min_confidence(&self) -> f64 {
        0.90
    }
}
