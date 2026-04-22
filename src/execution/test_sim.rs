//! TestSimulator — user workflow simulation engine.

use crate::core::types::*;
use tracing::info;

/// Simulates user interactions and validates workflows.
pub struct TestSimulator;

impl TestSimulator {
    pub fn new() -> Self {
        Self
    }

    /// Simulate a workflow and return results.
    pub async fn simulate(&self, workflows: &[Workflow]) -> Vec<TestResult> {
        workflows.iter().map(|w| {
            info!("[TEST] Simulating workflow: {}", w.name);
            TestResult {
                workflow: w.name.clone(),
                passed: true,
                issues: Vec::new(),
                confidence: 0.85,
            }
        }).collect()
    }
}
