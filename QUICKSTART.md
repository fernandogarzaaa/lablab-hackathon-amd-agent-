# Quickstart

Get Chimera Builder running in 2 minutes.

## Try the Demo (no install needed)

```bash
docker run --rm ghcr.io/fernandogarzaaa/lablab-hackathon-amd-agent-/demo
```

Or build locally:

```bash
docker build -t chimera-builder .
docker run --rm chimera-builder
```

## Install from Source

```bash
# Prerequisites: Rust 1.75+ or Docker

# Clone
git clone https://github.com/fernandogarzaaa/lablab-hackathon-amd-agent-.git
cd lablab-hackathon-amd-agent-

# Run the demo
cargo run -- demo

# Analyze a real repo
cargo run -- analyze https://github.com/example/repo
```

## What It Does

```
Analyst    →  Scans repo structure, tech stack, detects issues
Planner    →  Generates prioritized roadmap (P0-P3)
Builder    →  Generates code changes for each roadmap task
Tester     →  Simulates workflow validation
Critic     →  Cross-agent evaluation → APPROVE/CONTINUE/ABORT
```

Each agent's output flows through the ChimeraLang confidence middleware:
- **Detect** → hallucination detection
- **Confident** → confidence gating
- **Constrain** → output validation
- **Audit** → integrity proofs

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Orchestrator                          │
│  Plan → Execute → Evaluate → Improve → Repeat           │
├──────────────────────────────────────────────────────────┤
│  Analyst  │  Planner  │  Builder  │  Tester  │  Critic  │
│           │           │           │          │          │
│  Detect   │  Roadmap  │  Generate │  Validate│  Approve │
│  Issues   │  Tasks    │  Code     │  Tests   │  Gate    │
├──────────────────────────────────────────────────────────┤
│  Confidence Middleware  │  Three-Layer Memory            │
│  Detect → Confident → Constrain → Audit    │  SQLite + ChromaDB + FTS5         │
└──────────────────────────────────────────────────────────┘
```
