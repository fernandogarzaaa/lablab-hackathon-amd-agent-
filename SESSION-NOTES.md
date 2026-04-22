# Chimera Builder — Session Notes

## Session Goal
Production harden the Chimera Builder autonomous multi-agent system (Rust) so it's ready for real-world use. The codebase was a solid PoC with working tests but zero production-grade infrastructure.

---

## What Was Done

### Phase 1: LLM Resilience (CRITICAL)
- **`src/llm/client_shared.rs`** — New file: `SharedHttpClient` (single connection-pooled `reqwest::Client`, cloned to all providers) + `HttpClientConfig` (10s connect timeout, 120s request timeout)
- **`retry_with_backoff()`** — 5 retries, exponential backoff (1s, 2s, 4s, 8s, 30s max). Only retriggers on transient errors (5xx, 429, network failures). Permanent 4xx fails immediately.
- **All 4 providers updated** (`anthropic.rs`, `openai.rs`, `ollama.rs`, `openai_compat.rs`) — accept `SharedHttpClient` in `new()`, wrap `generate()` body in `retry_with_backoff()`
- **`src/llm/routing.rs`** — `ModelRouter` now takes `SharedHttpClient` in `new()`, passes cloned client to all providers
- **`src/cli/analyze.rs`** — Creates `SharedHttpClient` before building router
- Removed dead `do_generate()` methods (logic moved into retry wrapper)
- **Result**: Zero warnings, zero errors on build

### Phase 2: CLI Ergonomics + Config Validation
- **`src/cli/validation.rs`** — New file: `validate_analyze_command()` validates URL format, provider enum, config_dir existence, LoopConfig bounds
- **CLI flags added** to `AnalyzeCommand`:
  - `-v, --verbose` — global flag → debug logging
  - `--dry-run` — plan phase only, no execution
  - `--max-iterations <N>` (default: 3, range: 1-100)
  - `--min-confidence <F>` (default: 0.85, range: 0.0-1.0)
- **Global `-v/--verbose`** on `Cli` struct → affects tracing level globally
- **`Orchestrator::run_dry()`** — new method: runs Analyst + Planner only, returns FinalOutput with plan but no build/test
- **Result**: `cargo test` 6/6 pass

### Phase 3: Security Fixes
- **Path traversal in `FileManager.write()`** — canonicalization check rejects `..` components that escape temp_dir, atomic writes via temp file + rename
- **Process orphans in `CodeRunner.execute()`** — on timeout: `start_kill()` + `wait()` to reap child process tree
- **Git branch creation in `GitOps.create_branch()`** — was a no-op (just `find_branch`), now actually creates branch + updates HEAD
- **`GitOps.commit()`** — uses `**/*` recursive glob, handles initial commit (no HEAD)

### Phase 4: Verification
- `cargo build --release` — zero warnings
- `cargo test` — 6/6 pass
- `chimera-builder analyze --help` — all flags present
- Committed and pushed to GitHub

---

## Current State

| Area | Status |
|------|--------|
| LLM providers (Anthropic, OpenAI, Ollama, OpenAI-compatible) | Resilient with retries/timeouts |
| CLI | Full flag set, validation, verbose, dry-run |
| Security (path traversal, process safety, git) | Fixed |
| Build | Zero warnings |
| Tests | 6/6 pass |
| Code coverage | Low — no unit tests, only integration |

---

## What's Left (Next Steps)

### High Priority
1. **LLM integration tests** — The retry logic and shared client have no tests. Add unit tests for:
   - `retry_with_backoff()` — test all 5 retry attempts, transient vs permanent errors
   - `SharedHttpClient` timeout configuration
   - `validate_analyze_command()` — all validation paths
   - `FileManager::write()` path traversal rejection

2. **Config loading** — `load_models()` silently falls back on errors. Add:
   - TOML schema validation on load
   - Error messages that tell users which fields are wrong
   - Example `config/models.toml` with all options documented

3. **Demo mode** — the `DemoCommand` has no CLI flags. Give it the same `--verbose` support as `AnalyzeCommand`

### Medium Priority
4. **Memory leak** — `clone_repo()` uses `std::mem::forget(temp_dir)` to keep temp dir alive. This leaks memory. Better approach: use `tempfile::TempDir` with `into_path()` + `std::fs::remove_dir_all()` on cleanup

5. **Error handling in `Orchestrator::run_dry()`** — uses `.unwrap()` on `find(|a| ...)` calls. These can't fail (agents are always present) but could be `expect()` for clarity

6. **GitOps `create_branch` checkout** — sets HEAD but doesn't update working directory. Should call `git2::Repository::checkout_index()` or `git2::Repository::reset()` with hard mode

7. **`LlmClient::new_demo()` fallback** — in `analyze.rs` if `router.create_client()` returns None, uses demo. This hides configuration errors

### Low Priority
8. **Logging** — `warn!()` calls in `analyze.rs` with static "VALIDATION WARNING" prefix could use `tracing::warn!` with structured fields instead of string formatting

9. **`main.rs` tracing init** — called before `cmd.run()`, so `--verbose` on `Demo` affects tracing but `AnalyzeCommand`'s `--verbose` also tries to set it (duplicated). Should consolidate: init tracing inside each command's `run()` to avoid double-init errors

10. **Benchmark setup** — `benches/` was deleted during hardening. Re-add benchmarks for:
    - `retry_with_backoff()` latency (normal + failure cases)
    - `FileManager.write()` path traversal check vs canonical write
    - LLM provider throughput

11. **CI/CD** — no GitHub Actions. Add:
    - `cargo test` on PR
    - `cargo clippy` on PR
    - `cargo build --release` on merge
    - Optional: `cargo fmt --check`

---

## Files Changed (Current Session)

| File | Change |
|------|--------|
| `src/llm/client_shared.rs` | NEW — SharedHttpClient + retry_with_backoff |
| `src/cli/validation.rs` | NEW — config validation |
| `src/llm/mod.rs` | Add client_shared re-export |
| `src/llm/providers/mod.rs` | Re-export SharedHttpClient |
| `src/llm/providers/anthropic.rs` | SharedHttpClient + retry |
| `src/llm/providers/openai.rs` | SharedHttpClient + retry |
| `src/llm/providers/ollama.rs` | SharedHttpClient + retry |
| `src/llm/providers/openai_compat.rs` | SharedHttpClient + retry |
| `src/llm/routing.rs` | Accept SharedHttpClient |
| `src/cli/mod.rs` | Export validation |
| `src/cli/analyze.rs` | New flags, validation, dry-run |
| `src/core/orchestrator.rs` | run_dry() method |
| `src/execution/file_manager.rs` | Path traversal + atomic writes |
| `src/execution/runner.rs` | Process kill on timeout |
| `src/execution/git_ops.rs` | Branch creation + recursive commit |
| `src/main.rs` | Global --verbose flag |

## How to Resume

1. `cd /root/chimera-builder`
2. `git log --oneline` — last commit is the hardening work
3. Pick a section above from "What's Left"
4. `git checkout -b feature/your-feature-name`
5. Work on it, then commit and push
