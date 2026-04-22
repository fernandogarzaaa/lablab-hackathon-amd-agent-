//! Analyze command handler.

use crate::agents::*;
use crate::cli::validate_analyze_command;
use crate::core::types::*;
use crate::core::Orchestrator;
use crate::execution::{FileManager, CodeRunner};
use crate::llm::{LlmClient, ModelRouter, ProviderType};
use crate::llm::client_shared::{HttpClientConfig, SharedHttpClient};
use crate::memory::MemoryStore;
use anyhow::Result;
use clap::Parser;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(about = "Analyze a GitHub repository and generate improvements")]
pub struct AnalyzeCommand {
    /// GitHub repository URL
    #[arg(name = "repo_url")]
    pub repo_url: String,

    /// LLM provider to use (anthropic, openai, ollama, openai-compatible)
    #[arg(long, default_value = "anthropic")]
    pub provider: String,

    /// Path to config directory containing models.toml
    #[arg(long, default_value = "config")]
    pub config_dir: String,

    /// Enable verbose (debug-level) logging
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Run without executing any changes (dry run)
    #[arg(long)]
    pub dry_run: bool,

    /// Maximum number of self-improvement iterations (1-100)
    #[arg(long, default_value_t = 3)]
    pub max_iterations: u32,

    /// Minimum confidence threshold to accept changes (0.0-1.0)
    #[arg(long, default_value = "0.85")]
    pub min_confidence: f64,
}

impl AnalyzeCommand {
    pub async fn run(self) -> Result<()> {
        // Validate arguments early
        let (validation_result, warnings) = validate_analyze_command(
            &self.repo_url,
            &self.provider,
            &self.config_dir,
            self.max_iterations,
            self.min_confidence,
        );
        for w in &warnings {
            warn!("{}: {}", "VALIDATION WARNING", w);
        }
        validation_result?;

        info!("Starting Chimera Builder analysis: {}", self.repo_url);

        // Create memory store
        let mut memory = MemoryStore::new("/tmp/chimera-builder-mem.db")?;
        let session_id = memory.create_session(&self.repo_url).await?;
        info!("Session created: {}", session_id);

        // Clone the repo into a temp directory
        let repo_path = self.clone_repo(&self.repo_url)?;

        // Configure LLM provider
        let provider_type = self.resolve_provider()?;
        let models_path = format!("{}/models.toml", self.config_dir);
        let routing = match crate::llm::config::load_models(&models_path, provider_type) {
            Ok(r) => r,
            Err(e) => {
                info!("Could not load models config: {}. Using defaults.", e);
                let mut r = crate::llm::config::RoutingConfig {
                    agents: std::collections::HashMap::new(),
                    provider: ProviderType::Anthropic,
                };
                r.provider = provider_type;
                r
            }
        };
        let http_config = HttpClientConfig::default();
        let shared_client = SharedHttpClient::new(&http_config);
        let router = ModelRouter::new(provider_type, routing, shared_client);

        // Build agent swarm with LLM client
        let llm_client = router.create_client("analyst").unwrap_or_else(LlmClient::new_demo);
        let agents: Vec<Box<dyn crate::agents::Agent>> = vec![
            Box::new(AnalystAgent::new()),
            Box::new(PlannerAgent::new()),
            Box::new(BuilderAgent::new()),
            Box::new(TesterAgent::new()),
            Box::new(CriticAgent::new()),
        ];

        // Create execution tools
        let file_manager = FileManager::new(repo_path.clone());
        let code_runner = CodeRunner::new();

        // Create orchestrator
        let config = LoopConfig {
            max_iterations: self.max_iterations,
            min_confidence: self.min_confidence,
            ..LoopConfig::default()
        };
        if self.dry_run {
            info!("DRY RUN MODE — no changes will be executed");
        }
        let mut orchestrator = Orchestrator::new(
            agents, memory, config, repo_path, llm_client, file_manager, code_runner,
        );

        // Run the loop
        info!("=== RUNNING AGENT LOOP ===");
        let result = if self.dry_run {
            orchestrator.run_dry().await
        } else {
            orchestrator.run().await
        };

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

    fn clone_repo(&self, url: &str) -> Result<String> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().to_string_lossy().to_string();
        // Keep temp_dir alive for the lifetime of this process
        // The orchestrator will use this path before temp_dir is dropped
        std::mem::forget(temp_dir);
        info!("Cloning {} into {}", url, path);

        // Try to clone; if it fails (e.g., no network), fall back to demo mode
        match crate::execution::GitOps::clone(url, &path) {
            Ok(_) => Ok(path),
            Err(e) => {
                warn!("Git clone failed: {}. Using demo mode (simulated data).", e);
                // Return a dummy path so the rest of the pipeline still works in demo mode
                Ok("/dev/null".to_string())
            }
        }
    }

    fn resolve_provider(&self) -> Result<ProviderType> {
        match self.provider.to_lowercase().as_str() {
            "anthropic" => Ok(ProviderType::Anthropic),
            "openai" => Ok(ProviderType::OpenAi),
            "ollama" => Ok(ProviderType::Ollama),
            "openai-compatible" | "openai_compat" | "compatible" => Ok(ProviderType::OpenAiCompatible),
            other => Err(anyhow::anyhow!("Unknown provider: {}. Valid: anthropic, openai, ollama, openai-compatible", other)),
        }
    }
}
