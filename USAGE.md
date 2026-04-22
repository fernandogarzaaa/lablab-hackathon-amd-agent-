# Usage Guide

## Prerequisites

Choose one:

**Option A: Docker** (recommended)
```bash
docker --version
```

**Option B: Rust toolchain**
```bash
rustc --version   # 1.85+
cargo --version
```

---

## Command 1: Run the Demo (simulated agent loop)

```bash
# Docker (instant, no install)
docker build -t chimera-builder .
docker run --rm chimera-builder

# Or with cargo
cargo run -- demo
```

**What happens:** No API keys, no real git repo, no external services needed.

5 agents run sequentially in a loop:

```
Phase 1: ANALYST (Agent)
  → Scans repo structure, detects tech stack, finds issues
  → Output: audit_report with 13 dirs, 7 tech items, 2 issues
  → Confidence: 92%

Phase 2: PLANNER (Agent)
  → Takes Analyst's issues, generates prioritized roadmap (P0-P3)
  → Output: roadmap with 4 tasks sorted by priority
  → Confidence: 88%

Phase 3: BUILDER (Agent)
  → Takes roadmap, generates file changes
  → Output: 7 files modified, 12 created, 1,847 lines
  → Confidence: 87%

Phase 4: TESTER (Agent)
  → Simulates workflow validation
  → Output: 5 workflows tested, 5 passed
  → Confidence: 84%

Phase 5: CRITIC (Agent)
  → Reviews all outputs, takes minimum confidence
  → min(88%, 87%, 84%) = 84% >= 80% threshold -> APPROVED
  → Verdict: APPROVED

Loop ends after 1 iteration with APPROVED verdict.
```

---

## Command 2: Analyze a Real Repository

```bash
cargo run -- analyze https://github.com/example/repo
```

**What happens:**

1. Creates a memory store (SQLite at `/tmp/chimera-builder-mem.db`)
2. Starts a new session
3. Runs the same 5-agent loop:
   - Analyst: parses repo structure (using git2 crate)
   - Planner: generates roadmap from detected issues
   - Builder: generates code changes
   - Tester: validates workflows
   - Critic: confidence gate -> APPROVED/CONTINUE/ABORTED
4. If Critic says CONTINUE, loops back up to 3 times
5. Persists session data to SQLite
6. Prints summary

**Output:**

```
[INFO] Starting Chimera Builder analysis: https://github.com/...
[INFO] Session created: abc-123-def
[INFO] [ANALYST] Starting repository analysis (iteration 0)
[INFO] [PLANNER] Generating roadmap (iteration 0)
[INFO] [BUILDER] Generating code changes (iteration 0)
[INFO] [TESTER] Simulating workflows (iteration 0)
[INFO] [CRITIC] Evaluating all agent outputs (iteration 0)
[INFO] Loop approved after 1 iterations

=== ANALYSIS COMPLETE ===
Verdict: APPROVED
Iterations: 1
Changes implemented: 12
Workflows tested: 5

=== AUDIT REPORT ===
Architecture: Detected 45 directories, 234 files
Tech stack: ["Rust", "tokio", "serde", "sqlx"]
Issues found: 8
  [HIGH] Missing error handling: src/main.rs:42
  [MEDIUM] No tests: src/processor.rs

=== ROADMAP ===
  [P0] Add error handling (effort: Large)
  [P1] Add tests (effort: Medium)
  [P2] Add logging (effort: Small)
```

---

## Architecture Logic

### Data Flow

```
[Orchestrator Loop]

for iteration in 0..max_iterations:

  [Plan Phase]
    Analyst.run(input)
      -> audit_report + issues + tech_stack
      -> middleware.process() -> detect -> gate

    Planner.run(analyzed_output)
      -> plan (roadmap with prioritized tasks)
      -> middleware.process() -> detect -> gate

  [Execute Phase]
    Builder.run(plan.plan)
      -> build_output (file changes)
      -> middleware.process() -> detect -> gate

    Tester.run(built_data)
      -> test_output (workflows tested)

  [Evaluate Phase]
    Critic.run({plan, build, test})
      -> critique = {verdict, rationale, fixes}
      -> middleware.process() -> detect -> gate

  match critique.verdict:
    APPROVED -> return FinalOutput
    CONTINUE -> loop again (if iteration < max)
    ABORTED -> return error
```

### Confidence Middleware Pipeline

Every agent output passes through this pipeline:

```
detect -> confident -> constrain -> audit

detect:     Check for absolute certainty markers (hallucination)
confident:  Compute confidence score (content-based)
constrain:  Gate if score < threshold (0.85 for plan/build, 0.90 for critic)
audit:      Record audit entry with SHA-256 hash
```

### Loop State Machine

```
[Running: iteration 0]
  -> Critic says CONTINUE
[Running: iteration 1]
  -> Critic says CONTINUE
[Running: iteration 2]
  -> Critic says APPROVED
[Approved] -> FinalOutput returned

[Running: iteration 3]
  -> Max iterations reached (3)
[Aborted] -> Error returned
```

### Memory (three layers)

- **SQLite**: sessions, tasks, decisions, abilities tables
- **ChromaDB**: vector search over session content (sidecar container)
- **FTS5**: full-text search across all indexed sessions
