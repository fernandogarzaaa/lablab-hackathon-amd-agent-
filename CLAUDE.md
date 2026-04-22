# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Chimera Builder is an autonomous multi-agent software engineering system written in Rust. It analyzes GitHub repositories, generates development roadmaps, implements improvements, and iteratively self-improves through a confidence-gated evaluation loop.

## Key Architecture

- **Agents** (`src/agents/`): 5 agents (Analyst, Planner, Builder, Tester, Critic) implementing the `Agent` trait
- **Orchestrator** (`src/core/orchestrator.rs`): Manages the Plan→Execute→Evaluate→Improve→Repeat loop
- **Confidence Middleware** (`src/middleware/`): Global ChimeraLang integration — every agent output flows through confidence/hallucination gates
- **Memory** (`src/memory/`): Three-layer system (SQLite + ChromaDB + FTS5) with ability extraction
- **Analysis** (`src/analysis/`): Repo parser, dependency mapper, issue detector

## Commands

```bash
# Build
cargo build --release

# Run
cargo run -- analyze https://github.com/example/repo

# Test
cargo test

# Benchmarks
cargo bench
```

## Development Notes

- The orchestrator owns the agent lifecycle — never call agents directly
- All agent communication goes through typed `AgentMessage` — no direct function calls
- The confidence middleware MUST process all agent outputs before routing
- Memory persistence uses SQLite; test with `/tmp/` paths
- Prompt templates are in `prompts/` and loaded via `include_str!()`
- The `analysis/` module currently uses simulated data — real repo analysis requires git2 integration
