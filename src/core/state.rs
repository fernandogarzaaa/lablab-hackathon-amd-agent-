//! Loop state machine with typed transitions.

use crate::core::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum LoopState {
    Running(RunningState),
    Approved(ApprovedState),
    Aborted(AbortedState),
}

impl LoopState {
    pub fn running(max_iterations: u32) -> Self {
        Self::Running(RunningState {
            current_iteration: 0,
            max_iterations,
            agent_outputs: Vec::new(),
            audit_log: Vec::new(),
            abilities_loaded: Vec::new(),
        })
    }

    pub fn approve(self, final_output: FinalOutput) -> Result<Self> {
        match self {
            LoopState::Running(state) => {
                Ok(Self::Approved(ApprovedState {
                    final_output,
                    total_iterations: state.current_iteration,
                }))
            }
            _ => Err(anyhow::anyhow!("Cannot approve from non-running state")),
        }
    }

    pub fn abort(self, reason: String) -> Result<Self> {
        let partial = match self {
            LoopState::Running(state) => Some(FinalOutput {
                repo_url: String::new(),
                audit_report: AuditReport {
                    repo_url: String::new(),
                    architecture: String::new(),
                    tech_stack: Vec::new(),
                    issues: Vec::new(),
                    confidence: 0.0,
                },
                plan: Plan {
                    roadmap: Vec::new(),
                    total_estimated_cost: 0.0,
                    confidence: 0.0,
                },
                build_output: None,
                test_output: None,
                final_verdict: Verdict::Abort,
                total_iterations: state.current_iteration,
                abilities_extracted: Vec::new(),
                session_trace: state.audit_log,
            }),
            _ => None,
        };
        Ok(Self::Aborted(AbortedState {
            reason,
            partial_output: partial,
        }))
    }

    pub fn is_running(&self) -> bool {
        matches!(self, LoopState::Running(_))
    }

    pub fn current_iteration(&self) -> u32 {
        match self {
            LoopState::Running(s) => s.current_iteration,
            _ => 0,
        }
    }

    pub fn log(&mut self, entry: SessionTraceEntry) -> Result<()> {
        match self {
            LoopState::Running(s) => s.audit_log.push(entry),
            _ => return Err(anyhow::anyhow!("Cannot log to non-running state")),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningState {
    pub current_iteration: u32,
    pub max_iterations: u32,
    pub agent_outputs: Vec<AgentMessage>,
    pub audit_log: Vec<SessionTraceEntry>,
    pub abilities_loaded: Vec<String>,
}

impl RunningState {
    pub fn next_iteration(&mut self) -> Result<()> {
        if self.current_iteration >= self.max_iterations {
            return Err(anyhow::anyhow!("Max iterations reached"));
        }
        self.current_iteration += 1;
        Ok(())
    }

    pub fn record_output(&mut self, output: AgentMessage) -> Result<()> {
        self.agent_outputs.push(output);
        Ok(())
    }

    pub fn add_ability(&mut self, ability: String) -> Result<()> {
        self.abilities_loaded.push(ability);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedState {
    pub final_output: FinalOutput,
    pub total_iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortedState {
    pub reason: String,
    pub partial_output: Option<FinalOutput>,
}
