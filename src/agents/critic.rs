//! Critic Agent — cross-agent output review and final gate.

use crate::agents::base::{Agent, AgentContext};
use crate::core::types::{AgentName, CritiqueOutput, Priority, RequiredFix, Verdict};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
struct LlmCritiqueOutput {
    verdict: String,
    rationale: String,
    required_fixes: Vec<LlmRequiredFix>,
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct LlmRequiredFix {
    agent: String,
    issue: String,
    priority: String,
}

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

        // Try LLM for critical evaluation
        if let Some(ref llm) = ctx.llm_client {
            if let Some(llm_critique) = self.try_llm_critique(&input, &input, &input, llm).await {
                let verdict = match llm_critique.verdict.to_lowercase().as_str() {
                    "approve" | "approved" | "pass" => Verdict::Approve,
                    "abort" | "aborted" | "fail" => Verdict::Abort,
                    _ => Verdict::Continue,
                };

                let required_fixes: Vec<RequiredFix> = llm_critique.required_fixes.iter().map(|f| {
                    let priority = match f.priority.to_lowercase().as_str() {
                        "p0" | "critical" => Priority::P0,
                        "p1" | "high" => Priority::P1,
                        "p2" | "medium" => Priority::P2,
                        _ => Priority::P3,
                    };
                    let agent = match f.agent.to_lowercase().as_str() {
                        "analyst" => AgentName::Analyst,
                        "planner" => AgentName::Planner,
                        "builder" => AgentName::Builder,
                        "tester" => AgentName::Tester,
                        _ => AgentName::Critic,
                    };
                    RequiredFix { agent, issue: f.issue.clone(), priority }
                }).collect();

                let critique = CritiqueOutput {
                    verdict: verdict.clone(),
                    rationale: llm_critique.rationale.clone(),
                    required_fixes: required_fixes.clone(),
                    confidence: llm_critique.confidence,
                    session_trace: Vec::new(),
                };

                return Ok(serde_json::json!({
                    "verdict": verdict.to_string(),
                    "rationale": critique.rationale.clone(),
                    "required_fixes": required_fixes,
                    "confidence": critique.confidence,
                    "session_trace": Vec::<serde_json::Value>::new(),
                }));
            }
        }

        // Fallback: threshold-based evaluation
        let min_conf = plan_confidence.min(build_confidence).min(test_confidence);
        let approved = min_conf >= 0.80;

        let mut required_fixes = Vec::new();
        if plan_confidence < 0.80 {
            required_fixes.push(RequiredFix { agent: AgentName::Planner, issue: "Plan confidence below threshold".to_string(), priority: Priority::P1 });
        }
        if build_confidence < 0.85 {
            required_fixes.push(RequiredFix { agent: AgentName::Builder, issue: "Build confidence below threshold".to_string(), priority: Priority::P1 });
        }
        if test_confidence < 0.80 {
            required_fixes.push(RequiredFix { agent: AgentName::Tester, issue: "Test confidence below threshold".to_string(), priority: Priority::P1 });
        }

        let rationale = if approved {
            "All agent outputs meet confidence thresholds.".to_string()
        } else {
            format!("Minimum confidence {:.2} below threshold. Required fixes: {}", min_conf, required_fixes.len())
        };

        let verdict = if approved { Verdict::Approve } else { Verdict::Continue };

        let critique = CritiqueOutput {
            verdict: verdict.clone(),
            rationale: rationale.clone(),
            required_fixes: required_fixes.clone(),
            confidence: min_conf,
            session_trace: Vec::new(),
        };

        Ok(serde_json::json!({
            "verdict": verdict.to_string(),
            "rationale": critique.rationale.clone(),
            "required_fixes": critique.required_fixes,
            "confidence": min_conf,
            "session_trace": Vec::<serde_json::Value>::new(),
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

impl CriticAgent {
    async fn try_llm_critique(
        &self,
        plan: &serde_json::Value,
        build: &serde_json::Value,
        test: &serde_json::Value,
        llm: &crate::llm::LlmClient,
    ) -> Option<LlmCritiqueOutput> {
        let system = self.prompt_template();

        let prompt = format!(
            "Evaluate the following multi-agent outputs and make a go/no-go decision.\n\n\
             Plan:\n{}\n\n\
             Build:\n{}\n\n\
             Test:\n{}",
            serde_json::to_string_pretty(plan).unwrap_or_default(),
            serde_json::to_string_pretty(build).unwrap_or_default(),
            serde_json::to_string_pretty(test).unwrap_or_default(),
        );

        let response = llm.generate(system, &prompt).await.ok()?;
        let json_str = response.trim();
        let json_str = if json_str.contains("```") {
            json_str.split("```").nth(1).unwrap_or(json_str)
        } else {
            json_str
        };
        serde_json::from_str(json_str).ok()
    }
}
