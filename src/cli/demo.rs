//! Demo — simulated agent loop with colorful terminal output.
//!
//! Run with: `chimera-builder demo`
//! Requires no API keys, no network access, no git repo.

use anyhow::Result;
use clap::Parser;
use colored::{Color, Colorize};

#[derive(Parser, Debug)]
#[command(about = "Run a simulated agent loop with live terminal output")]
pub struct DemoCommand;

fn banner_line(s: &str, color: Color) -> String {
    let cs: colored::ColoredString = s.color(color);
    cs.bold().to_string()
}

impl DemoCommand {
    pub async fn run(self) -> Result<()> {
        println!();
        print_banner();
        println!();

        println!("{}", banner_line("━━━ Phase 1: ANALYSIS ━━━", Color::Cyan));
        println!();
        run_phase(
            "ANALYST",
            "Agent",
            "Scanning repository structure and tech stack...",
            &[
                ("Architecture", "Monorepo — 13 directories, 51 files"),
                ("Tech Stack", "Rust, tokio, clap, serde, rusqlite, tracing, axum"),
                ("Issues Found", "3 critical, 2 high, 1 medium"),
                ("Confidence", "92%"),
            ],
        );

        println!();
        println!("{}", banner_line("━━━ Phase 2: PLANNING ━━━", Color::Yellow));
        println!();
        run_phase(
            "PLANNER",
            "Agent",
            "Generating roadmap from detected issues...",
            &[
                ("P0", "Security: Add input validation middleware"),
                ("P0", "Security: Implement rate limiting"),
                ("P1", "Testing: Add integration test suite"),
                ("P2", "Performance: Add Redis caching layer"),
                ("Estimated Cost", "22.0 agent-hours"),
                ("Confidence", "88%"),
            ],
        );

        println!();
        println!("{}", banner_line("━━━ Phase 3: BUILD ━━━", Color::Blue));
        println!();
        run_phase(
            "BUILDER",
            "Agent",
            "Generating code changes from roadmap...",
            &[
                ("Files Modified", "7"),
                ("Files Created", "12"),
                ("Lines Changed", "1,847"),
                ("Confidence", "87%"),
            ],
        );

        println!();
        println!("{}", banner_line("━━━ Phase 4: VALIDATION ━━━", Color::Magenta));
        println!();
        run_phase(
            "TESTER",
            "Agent",
            "Simulating workflow execution and validation...",
            &[
                ("Workflows Tested", "5"),
                ("Passed", "5"),
                ("Failed", "0"),
                ("Confidence", "84%"),
            ],
        );

        println!();
        println!("{}", banner_line("━━━ Phase 5: EVALUATION ━━━", Color::Green));
        println!();
        run_phase(
            "CRITIC",
            "Critic",
            "Cross-agent evaluation and final gate...",
            &[
                ("Plan Confidence", "88%"),
                ("Build Confidence", "87%"),
                ("Test Confidence", "84%"),
                ("Minimum Confidence", "84% ≥ 80% threshold"),
                ("VERDICT", "APPROVED ✓"),
            ],
        );

        let sep = banner_line(
            "╔══════════════════════════════════════════════════════════════╗",
            Color::Cyan,
        );
        let msg = banner_line(
            "║              LOOP COMPLETE — ALL PHASES PASSED              ║",
            Color::Cyan,
        );
        let bot = banner_line(
            "╚══════════════════════════════════════════════════════════════╝",
            Color::Cyan,
        );
        println!("{}", sep);
        println!("{}", msg);
        println!("{}", bot);
        println!();

        let summary = banner_line("  Summary:", Color::White);
        println!("{}", summary);
        println!("    {} iterations executed", banner_line("1", Color::White));
        println!(
            "    {} issues detected, {} fixed",
            banner_line("6", Color::White),
            banner_line("6", Color::White),
        );
        println!(
            "    {} files modified",
            banner_line("19", Color::White),
        );
        println!(
            "    {} lines generated",
            banner_line("1,847", Color::White),
        );
        println!(
            "    {} final confidence",
            banner_line("84%", Color::White),
        );
        println!();
        println!(
            "  {} Agent Swarm: Analyst → Planner → Builder → Tester → Critic",
            banner_line("━", Color::White),
        );
        println!();
        println!(
            "  {} Confidence Middleware: Hallucination detection ✓ Confidence gating ✓ Consensus collapse",
            banner_line("━", Color::White),
        );
        println!();
        println!(
            "  {} Memory: SQLite structured + ChromaDB vector + FTS5 full-text",
            banner_line("━", Color::White),
        );
        println!();

        let cmd = banner_line("chimera-builder analyze https://github.com/example/repo", Color::Cyan);
        let label = "  To try with a real repo:".dimmed().to_string();
        println!("{}", label);
        println!("    {}", cmd);
        println!();

        Ok(())
    }
}

fn print_banner() {
    let title = "  CHIMERA BUILDER";
    let sub = "Autonomous Multi-Agent Software Engineering System";
    let version = "  v0.1.0 — Demo Mode";
    let top = "  ╔═══════════════════════════════════════════════════════╗";
    let bot = "  ╚═══════════════════════════════════════════════════════╝";

    println!("{}", banner_line(title, Color::White));
    println!("{}", banner_line(top, Color::White));
    println!("  {}", banner_line(sub, Color::Yellow));
    println!("  {}", banner_line(version, Color::Cyan));
    println!("{}", banner_line(bot, Color::White));
    println!();
}

fn run_phase(phase_name: &str, agent_name: &str, status: &str, details: &[(&str, &str)]) {
    println!(
        "  ┌─ {} ({})",
        banner_line(phase_name, Color::Yellow),
        agent_name.dimmed()
    );
    println!("  │  {}", status.dimmed());
    println!("  ├─ Results:");
    for (key, value) in details {
        let key_col = banner_line(key, Color::White);
        let val = if *key == "VERDICT" {
            banner_line(value, Color::Green)
        } else {
            banner_line(value, Color::White)
        };
        println!("  │    {}: {}", key_col, val);
    }
    println!("  └─────────────────────────────");
}
