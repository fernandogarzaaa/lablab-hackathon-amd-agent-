# Chimera Builder

**Autonomous Multi-Agent Software Engineering System**

A production-grade Rust system that analyzes GitHub repositories, generates development roadmaps, implements improvements, and iteratively self-improves through a confidence-gated evaluation loop.

## Architecture

```
User → Coordinator → Confidence Middleware → Agent Swarm → Memory Layer
                                              ↓
                                    Analysis → Planning → Building → Testing → Critic
```

### Components

- **Task Decomposer** — Parses repo URLs, validates access, routes to orchestrator
- **Agent Swarm** — 5 specialized agents (Analyst, Planner, Builder, Tester, Critic) communicating via typed message passing
- **Confidence Middleware** — Global ChimeraLang integration (detect, confident, constrain, audit)
- **Memory Layer** — Three-layer system: SQLite (structured), ChromaDB (semantic), FTS5 (full-text)
- **Execution Engine** — Sandboxed code execution, file I/O, Git operations

## Quick Start

```bash
# Clone and build
git clone <repo>
cd chimera-builder
cargo build --release

# Analyze a repository
./target/release/chimera-builder analyze https://github.com/example/repo
```

## Try It

No Rust install? Try the demo in one command:

```bash
docker build -t chimera-builder .
docker run --rm chimera-builder
```

Or with cargo:

```bash
cargo run -- demo
```

The demo runs the full agent loop — Analyst → Planner → Builder → Tester → Critic — with live terminal output, confidence scores, and a final verdict. Zero config, zero API keys.

## Agent Descriptions

| Agent | Role | Key Tools |
|-------|------|-----------|
| Analyst | Repo analysis, tech stack detection, issue detection | chimera_detect, chimera_confident |
| Planner | Roadmap generation, task prioritization | chimera_confident, chimera_gate |
| Builder | Code implementation, refactoring | chimera_detect, chimera_prove |
| Tester | Workflow simulation, regression testing | chimera_detect, chimera_explore |
| Critic | Cross-agent review, final gate | chimera_gate, chimera_audit, chimera_prove |

## Tech Stack

- **Async runtime:** tokio
- **CLI:** clap
- **Database:** SQLite (rusqlite) + FTS5
- **Vector search:** ChromaDB (sidecar container)
- **Git operations:** git2 (libgit2 FFI)
- **Serialization:** serde
- **Error handling:** thiserror + anyhow

## Project Structure

```
chimera-builder/
├── src/
│   ├── agents/       # Agent trait + 5 agents
│   ├── analysis/     # Repo parser, dependency mapper, issue detector
│   ├── cli/          # CLI interface (clap)
│   ├── core/         # Orchestrator, state machine, types
│   ├── execution/    # Code runner, file manager, git ops, test sim
│   ├── llm/          # LLM client + model router
│   ├── memory/       # SQLite store, semantic search, FTS5, ability extraction
│   ├── middleware/   # Confidence middleware (ChimeraLang integration)
│   └── main.rs       # Entry point
├── config/           # Agent + model configs
├── prompts/          # Agent prompt templates
└── tests/            # Integration tests
```

## License

MIT
