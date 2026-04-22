//! Chimera Builder — Entry point
//!
//! Usage: chimera-builder analyze <repo_url>

use anyhow::Result;
use clap::Parser;
use chimera_builder::cli::{AnalyzeCommand, DemoCommand};

#[derive(Parser, Debug)]
#[command(name = "chimera-builder", version, about = "Autonomous Multi-Agent Software Engineering System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Analyze a GitHub repository and generate improvements
    Analyze(AnalyzeCommand),
    /// Run a simulated agent loop with live terminal output
    Demo(DemoCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    match cli.command {
        Commands::Analyze(cmd) => cmd.run().await,
        Commands::Demo(cmd) => cmd.run().await,
    }
}
