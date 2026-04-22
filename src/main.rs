//! Chimera Builder — Entry point
//!
//! Usage: chimera-builder analyze <repo_url>

use anyhow::Result;
use clap::Parser;
use chimera_builder::cli::{AnalyzeCommand, DemoCommand};

#[derive(Parser, Debug)]
#[command(name = "chimera-builder", version, about = "Autonomous Multi-Agent Software Engineering System")]
struct Cli {
    /// Enable verbose (debug-level) logging
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

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

    // Initialize tracing with verbosity support
    let filter = if cli.verbose {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "debug".into())
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into())
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    match cli.command {
        Commands::Analyze(cmd) => cmd.run().await,
        Commands::Demo(cmd) => cmd.run().await,
    }
}
