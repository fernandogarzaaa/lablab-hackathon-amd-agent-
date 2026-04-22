//! Analyze command handler.

use crate::agents::*;
use crate::core::types::*;
use crate::core::Orchestrator;
use crate::memory::MemoryStore;
use anyhow::Result;
use clap::Parser;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(about = "Analyze a GitHub repository and generate improvements")]
pub struct AnalyzeCommand {
    /// GitHub repository URL
    #[arg(name = "repo_url")]
    pub repo_url: String,
}

impl AnalyzeCommand {
    pub async fn run(self) -> Result<()> {
        info!("Starting Chimera Builder analysis: {}", self.repo_url);

        // Create memory store
        let mut memory = MemoryStore::new("/tmp/chimera-builder-mem.db")?;
        let session_id = memory.create_session(&self.repo_url).await?;
        info!("Session created: {}", session_id);

        // Build agent swarm
        let agents: Vec<Box<dyn crate::agents::Agent>> = vec![
            Box::new(AnalystAgent::new()),
            Box::new(PlannerAgent::new()),
            Box::new(BuilderAgent::new()),
            Box::new(TesterAgent::new()),
            Box::new(CriticAgent::new()),
        ];

        // Create orchestrator
        let config = LoopConfig::default();
        let mut orchestrator = Orchestrator::new(agents, memory, config);

        // Run the loop
        info!("=== RUNNING AGENT LOOP ===");
        let result = orchestrator.run().await;

        match result {
            Ok(output) => {
                info!("\n=== ANALYSIS COMPLETE ===");
                info!("Verdict: {:?}", output.final_verdict);
                info!("Iterations: {}", output.total_iterations);
                if let Some(build) = &output.build_output {
                    info!("Changes implemented: {}", build.changes.len());
                }
                if let Some(test) = &output.test_output {
                    info!("Workflows tested: {}", test.workflows_tested.len());
                }

                // Print audit report
                info!("\n=== AUDIT REPORT ===");
                info!("Architecture: {}", output.audit_report.architecture);
                info!("Tech stack: {:?}", output.audit_report.tech_stack);
                info!("Issues found: {}", output.audit_report.issues.len());
                for issue in &output.audit_report.issues {
                    info!("  [{}] {}: {} ({}{})",
                        issue.severity, issue.category, issue.description,
                        issue.file_path.as_deref().unwrap_or("n/a"),
                        issue.line.map(|l| format!(":{}", l)).unwrap_or_default(),
                    );
                }

                // Print roadmap
                info!("\n=== ROADMAP ===");
                for task in &output.plan.roadmap {
                    info!("  [{:?}] {} (effort: {:?})", task.priority, task.title, task.effort);
                }

                println!("\nChimera Builder analysis complete. Session persisted.");
                Ok(())
            }
            Err(e) => {
                error!("Analysis failed: {}", e);
                Err(e)
            }
        }
    }
}
