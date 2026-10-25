# Auto-Fix & Hot Reload Pipeline

**Category:** Architecture / CI
**Source:** Backlog item (High) — "Auto-fix & hot reload: automated bug detection and resolution."
**Date:** 2026-10-25

## Description

The closed loop that makes overnight agent iteration safe: detect a problem → an agent fixes it → the fix is verified → hot-reloaded into the running sim — with rollback if anything fails. Connects `ollama-integration.md`, `hot-reload.md`, `deterministic-simulation-ai-safety.md`, `agent-failure-handling.md`, `component-testing-framework.md`.

## The pipeline

```
detect → diagnose → fix → verify → reload → confirm
   ↑                                            |
   └──────────── rollback on any failure ───────┘
```

| Stage | What happens | Mechanism |
|---|---|---|
| **Detect** | invariant violation, telemetry anomaly, test failure | telemetry thresholds, scheduler invariants, test suite |
| **Diagnose** | localize the fault to a mod/change | replay the deterministic run; bisect component boundaries |
| **Fix** | an agent proposes a patch on an isolated worktree | `infra/worktree.rs`, `infra/ollama.rs`, `infra/pr_flow.rs` |
| **Verify** | build + contract check + replay + tests | `component-testing-framework.md` stages |
| **Reload** | hot-swap the fixed mod | `infra/hot_reload.rs` |
| **Confirm** | smoke-tick + telemetry — the sim keeps running correctly | post-reload invariant check |
| **Rollback** | any stage fails → revert to checkpoint, quarantine the change | `save_load.rs` checkpoint, `agent-failure-handling.md` |

## Design principles

- **Every stage is mechanical** — no agent judgment in the pipeline; agents propose, the pipeline verifies. A green checkmark means replay passed, not that the agent *said* it worked.
- **Fail closed** — any ambiguity rolls back; a slow/cautious loop beats a fast/broken one
- **Escalation is a stage** — when the pipeline can't converge, it produces a `needs-gary` artifact (a reproducible failing run + the diagnosis) rather than silently retrying
- **Local models in the loop** — the diagnose/fix stages are exactly what the small local Ollama models are for (`ollama-integration.md`)

## The overnight scenario (the payoff)

The fleet runs sims overnight; a system fails; the pipeline detects it, isolates the bad mod, a local model drafts a fix, replay verifies, hot-reload applies, the run continues — morning's report says "3 failures auto-fixed, 1 needs-gary." That's the whole point of the architecture.

## Rating

The operating loop of the AI-dev architecture — the thing that turns agents into a working dev org.
