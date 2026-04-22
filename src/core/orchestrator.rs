//! Orchestrator — manages the Plan→Execute→Evaluate→Improve→Repeat loop.

use crate::agents::base::AgentContext;
use crate::core::state::{LoopState, RunningState};
use crate::core::types::*;
use crate::middleware::ConfidenceMiddleware;
use crate::memory::MemoryStore;
use anyhow::Result;
use serde_json::Value;
use tracing::{info, warn};

pub struct Orchestrator {
    state: LoopState,
    config: LoopConfig,
    agents: Vec<Box<dyn crate::agents::Agent>>,
    memory: MemoryStore,
    middleware: ConfidenceMiddleware,
}

impl Orchestrator {
    pub fn new(agents: Vec<Box<dyn crate::agents::Agent>>, memory: MemoryStore, config: LoopConfig) -> Self {
        let middleware = ConfidenceMiddleware::new();
        Self {
            state: LoopState::running(config.max_iterations),
            config,
            agents,
            memory,
            middleware,
        }
    }

    pub async fn run(&mut self) -> Result<FinalOutput> {
        let RunningState { ref mut current_iteration, .. } = match &mut self.state {
            LoopState::Running(s) => s.clone(),
            _ => return Err(anyhow::anyhow!("Orchestrator not in running state")),
        };

        loop {
            if *current_iteration >= self.config.max_iterations {
                return self.handle_abort("Max iterations reached".to_string()).await;
            }

            tracing::info!("[{}/{}] Starting iteration", *current_iteration + 1, self.config.max_iterations);
            self.memory.record_iteration(*current_iteration).await?;

            let plan_output = self.execute_plan_phase(*current_iteration).await?;
            let execution_output = self.execute_execution_phase(&plan_output, *current_iteration).await?;
            let critique = self.execute_evaluation_phase(&plan_output, &execution_output, *current_iteration).await?;

            self.state.log(SessionTraceEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                agent: AgentName::Critic,
                action: format!("Verdict: {}", critique.verdict),
                confidence: critique.confidence,
            })?;

            match critique.verdict {
                Verdict::Approve => {
                    info!("Loop approved after {} iterations", *current_iteration + 1);
                    let final_output = FinalOutput {
                        repo_url: String::new(),
                        audit_report: plan_output.audit_report,
                        plan: plan_output.plan,
                        build_output: Some(execution_output.build_output),
                        test_output: Some(execution_output.test_output),
                        final_verdict: Verdict::Approve,
                        total_iterations: *current_iteration + 1,
                        abilities_extracted: Vec::new(),
                        session_trace: self.get_trace(),
                    };
                    let new_state = self.state.clone().approve(final_output)?;
                    self.state = new_state;
                    if let LoopState::Approved(s) = &self.state {
                        return Ok(s.final_output.clone());
                    }
                    unreachable!()
                }
                Verdict::Continue => {
                    info!("Loop continuing with {} fixes required", critique.required_fixes.len());
                    for fix in &critique.required_fixes {
                        warn!("  Fix: [{:?}] - {}", fix.agent, fix.issue);
                    }
                    if let LoopState::Running(ref mut s) = self.state {
                        s.next_iteration()?;
                    } else {
                        return Err(anyhow::anyhow!("State changed unexpectedly"));
                    }
                }
                Verdict::Abort => {
                    return self.handle_abort(critique.rationale.clone()).await;
                }
            }
        }
    }

    async fn handle_abort(&self, reason: String) -> Result<FinalOutput> {
        Err(anyhow::anyhow!("Aborted: {}", reason))
    }

    async fn execute_plan_phase(&mut self, iteration: u32) -> Result<PlanOutput> {
        let analyst = self.agents.iter().find(|a| a.name() == "analyst").unwrap();
        let planner = self.agents.iter().find(|a| a.name() == "planner").unwrap();

        let ctx = AgentContext::new(iteration);
        let analysis = analyst.run(Value::Null, &ctx).await?;
        let analysis_clone = analysis.clone();

        let analyzed = self.middleware.process(analysis, 0.85).await?;

        let plan = planner.run(analyzed.data, &ctx).await?;

        let plan_val: Plan = serde_json::from_value(plan["plan"].clone())?;
        let audit_val: AuditReport = serde_json::from_value(analysis_clone["audit_report"].clone())?;

        Ok(PlanOutput { audit_report: audit_val, plan: plan_val })
    }

    async fn execute_execution_phase(&mut self, plan: &PlanOutput, iteration: u32) -> Result<ExecutionOutput> {
        let builder = self.agents.iter().find(|a| a.name() == "builder").unwrap();
        let tester = self.agents.iter().find(|a| a.name() == "tester").unwrap();

        let ctx = AgentContext::new(iteration);
        let build = builder.run(serde_json::to_value(&plan.plan).unwrap(), &ctx).await?;

        let built = self.middleware.process(build, 0.85).await?;
        let build_out: BuildOutput = serde_json::from_value(built.data["build_output"].clone())?;

        let test = tester.run(built.data, &ctx).await?;
        let test_val: TestOutput = serde_json::from_value(test["test_output"].clone())?;

        Ok(ExecutionOutput {
            build_output: build_out,
            test_output: test_val,
        })
    }

    async fn execute_evaluation_phase(&mut self, plan: &PlanOutput, exec: &ExecutionOutput, iteration: u32) -> Result<CritiqueOutput> {
        let critic = self.agents.iter().find(|a| a.name() == "critic").unwrap();
        let ctx = AgentContext::new(iteration);

        let all_data = serde_json::json!({
            "plan": plan.plan,
            "build": exec.build_output,
            "test": exec.test_output,
        });

        let output = critic.run(all_data, &ctx).await?;
        let gated: crate::middleware::types::ProcessedOutput<serde_json::Value> = self.middleware
            .process::<serde_json::Value>(output, 0.90)
            .await?;

        let critique: CritiqueOutput = serde_json::from_value(gated.data)?;
        Ok(critique)
    }

    fn get_trace(&self) -> Vec<SessionTraceEntry> {
        match &self.state {
            LoopState::Running(s) => s.audit_log.clone(),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct PlanOutput {
    audit_report: AuditReport,
    plan: Plan,
}

#[derive(Debug, Clone)]
struct ExecutionOutput {
    build_output: BuildOutput,
    test_output: TestOutput,
}
